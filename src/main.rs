use std::env;
use unicode_width::UnicodeWidthStr;

#[derive(PartialEq, Clone, Debug)]
enum TokenKind {
    Reserved,
    Num(u16),
    Eof,
}

enum NodeKind {
    Add,
    Sub,
    Mul,
    Div,
    Num(u16),
}

#[derive(Debug, Clone)]
struct Tokens {
    tokens: Vec<Token>,
    index: usize,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    str: char,
}

struct Node {
    kind: NodeKind,
    lhs: Option<Box<Node>>,
    rhs: Option<Box<Node>>,
}

fn error_at(loc: String, input: String, error: String) -> String {
    String::from(format!(
        "{}\n{}",
        input,
        format!(
            "{}^ {}",
            (1..loc.width()).map(|_| " ").collect::<String>(),
            error
        )
    ))
}

impl Node {
    fn new(kind: NodeKind, lhs: Node, rhs: Node) -> Self {
        Node {
            kind,
            lhs: Some(Box::new(lhs)),
            rhs: Some(Box::new(rhs)),
        }
    }

    fn new_node_num(val: u16) -> Self {
        Node {
            kind: NodeKind::Num(val),
            lhs: None,
            rhs: None,
        }
    }

    fn gen(&self, asm: &mut Vec<String>) {
        if let NodeKind::Num(val) = self.kind {
            asm.push(format!("  push {}", val));
            return;
        }

        if let Some(node) = self.lhs.as_ref() {
            node.gen(asm);
        }
        if let Some(node) = self.rhs.as_ref() {
            node.gen(asm);
        }

        asm.push(String::from("  pop rdi"));
        asm.push(String::from("  pop rax"));

        match self.kind {
            NodeKind::Add => {
                asm.push(String::from("  add rax, rdi"));
            }
            NodeKind::Sub => {
                asm.push(String::from("  sub rax, rdi"));
            }
            NodeKind::Mul => {
                asm.push(String::from("  imul rax, rdi"));
            }
            NodeKind::Div => {
                asm.push(String::from("  cqo"));
                asm.push(String::from("  idiv rdi"));
            }
            _ => {}
        }

        asm.push(String::from("  push rax"));
    }
}

impl Tokens {
    fn new(tokens: Vec<Token>) -> Self {
        Tokens { tokens, index: 0 }
    }

    fn next(&mut self) -> Option<&Token> {
        self.index += 1;
        self.tokens.get(self.index - 1)
    }

    fn token(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn expr(&mut self) -> Node {
        let mut node = self.mul();

        loop {
            if self.consume('+') {
                node = Node::new(NodeKind::Add, node, self.mul());
            } else if self.consume('-') {
                node = Node::new(NodeKind::Sub, node, self.mul());
            } else {
                return node;
            }
        }
    }

    fn mul(&mut self) -> Node {
        let mut node = self.unary();

        loop {
            if self.consume('*') {
                node = Node::new(NodeKind::Mul, node, self.unary());
            } else if self.consume('/') {
                node = Node::new(NodeKind::Div, node, self.unary());
            } else {
                return node;
            }
        }
    }

    fn unary(&mut self) -> Node {
        if self.consume('+') {
            return self.primary();
        } else if self.consume('-') {
            return Node::new(NodeKind::Sub, Node::new_node_num(0), self.primary());
        }
        self.primary()
    }

    fn primary(&mut self) -> Node {
        if self.consume('(') {
            let node = self.expr();
            self.consume(')');
            return node;
        }

        Node::new_node_num(self.expect_number().unwrap())
    }

    fn consume(&mut self, op: char) -> bool {
        let token = self.token();
        if token.kind != TokenKind::Reserved || token.str != op {
            return false;
        }
        self.next();
        true
    }

    fn expect_number(&mut self) -> Result<u16, String> {
        let token = self.token();
        if let TokenKind::Num(val) = token.kind {
            self.next();
            Ok(val)
        } else {
            Err(format!("{} is not number", token.str))
        }
    }
}

impl Token {
    fn new(kind: TokenKind, str: char) -> Self {
        let tok = Self { kind, str };
        tok
    }

    fn tokenize(p: String) -> Result<Vec<Token>, String> {
        let mut tokens = vec![];
        let chars = p.chars();
        let mut chars_iter = chars.clone().enumerate();

        let mut index = None;
        while let Some((i, p)) = chars_iter.next() {
            index = Some(i);
            if p.is_whitespace() {
                continue;
            }

            if p == '+' || p == '-' || p == '*' || p == '/' || p == '(' || p == ')' {
                tokens.push(Self::new(TokenKind::Reserved, p));
                continue;
            }

            if p.is_digit(10) {
                let mut number = vec![p];
                let mut op = None;
                while let Some((i, c)) = chars_iter.next() {
                    index = Some(i);
                    if !c.is_digit(10) {
                        if !c.is_whitespace() {
                            op = Some(c);
                        }
                        break;
                    }
                    number.push(c);
                }
                tokens.push(Self::new(
                    TokenKind::Num(number.iter().collect::<String>().parse::<u16>().or_else(
                        |_| Err(format!("cannot convert char to integer: {:?}", number)),
                    )?),
                    p,
                ));
                if let Some(op) = op {
                    tokens.push(Self::new(TokenKind::Reserved, op));
                }
                continue;
            };
            return Err(error_at(
                chars
                    .clone()
                    .enumerate()
                    .filter(|(idx, _)| idx <= &index.unwrap_or(0))
                    .map(|(_, v)| v)
                    .collect(),
                chars.clone().collect::<String>(),
                "cannot tokenize".to_string(),
            ));
        }

        tokens.push(Self::new(TokenKind::Eof, '\0'));
        Ok(tokens)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let arg = env::args().nth(1).unwrap();

    let chars = arg.chars();
    let mut asm = vec![];

    let tokens = match Token::tokenize(chars.clone().collect::<String>()) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("{}", e);
            return Ok(());
        }
    };

    let mut tokens = Tokens::new(tokens);
    let node = tokens.expr();

    log::debug!("tokens: {:?}", tokens);

    asm.push(String::from(".intel_syntax noprefix"));
    asm.push(String::from(".globl main"));
    asm.push(String::from("main:"));

    node.gen(&mut asm);

    asm.push(String::from("  pop rax"));
    asm.push(String::from("  ret"));

    println!("{}", asm.join("\n"));
    Ok(())
}
