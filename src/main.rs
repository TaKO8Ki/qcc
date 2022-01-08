use std::collections::LinkedList;
use std::env;

mod codegen;
mod parse;
mod tokenize;

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
    Block,
    LVar(usize),
    Num(u16),
}

#[derive(Debug)]
struct Tokens {
    locals: LinkedList<LVar>,
    tokens: Vec<Token>,
    code: Vec<Node>,
    index: usize,
}

#[derive(Debug)]
struct LVar {
    name: String,
    offset: usize,
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

    log::debug!("parsed tokens: {:?}", tokens);

    Node::codegen(&mut asm, tokens.code);

    println!("{}", asm.join("\n"));
    Ok(())
}
