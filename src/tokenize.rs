use crate::{Token, TokenKind, Type};
use unicode_width::UnicodeWidthStr;

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
    pub fn new(kind: TokenKind, str: impl Into<String>) -> Self {
        let tok = Self {
            kind,
            str: str.into(),
        };
        tok
    }

    pub fn tokenize(p: String) -> Result<Vec<Token>, String> {
        let mut tokens = vec![];
        let chars = p.chars();
        let chars_vec = p.chars().collect::<Vec<char>>();
        let mut chars_iter = chars.clone().enumerate();

        while let Some((i, p)) = chars_iter.next() {
            log::debug!("tokens={:?}", tokens);
            if p.is_whitespace() {
                continue;
            }

            if p == '"' {
                let token = read_string_literal(&mut chars_iter);
                tokens.push(token?);
                continue;
            }

            if is_ident(p) {
                let mut ident = p.to_string();
                if let Some(next_c) = chars_vec.get(i + 1) {
                    if !(is_ident(*next_c) || is_number(*next_c)) {
                        tokens.push(Self::new(TokenKind::Ident, ident));
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
                tokens.push(Self::new(TokenKind::Ident, ident));
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
                tokens.push(Self::new(TokenKind::Punct, op));
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
                        ));
                        continue;
                    }
                }
                while let Some((i, c)) = chars_iter.next() {
                    number.push(c);
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
                ));
                continue;
            };
            return Err(error_at(
                chars
                    .clone()
                    .enumerate()
                    .filter(|(idx, _)| idx <= &i)
                    .map(|(_, v)| v)
                    .collect(),
                chars.clone().collect::<String>(),
                "cannot tokenize".to_string(),
            ));
        }

        tokens.push(Self::new(TokenKind::Eof, ""));
        convert_keywords(&mut tokens);
        Ok(tokens)
    }
}

fn is_keyword(token: impl Into<String>) -> bool {
    [
        "return", "if", "else", "while", "for", "int", "char", "sizeof",
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

fn convert_keywords(tokens: &mut Vec<Token>) {
    for token in tokens.iter_mut() {
        if let TokenKind::Ident = &token.kind {
            if is_keyword(&token.str) {
                token.kind = TokenKind::Keyword;
            }
        }
    }
}

fn read_string_literal(chars: &mut impl Iterator<Item = (usize, char)>) -> Result<Token, String> {
    let mut str = String::new();
    while let Some((_, c)) = chars.next() {
        log::debug!("string literal={}", c);
        if c == '\n' || c == '\0' {
            return Err(format!("unclosed string literal: {:?}", c));
        }
        if c == '"' {
            break;
        }
        str.push(c);
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
    ))
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
