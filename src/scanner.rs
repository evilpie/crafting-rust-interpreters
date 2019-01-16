#[derive(Debug)]
pub enum Token {
    Assign,
    Eq,
    Ne,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Plus,
    Minus,
    Star,
    Dot,
    Colon,
    Comma,
    Semicolon,
    OpenParen,    // (
    CloseParen,   // )
    OpenBracket,  // [
    CloseBracket, // ]
    OpenBrace,    // {
    CloseBrace,   // }
    Number(i32),
    String(String),
    Identifier(String),
    Var,
    Print,
    Fun,
    Return,
    While,
    For,
    If,
    Else,
    True,
    False,
}

fn single_token(ch: char) -> Option<Token> {
    match ch {
        '+' => Some(Token::Plus),
        '*' => Some(Token::Star),
        '-' => Some(Token::Minus),
        '(' => Some(Token::OpenParen),
        ')' => Some(Token::CloseParen),
        '[' => Some(Token::OpenBracket),
        ']' => Some(Token::CloseBracket),
        '{' => Some(Token::OpenBrace),
        '}' => Some(Token::CloseBrace),
        '.' => Some(Token::Dot),
        ':' => Some(Token::Colon),
        ',' => Some(Token::Comma),
        ';' => Some(Token::Semicolon),
        _ => None,
    }
}

pub fn scan(source: &str) -> Result<Vec<Token>, String> {
    let mut iter = source.chars().peekable();
    let mut tokens = Vec::new();
    loop {
        let n = iter.next();
        if n.is_none() {
            break;
        }

        if let Some(token) = single_token(n.unwrap()) {
            tokens.push(token);
            continue;
        }

        match n.unwrap() {
            i @ 'a'...'z' | i @ 'A'...'Z' => {
                let mut name = String::new();
                name.push(i);

                loop {
                    match iter.peek() {
                        Some('a'...'z') | Some('A'...'Z') | Some('_') => {
                            name.push(iter.next().unwrap())
                        }
                        _ => break,
                    };
                }

                tokens.push(match name.as_str() {
                    "var" => Token::Var,
                    "print" => Token::Print,
                    "fun" => Token::Fun,
                    "return" => Token::Return,
                    "while" => Token::While,
                    "for" => Token::For,
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

            '"' => {
                let mut string = String::new();

                loop {
                    match iter.peek() {
                        Some('"') => {
                            iter.next();
                            break;
                        }
                        Some(_) => string.push(iter.next().unwrap()),
                        _ => break,
                    };
                }

                tokens.push(Token::String(string));
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

            '>' => tokens.push(match iter.peek() {
                Some('=') => {
                    iter.next();
                    Token::GreaterEqual
                }
                _ => Token::Greater,
            }),

            '<' => tokens.push(match iter.peek() {
                Some('=') => {
                    iter.next();
                    Token::LessEqual
                }
                _ => Token::Less,
            }),

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
