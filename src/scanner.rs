#[derive(Debug)]
pub enum Token {
    Assign,
    Eq,
    Ne,
    Plus,
    Minus,
    Star,
    Semicolon,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Number(i32),
    Identifier(String),
    Print,
    While,
    If,
    Else,
    True,
    False,
}

pub fn scan(source: &str) -> Result<Vec<Token>, String> {
    let mut iter = source.chars().peekable();
    let mut tokens = Vec::new();
    loop {
        let n = iter.next();
        if n.is_none() {
            break;
        }
        match n.unwrap() {
            i @ 'a'...'z' => {
                let mut name = String::new();
                name.push(i);

                loop {
                    match iter.peek() {
                        Some('a'...'z') => name.push(iter.next().unwrap()),
                        _ => break,
                    };
                }

                tokens.push(match name.as_str() {
                    "print" => Token::Print,
                    "while" => Token::While,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "true" => Token::True,
                    "false" => Token::False,
                    _ => Token::Identifier(name),
                });
            }

            n @ '0'...'9' => {
                let mut number = String::new();
                number.push(n);

                loop {
                    match iter.peek() {
                        Some('0'...'9') => number.push(iter.next().unwrap()),
                        _ => break,
                    };
                }

                tokens.push(Token::Number(number.parse().unwrap()));
            }

            '!' => tokens.push(match iter.peek() {
                Some('=') => {
                    iter.next();
                    Token::Ne
                }
                _ => panic!("nyi"),
            }),

            '=' => tokens.push(match iter.peek() {
                Some('=') => {
                    iter.next();
                    Token::Eq
                }
                _ => Token::Assign,
            }),

            '+' => {
                tokens.push(Token::Plus);
            }

            '*' => {
                tokens.push(Token::Star);
            }

            '-' => {
                tokens.push(Token::Minus);
            }

            '(' => {
                tokens.push(Token::OpenParen);
            }

            ')' => {
                tokens.push(Token::CloseParen);
            }

            '{' => {
                tokens.push(Token::OpenBrace);
            }

            '}' => {
                tokens.push(Token::CloseBrace);
            }

            ';' => {
                tokens.push(Token::Semicolon);
            }

            ' ' | '\n' => {
                // Ignore whitespace
                continue;
            }

            c @ _ => {
                return Err(format!("Unexpected token: {}", c));
            }
        }
    }

    Ok(tokens)
}
