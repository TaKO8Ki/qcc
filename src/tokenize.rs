use crate::{Token, TokenKind};
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

        let mut index = None;
        while let Some((i, p)) = chars_iter.next() {
            index = Some(i);
            if p.is_whitespace() {
                continue;
            }

            if let Some(index) = index {
                let two_chars = format!(
                    "{}{}",
                    p,
                    &chars_vec
                        .get(index + 1)
                        .map(|p| p.to_string())
                        .unwrap_or_default()
                );
                log::debug!("two_chars={}", two_chars);
                if two_chars.starts_with("==")
                    || two_chars.starts_with("!=")
                    || two_chars.starts_with("<=")
                    || two_chars.starts_with(">=")
                {
                    let second = chars_iter.next();
                    tokens.push(Token::new(
                        TokenKind::Reserved,
                        format!("{}{}", p, second.unwrap().1),
                    ));
                    continue;
                }
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
                    let two_chars = format!(
                        "{}{}",
                        op,
                        chars_vec
                            .get(index.unwrap() + 1)
                            .map(|p| p.to_string())
                            .unwrap_or_default()
                    );
                    log::debug!("two_chars_2={}", two_chars);
                    if two_chars == "=="
                        || two_chars == "!="
                        || two_chars == "<="
                        || two_chars == ">="
                    {
                        let second = chars_iter.next();
                        tokens.push(Token::new(
                            TokenKind::Reserved,
                            format!("{}{}", op, second.unwrap().1),
                        ));
                        continue;
                    } else {
                        tokens.push(Self::new(TokenKind::Reserved, op));
                    }
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

        tokens.push(Self::new(TokenKind::Eof, ""));
        Ok(tokens)
    }
}
