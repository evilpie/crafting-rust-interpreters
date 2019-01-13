use crate::scanner::Token;

pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

#[derive(Debug, Clone)]
pub enum Node {
    Print(Box<Expr>),
    Fun(String, Vec<String>, Box<Node>),
    Return(Box<Expr>),
    While(Box<Expr>, Box<Node>),
    If(Box<Expr>, Box<Node>, Box<Node>),
    ExpressionStatement(Box<Expr>),
    Statements(Vec<Box<Node>>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Eq(Box<Expr>, Box<Expr>),
    Ne(Box<Expr>, Box<Expr>),
    Greater(Box<Expr>, Box<Expr>),
    GreaterEqual(Box<Expr>, Box<Expr>),
    Less(Box<Expr>, Box<Expr>),
    LessEqual(Box<Expr>, Box<Expr>),
    Plus(Box<Expr>, Box<Expr>),
    Minus(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Box<Expr>>),
    Array(Vec<Box<Expr>>),
    Identifier(String),
    Assign(String, Box<Expr>),
    Get(Box<Expr>, Box<Expr>),
    Set(Box<Expr>, Box<Expr>, Box<Expr>),
    Number(i32),
    String(String),
    Boolean(bool),
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        return Parser { tokens, index: 0 };
    }

    pub fn parse(&mut self) -> Result<Node, String> {
        self.statements()
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.index);
        self.index += 1;
        token
    }

    fn current(&mut self) -> Option<&Token> {
        self.tokens.get(self.index)
    }

    fn statements(&mut self) -> Result<Node, String> {
        let mut statements = Vec::new();
        loop {
            statements.push(Box::new(self.statement()?));

            if self.current().is_none() {
                break;
            }
        }
        Ok(Node::Statements(statements))
    }

    fn statement(&mut self) -> Result<Node, String> {
        match self.current() {
            Some(Token::Print) => self.print_statement(),
            Some(Token::Fun) => self.fun_statement(),
            Some(Token::Return) => self.return_statement(),
            Some(Token::While) => self.while_statement(),
            Some(Token::For) => self.for_statement(),
            Some(Token::If) => self.if_statement(),
            Some(Token::OpenBrace) => self.block(),
            _ => self.expression_statement(),
        }
    }

    fn print_statement(&mut self) -> Result<Node, String> {
        self.advance();

        let expr = self.expression()?;
        match self.advance() {
            Some(Token::Semicolon) => {}
            _ => return Err("Expected semicolon after print".to_string()),
        }
        Ok(Node::Print(Box::new(expr)))
    }

    fn fun_statement(&mut self) -> Result<Node, String> {
        self.advance();

        let name = match self.advance() {
            Some(Token::Identifier(name)) => name,
            _ => return Err("expected function name".to_string())
        }.clone();

        match self.advance() {
            Some(Token::OpenParen) => {}
            _ => return Err("expected open parens (".to_string()),
        }

        let mut parameters: Vec<String> = Vec::new();
        match self.current() {
            Some(Token::CloseParen) => {},
            _ => loop {
                match self.advance() {
                    Some(Token::Identifier(name)) => parameters.push(name.clone()),
                    _ => return Err("expected parameter name".to_string()),
                }

                match self.current() {
                    Some(Token::Comma) => {
                        self.advance();
                    }
                    _ => break
                }
            }
        }

        match self.advance() {
            Some(Token::CloseParen) => {}
            _ => return Err("expected close parens )".to_string()),
        }

        let block = self.block()?;
        Ok(Node::Fun(name, parameters, Box::new(block)))
    }

    fn return_statement(&mut self) -> Result<Node, String> {
        self.advance();

        let expr = self.expression()?;
        match self.advance() {
            Some(Token::Semicolon) => {}
            _ => return Err("Expected semicolon after return".to_string()),
        }
        Ok(Node::Return(Box::new(expr)))
    }


    fn while_statement(&mut self) -> Result<Node, String> {
        self.advance();

        match self.advance() {
            Some(Token::OpenParen) => {}
            _ => return Err("expected open parens (".to_string()),
        }

        let condition = self.expression()?;

        match self.advance() {
            Some(Token::CloseParen) => {}
            _ => return Err("expected close parens ) after condition".to_string()),
        }

        let block = self.block()?;
        Ok(Node::While(Box::new(condition), Box::new(block)))
    }

    fn for_statement(&mut self) -> Result<Node, String> {
        self.advance();

        match self.advance() {
            Some(Token::OpenParen) => {}
            _ => return Err("expected open parens ( after for".to_string()),
        }

        let init = match self.current() {
            Some(Token::Semicolon) => {
                self.advance();
                None
            }
            _ => Some(self.expression_statement()?)
        };

        let condition = match self.current() {
            Some(Token::Semicolon) => Expr::Boolean(true),
            _ => self.expression()?
        };

        match self.advance() {
            Some(Token::Semicolon) => {}
            _ => return Err("expected semicolon after condition".to_string()),
        }

        let update = match self.current() {
            Some(Token::CloseParen) => None,
            _ => Some(self.expression()?)
        };

        match self.advance() {
            Some(Token::CloseParen) => {}
            _ => return Err("expected ) after for".to_string()),
        }

        let mut body = self.block()?;

        // Desugaring

        if let Some(update) = update {
            body = Node::Statements(vec![
                Box::new(body),
                Box::new(Node::ExpressionStatement(Box::new(update)))]);
        }

        let while_loop = Node::While(Box::new(condition), Box::new(body));

        Ok(match init {
            Some(init) => Node::Statements(vec![Box::new(init), Box::new(while_loop)]),
            _ => while_loop
        })
    }

    fn if_statement(&mut self) -> Result<Node, String> {
        self.advance();

        match self.advance() {
            Some(Token::OpenParen) => {}
            _ => return Err("expected open parens (".to_string()),
        }

        let condition = self.expression()?;

        match self.advance() {
            Some(Token::CloseParen) => {}
            _ => return Err("expected close parens ) after condition".to_string()),
        }

        let then = self.block()?;
        let other = match self.current() {
            Some(Token::Else) => {
                self.advance();
                self.block()?
            }
            _ => Node::Statements(Vec::new()),
        };
        Ok(Node::If(
            Box::new(condition),
            Box::new(then),
            Box::new(other),
        ))
    }

    fn block(&mut self) -> Result<Node, String> {
        match self.advance() {
            Some(Token::OpenBrace) => {}
            _ => return Err("expected open brace {{".to_string()),
        }

        let mut statements = Vec::new();
        loop {
            match self.current() {
                Some(Token::CloseBrace) => {
                    self.advance();
                    break;
                }
                Some(_) => {}
                None => return Err("missing closing brace }".to_string()),
            }

            statements.push(Box::new(self.statement()?));
        }
        Ok(Node::Statements(statements))
    }

    fn expression_statement(&mut self) -> Result<Node, String> {
        let expr = self.expression()?;
        match self.advance() {
            Some(Token::Semicolon) => {}
            _ => return Err("Expected semicolon after expression".to_string()),
        }
        Ok(Node::ExpressionStatement(Box::new(expr)))
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let left = self.equality()?;

        match self.current() {
            Some(Token::Assign) => {
                self.advance();

                let right = self.assignment()?;
                match left {
                    Expr::Identifier(name) => Ok(Expr::Assign(name.clone(), Box::new(right))),
                    Expr::Get(base, key) => Ok(Expr::Set(base, key, Box::new(right))),
                    _ => Err("Unexpected left hand side of assignment".to_string()),
                }
            }
            _ => Ok(left),
        }
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut left = self.comparison()?;

        loop {
            match self.current() {
                Some(Token::Eq) => {
                    self.advance();

                    let right = self.comparison()?;
                    left = Expr::Eq(Box::new(left), Box::new(right))
                }
                Some(Token::Ne) => {
                    self.advance();

                    let right = self.comparison()?;
                    left = Expr::Ne(Box::new(left), Box::new(right))
                }
                _ => return Ok(left),
            }
        }
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.addition()?;

        loop {
            match self.current() {
                Some(Token::Greater) => {
                    self.advance();

                    let right = self.addition()?;
                    left = Expr::Greater(Box::new(left), Box::new(right))
                }
                Some(Token::GreaterEqual) => {
                    self.advance();

                    let right = self.addition()?;
                    left = Expr::GreaterEqual(Box::new(left), Box::new(right))
                }
                Some(Token::Less) => {
                    self.advance();

                    let right = self.addition()?;
                    left = Expr::Less(Box::new(left), Box::new(right))
                }
                Some(Token::LessEqual) => {
                    self.advance();

                    let right = self.addition()?;
                    left = Expr::LessEqual(Box::new(left), Box::new(right))
                }
                _ => return Ok(left),
            }
        }
    }

    fn addition(&mut self) -> Result<Expr, String> {
        let mut left = self.multiplication()?;

        loop {
            match self.current() {
                Some(Token::Plus) => {
                    self.advance();

                    let right = self.multiplication()?;
                    left = Expr::Plus(Box::new(left), Box::new(right))
                }
                Some(Token::Minus) => {
                    self.advance();

                    let right = self.multiplication()?;
                    left = Expr::Minus(Box::new(left), Box::new(right))
                }
                _ => return Ok(left),
            }
        }
    }

    fn multiplication(&mut self) -> Result<Expr, String> {
        let mut left = self.unary()?;

        loop {
            match self.current() {
                Some(Token::Star) => {
                    self.advance();

                    let right = self.unary()?;
                    left = Expr::Multiply(Box::new(left), Box::new(right))
                }
                _ => return Ok(left),
            }
        }
    }

    fn unary(&mut self) -> Result<Expr, String> {
        match self.current() {
            Some(Token::Minus) => {
                self.advance();

                let expr = self.unary()?;
                Ok(Expr::Minus(Box::new(Expr::Number(0)), Box::new(expr)))
            }
            _ => self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            match self.current() {
                Some(Token::OpenParen) => expr = self.finish_call(expr)?,
                Some(Token::OpenBracket) => {
                    self.advance();

                    let key = self.expression()?;

                    match self.advance() {
                        Some(Token::CloseBracket) => {
                            expr = Expr::Get(Box::new(expr), Box::new(key))
                        }
                        _ => return Err("expecting ] after index".to_string())
                    }
                },
                Some(Token::Dot) => {
                    self.advance();

                    match self.advance() {
                        Some(Token::Identifier(name)) => {
                            expr = Expr::Get(Box::new(expr), Box::new(Expr::String(name.clone())))
                        },
                        _ => return Err("expecting indentifier after dot".to_string())
                    }
                }
                _ => return Ok(expr)
            }
        }
    }

    fn expression_list(&mut self) -> Result<Vec<Box<Expr>>, String> {
        let mut list = Vec::new();
        loop {
            let expr = self.expression()?;
            list.push(Box::new(expr));

            match self.current() {
                Some(Token::Comma) => self.advance(),
                _ => break
            };
        }
        Ok(list)
    }

    fn finish_call(&mut self, expr: Expr) -> Result<Expr, String> {
        self.advance(); // (

        let arguments = match self.current() {
            Some(Token::CloseParen) => Vec::new(),
            _ => self.expression_list()?
        };

        match self.advance() {
            Some(Token::CloseParen) => Ok(Expr::Call(Box::new(expr), arguments)),
            _ => Err("expecting ) after calle".to_string())
        }
    }

    fn primary(&mut self) -> Result<Expr, String> {
        match self.advance() {
            Some(Token::Identifier(name)) => Ok(Expr::Identifier(name.clone())),
            Some(Token::Number(n)) => Ok(Expr::Number(*n)),
            Some(Token::String(string)) => Ok(Expr::String(string.clone())),
            Some(Token::True) => Ok(Expr::Boolean(true)),
            Some(Token::False) => Ok(Expr::Boolean(false)),
            Some(Token::OpenBracket) => self.array(),
            t @ _ => Err(format!("Unexpected {:?}", t)),
        }
    }

    fn array(&mut self) -> Result<Expr, String> {
        let values = match self.current() {
            Some(Token::CloseBracket) => Vec::new(),
            _ => self.expression_list()?
        };

        match self.advance() {
            Some(Token::CloseBracket) => Ok(Expr::Array(values)),
            _ => Err("expecting ] after array literal".to_string())
        }
    }
}
