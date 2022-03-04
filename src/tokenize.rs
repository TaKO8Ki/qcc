use crate::{Token, TokenKind, Type};
use std::str::Chars;

fn error_at(c: char, input: Chars, index: usize, error: String) -> String {
    let loc: Vec<char> = input
        .clone()
        .enumerate()
        .filter(|(idx, _)| idx <= &index)
        .map(|(_, v)| v)
        .collect();

    String::from(format!(
        "{}{}",
        input.clone().collect::<String>(),
        format!(
            "{}^ {}",
            (1..loc.len()).map(|_| " ").collect::<String>(),
            error
        )
    ))
}

impl Token {
    pub fn new(kind: TokenKind, str: impl Into<String>, loc: usize, line_number: usize) -> Self {
        let tok = Self {
            kind,
            str: str.into(),
            loc,
            line_number,
        };
        tok
    }

    pub fn tokenize(p: String) -> Result<Vec<Token>, String> {
        let mut tokens = vec![];

        let mut line_number = 1;
        let chars = p.chars();
        let chars_vec = p.chars().collect::<Vec<char>>();
        let mut chars_iter = chars.clone().enumerate();

        while let Some((i, p)) = chars_iter.next() {
            log::debug!("tokens={:?}", tokens);

            if p == '\n' {
                line_number += 1;
            }

            if is_line_comments(chars_vec.clone(), p, i) {
                chars_iter.next();
                while let Some((_, p)) = chars_iter.next() {
                    if p == '\n' {
                        break;
                    }
                }
                continue;
            }

            if is_block_comments(chars_vec.clone(), p, i) {
                chars_iter.next();
                match chars_vec[i + 1..].iter().collect::<String>().find("*/") {
                    Some(idx) => {
                        for _ in 0..idx {
                            chars_iter.next();
                        }
                    }
                    None => {
                        return Err(error_at(
                            p,
                            chars,
                            i,
                            "unterminated block comment".to_string(),
                        ));
                    }
                }
                chars_iter.next();
                chars_iter.next();
                continue;
            }

            if p.is_whitespace() {
                continue;
            }

            if p == '"' {
                let token = read_string_literal(&mut chars_iter, i, line_number);
                tokens.push(token?);
                continue;
            }

            if is_ident(p) {
                let mut ident = p.to_string();
                if let Some(next_c) = chars_vec.get(i + 1) {
                    if !(is_ident(*next_c) || is_number(*next_c)) {
                        tokens.push(Self::new(TokenKind::Ident, ident, i, line_number));
                        continue;
                    }
                }
                while let Some((i, c)) = chars_iter.next() {
                    log::debug!("char={}", c);
                    ident.push(c);
                    if let Some(next_c) = chars_vec.get(i + 1) {
                        if !(is_ident(*next_c) || is_number(*next_c)) {
                            break;
                        }
                    }
                }
                tokens.push(Self::new(TokenKind::Ident, ident, i, line_number));
                continue;
            }

            if is_punctuators(p) {
                let mut op = p.to_string();
                if let Some(next_c) = chars_vec.get(i + 1) {
                    if is_cmp_op(format!("{}{}", op, next_c)) {
                        chars_iter.next();
                        op.push(*next_c)
                    };
                }
                tokens.push(Self::new(TokenKind::Punct, op, i, line_number));
                continue;
            }

            if p.is_digit(10) {
                let mut number = vec![p];
                if let Some(next_c) = chars_vec.get(i + 1) {
                    if !next_c.is_digit(10) {
                        tokens.push(Self::new(
                            TokenKind::Num(
                                number
                                    .iter()
                                    .collect::<String>()
                                    .parse::<u16>()
                                    .or_else(|_| {
                                        Err(format!("cannot convert char to integer: {:?}", number))
                                    })?,
                            ),
                            p,
                            i,
                            line_number,
                        ));
                        continue;
                    }
                }
                let mut idx = i;
                while let Some((i, c)) = chars_iter.next() {
                    number.push(c);
                    idx = 1;
                    if let Some(next_c) = chars_vec.get(i + 1) {
                        if !next_c.is_digit(10) {
                            break;
                        }
                    }
                }
                tokens.push(Self::new(
                    TokenKind::Num(number.iter().collect::<String>().parse::<u16>().or_else(
                        |_| Err(format!("cannot convert char to integer: {:?}", number)),
                    )?),
                    p,
                    idx,
                    line_number,
                ));
                continue;
            };
            return Err(error_at(p, chars, i, format!("invalid token: {}", p)));
        }

        tokens.push(Self::new(TokenKind::Eof, "", 0, 0));
        convert_keywords(&mut tokens);
        Ok(tokens)
    }
}

fn is_keyword(token: impl Into<String>) -> bool {
    [
        "return", "if", "else", "while", "for", "int", "char", "sizeof", "struct",
    ]
    .contains(&token.into().as_ref())
}

fn is_punctuators(ch: char) -> bool {
    ch == '+'
        || ch == '-'
        || ch == '*'
        || ch == '/'
        || ch == '('
        || ch == ')'
        || ch == ';'
        || ch == '>'
        || ch == '<'
        || ch == '='
        || ch == '!'
        || ch == '{'
        || ch == '}'
        || ch == '&'
        || ch == ','
        || ch == '['
        || ch == ']'
        || ch == '.'
}

fn is_cmp_op(op: String) -> bool {
    op == "==" || op == "!=" || op == "<=" || op == ">="
}

fn is_ident(ch: char) -> bool {
    ('a'..='z').contains(&ch) || ('A'..='Z').contains(&ch) || ch == '_'
}

fn is_number(ch: char) -> bool {
    ('0'..='9').contains(&ch)
}

fn is_line_comments(chars: Vec<char>, ch: char, i: usize) -> bool {
    if let Some(next_c) = chars.get(i + 1) {
        return format!("{}{}", ch, next_c) == "//";
    }
    false
}

fn is_block_comments(chars: Vec<char>, ch: char, i: usize) -> bool {
    if let Some(next_c) = chars.get(i + 1) {
        return format!("{}{}", ch, next_c) == "/*";
    }
    false
}

fn convert_keywords(tokens: &mut Vec<Token>) {
    for token in tokens.iter_mut() {
        if let TokenKind::Ident = &token.kind {
            if is_keyword(&token.str) {
                token.kind = TokenKind::Keyword;
            }
        }
    }
}

fn read_string_literal(
    chars: &mut impl Iterator<Item = (usize, char)>,
    column_number: usize,
    line_number: usize,
) -> Result<Token, String> {
    let mut str = String::new();
    while let Some((_, c)) = chars.next() {
        log::debug!("string literal={}", c);
        log::debug!("str={}", str);
        if c == '\n' || c == '\0' {
            return Err(format!("unclosed string literal: {:?}", c));
        }
        if c == '"' {
            break;
        }

        str.push(c);
        if c == '\\' {
            str.push(chars.next().unwrap().1);
        }
    }

    let mut buf = String::new();
    let mut chars_iter = str.chars();
    log::debug!("chars_iter={:?}", chars_iter);
    while let Some(c) = chars_iter.next() {
        if c == '\\' {
            buf.push_str(&read_escaped_char(&mut chars_iter));
        } else {
            buf.push(c);
        }
    }
    Ok(Token::new(
        TokenKind::Str {
            str: buf.clone(),
            ty: Box::new(Type::type_char().array_of(buf.len() as u16 + 1)),
        },
        str,
        column_number,
        line_number,
    ))
}

fn from_hex(c: char) -> u8 {
    if '0' <= c && c <= '9' {
        return c as u8 - '0' as u8;
    }
    if 'a' <= c && c <= 'f' {
        return c as u8 - 'a' as u8 + 10;
    }
    return c as u8 - 'A' as u8 + 10;
}

fn read_escaped_char(chars: &mut impl Iterator<Item = char>) -> String {
    let mut c = chars.next().unwrap();
    if '0' <= c && c <= '7' {
        let mut ch = c as u8 - '0' as u8;
        match chars.next() {
            Some(ch) => c = ch,
            None => return (ch as char).to_string(),
        }
        for _ in 0..2 {
            if '0' <= c && c <= '7' {
                ch = (ch << 3) + (c as u8 - '0' as u8);
                match chars.next() {
                    Some(ch) => c = ch,
                    None => break,
                }
            }
        }
        return (ch as char).to_string();
    }

    if c == 'x' {
        if let Some(ch) = chars.next() {
            c = ch
        }
        if !c.is_digit(16) {
            panic!("invalid hex escape sequence");
        }

        let mut ch = from_hex(c);
        while let Some(char) = chars.next() {
            if !char.is_digit(16) {
                break;
            }
            ch = (ch << 4) + from_hex(char);
        }
        return (ch as char).to_string();
    }

    match c {
        'a' => String::from("\u{07}"),
        'b' => String::from("\u{08}"),
        't' => String::from("\u{09}"),
        'n' => String::from("\u{0A}"),
        'v' => String::from("\u{0B}"),
        'f' => String::from("\u{0C}"),
        'r' => String::from("\u{0D}"),
        'e' => String::from("\u{1B}"),
        _ => c.to_string(),
    }
}
