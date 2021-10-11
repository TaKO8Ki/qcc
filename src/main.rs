use std::env;

#[derive(PartialEq, Clone, Debug)]
enum TokenKind {
    Reserved,
    Num(u16),
    Eof,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    str: char,
}

impl Token {
    fn new(kind: TokenKind, str: char) -> Self {
        let tok = Self { kind, str };
        tok
    }

    fn consume(&self, op: char) -> bool {
        if self.kind != TokenKind::Reserved || self.str != op {
            return false;
        }
        true
    }

    fn expect(&self, op: char) -> bool {
        if self.kind != TokenKind::Reserved || self.str != op {
            return false;
        }
        true
    }

    fn expect_number(&self) -> Result<u16, String> {
        if let TokenKind::Num(val) = self.kind {
            Ok(val)
        } else {
            Err(format!("{} is not number", self.str))
        }
    }

    fn at_eof(&self) -> bool {
        self.kind == TokenKind::Eof
    }

    fn tokenize(p: String) -> Result<Vec<Token>, String> {
        let mut tokens = vec![];
        let mut chars = p.chars();

        while let Some(p) = chars.next() {
            if p.is_whitespace() {
                continue;
            }

            if p == '+' || p == '-' {
                tokens.push(Self::new(TokenKind::Reserved, p));
                continue;
            }

            if p.is_digit(10) {
                let mut number = vec![p];
                let mut op = None;
                while let Some(c) = chars.next() {
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
            }

            return Err("cannot tokenize".to_string());
        }

        Self::new(TokenKind::Eof, '\0');
        Ok(tokens)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = env::args().nth(1).unwrap();

    let chars = arg.chars();

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    let tokens = Token::tokenize(chars.collect::<String>()).unwrap();
    let mut tokens_iter = tokens.iter();

    // println!("tokens: {:?}", tokens);

    println!(
        "  mov rax, {}",
        tokens_iter.next().unwrap().expect_number()?
    );

    while let Some(token) = tokens_iter.next() {
        if token.at_eof() {
            break;
        }
        if token.consume('+') {
            if let Some(token) = tokens_iter.next() {
                println!("  add rax, {}", token.expect_number()?);
            }
            continue;
        }

        if token.expect('-') {
            if let Some(token) = tokens_iter.next() {
                println!("  sub rax, {}", token.expect_number()?);
            }
        } else {
            println!("  sub rax, {}", token.expect_number()?);
        };
    }

    println!("  ret");
    Ok(())
}
