use std::collections::LinkedList;
use std::env;

mod codegen;
mod parse;
mod tokenize;
mod r#type;

#[derive(PartialEq, Clone, Debug)]
enum TokenKind {
    Keyword,
    Punct,
    Ident,
    Num(u16),
    Eof,
}

#[derive(Debug)]
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
    Block,
    ExprStmt,
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
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    str: String,
}

#[derive(Debug)]
struct Node {
    kind: NodeKind,
    body: Option<Box<Vec<Node>>>,
    lhs: Option<Box<Node>>,
    rhs: Option<Box<Node>>,
    ty: Option<Type>,
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
    log::debug!("all tokens: {:?}", tokens);
    tokens.program();

    log::debug!("parsed tokens: {:#?}", tokens);
    tokens.codegen(&mut asm);

    println!("{}", asm.join("\n"));
    Ok(())
}
