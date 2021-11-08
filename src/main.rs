use std::env;
use unicode_width::UnicodeWidthStr;

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

    fn value(&self) -> String {
        match self.kind {
            TokenKind::Num(val) => val.to_string(),
            _ => self.str.to_string(),
        }
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
        let chars = p.chars();
        let mut chars_iter = chars.clone().enumerate();

        let mut index = None;
        while let Some((i, p)) = chars_iter.next() {
            index = Some(i);
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

        Self::new(TokenKind::Eof, '\0');
        Ok(tokens)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = env::args().nth(1).unwrap();

    let chars = arg.chars();

    let mut result = vec![];

    result.push(String::from(".intel_syntax noprefix"));
    result.push(String::from(".globl main"));
    result.push(String::from("main:"));

    let tokens = match Token::tokenize(chars.clone().collect::<String>()) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("{}", e);
            return Ok(());
        }
    };

    let mut tokens_iter = tokens.iter().enumerate();

    // println!("tokens: {:?}", tokens);

    match tokens_iter.next().unwrap().1.expect_number() {
        Ok(num) => result.push(format!("  mov rax, {}", num)),
        Err(err) => panic!(
            "{}",
            error_at(
                tokens.first().unwrap().value(),
                chars.clone().collect::<String>(),
                err,
            )
        ),
    }
    // println!(
    //     "  mov rax, {}",
    //     tokens_iter.next().unwrap().expect_number()?
    // );

    while let Some((_, token)) = tokens_iter.next() {
        if token.at_eof() {
            break;
        }
        if token.consume('+') {
            if let Some((i, token)) = tokens_iter.next() {
                match token.expect_number() {
                    Ok(num) => result.push(format!("  add rax, {}", num)),
                    Err(err) => {
                        eprintln!(
                            "{}",
                            error_at(
                                tokens
                                    .iter()
                                    .enumerate()
                                    .filter(|(idx, _)| idx <= &i)
                                    .map(|(_, v)| v.value())
                                    .collect(),
                                chars.clone().collect::<String>(),
                                err,
                            )
                        );
                        return Ok(());
                    }
                }
            }
            continue;
        }

        if token.expect('-') {
            if let Some((_, token)) = tokens_iter.next() {
                result.push(format!("  sub rax, {}", token.expect_number()?));
            }
        } else {
            result.push(format!("  sub rax, {}", token.expect_number()?));
        };
    }

    result.push(String::from("  ret"));

    println!("{}", result.join("\n"));
    Ok(())
}
