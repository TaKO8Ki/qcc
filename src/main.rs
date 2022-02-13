use std::collections::LinkedList;
use std::env;

mod codegen;
mod parse;
mod tokenize;
mod r#type;

#[derive(Clone, Debug)]
enum TokenKind {
    Keyword,
    Punct,
    Ident,
    Num(u16),
    Str { str: String, ty: Box<Type> },
    Eof,
}

#[derive(Debug, Clone)]
enum NodeKind {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Lt,
    Le,
    Assign,
    Return,
    If {
        cond: Box<Node>,
        then: Box<Node>,
        els: Option<Box<Node>>,
    },
    While {
        cond: Box<Node>,
        then: Box<Node>,
    },
    For {
        init: Box<Node>,
        inc: Option<Box<Node>>,
        cond: Option<Box<Node>>,
        then: Box<Node>,
    },
    Deref,
    Addr,
    Block {
        body: Box<Vec<Node>>,
    },
    ExprStmt,
    StmtExpr {
        body: Box<Vec<Node>>,
    },
    FuncCall {
        name: String,
        args: Vec<Node>,
    },
    Var(Var),
    Num(u16),
}

#[derive(Debug, Clone)]
enum TypeKind {
    Int {
        size: u16,
    },
    Char {
        size: u16,
    },
    Func {
        params: Box<Vec<Type>>,
        return_ty: Option<Box<Type>>,
    },
    Ptr {
        size: u16,
        base: Box<Type>,
    },
    Array {
        size: u16,
        len: u16,
        base: Box<Type>,
    },
}

#[derive(Debug, Clone)]
struct Type {
    kind: TypeKind,
    name: Option<Token>,
}

#[derive(Debug)]
struct Tokens {
    locals: LinkedList<Var>,
    globals: LinkedList<Var>,
    tokens: Vec<Token>,
    index: usize,
    functions: LinkedList<Function>,
}

#[derive(Debug)]
struct Function {
    name: String,
    body: Node,
    params: LinkedList<Var>,
    locals: LinkedList<Var>,
}

#[derive(Debug, Clone)]
struct Var {
    name: String,
    offset: usize,
    ty: Type,
    is_local: bool,
    init_data: Option<String>,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    str: String,
}

#[derive(Debug, Clone)]
struct Node {
    kind: NodeKind,
    lhs: Option<Box<Node>>,
    rhs: Option<Box<Node>>,
    ty: Option<Type>,
}

impl Node {
    fn body(&self) -> Option<Vec<Node>> {
        match &self.kind {
            NodeKind::Block { body } | NodeKind::StmtExpr { body } => Some(*body.clone()),
            _ => None,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let arg = parse_args();
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
    log::debug!("all tokens: {:?}", tokens);
    tokens.program();

    log::debug!("parsed tokens: {:#?}", tokens);
    tokens.codegen(&mut asm);

    println!("{}", asm.join("\n"));
    Ok(())
}

fn parse_args() -> String {
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Read;

    if let Some(file_path) = env::args().nth(1) {
        let file = File::open(file_path).expect("failed to open a file");
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader
            .read_to_string(&mut contents)
            .expect("failed to read from a file");
        return contents;
    }
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("failed to read from pipe");
    input
}
