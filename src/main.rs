use std::collections::LinkedList;
use std::env;
use std::fs::File;

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
    Comma,
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

#[derive(Debug)]
struct VarScope {
    name: String,
    var: Var,
}

#[derive(Debug)]
struct Scope {
    vars: LinkedList<VarScope>,
}

impl Default for Scope {
    fn default() -> Self {
        Scope {
            vars: LinkedList::new(),
        }
    }
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
    scope: LinkedList<Scope>,
    index: usize,
    functions: LinkedList<Function>,
    string_literal_id: usize,
}

#[derive(Debug)]
struct Function {
    name: String,
    body: Node,
    params: LinkedList<Var>,
    locals: LinkedList<Var>,
    stack_size: Option<usize>,
}

#[derive(Debug, Clone)]
struct Var {
    id: usize,
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
    loc: usize,
    line_number: usize,
}

#[derive(Debug, Clone)]
struct Node {
    kind: NodeKind,
    lhs: Option<Box<Node>>,
    rhs: Option<Box<Node>>,
    ty: Option<Type>,
    token: Token,
}

#[derive(Debug)]
struct Cli {
    output: String,
    input: Option<String>,
    help: bool,
    contents: Option<String>,
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
    use std::io::Write;

    env_logger::init();

    let args = parse_args()?;
    if args.help {
        usage(0)
    }

    let contents = args.contents.unwrap();
    let chars = contents.chars();
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
    asm.push(format!(".file 1 \"{}\"", args.input.unwrap()));
    tokens.codegen(&mut asm);

    let mut file = File::create(args.output)?;
    file.write_all(format!("{}\n", asm.join("\n")).as_bytes())?;
    Ok(())
}

fn usage(status: i32) {
    println!("qcc [ -o <path> ] <file>");
    std::process::exit(status);
}

fn parse_args() -> Result<Cli, String> {
    use std::io::{BufReader, Read};

    let args: Vec<String> = env::args().collect();
    let mut args_iter = args.iter().skip(1);
    let mut cli_args = Cli {
        output: String::from("tmp.s"),
        input: None,
        help: false,
        contents: None,
    };
    log::debug!("args: {:?}", args);

    while let Some(arg) = args_iter.next() {
        if arg == "--help" {
            cli_args.help = true;
            return Ok(cli_args);
        }

        if arg == "-o" {
            cli_args.output = args_iter.next().unwrap().clone();
            continue;
        }

        cli_args.input = Some(arg.clone());
        break;
    }

    log::debug!("cli_args: {:?}", cli_args);

    if let Some(file_path) = &cli_args.input {
        let file = File::open(file_path).map_err(|_| "failed to open a file")?;
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader
            .read_to_string(&mut contents)
            .map_err(|_| "failed to read from a file")?;
        cli_args.contents = Some(contents);
    } else {
        let mut input = String::new();
        let stdin = std::io::stdin();
        stdin
            .lock()
            .read_to_string(&mut input)
            .map_err(|_| "failed to read from pipe")?;
        cli_args.contents = Some(input);
    }

    Ok(cli_args)
}
