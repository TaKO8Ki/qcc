use std::collections::LinkedList;
use std::env;

mod codegen;
mod parse;
mod tokenize;

#[derive(PartialEq, Clone, Debug)]
enum TokenKind {
    Reserved,
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
    log::debug!("tokens: {:?}", tokens);
    tokens.program();

    log::debug!("tokens: {:?}", tokens);

    Node::codegen(&mut asm, tokens.code);

    println!("{}", asm.join("\n"));
    Ok(())
}
