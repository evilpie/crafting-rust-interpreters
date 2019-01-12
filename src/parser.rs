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
    Plus(Box<Expr>, Box<Expr>),
    Minus(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Box<Expr>>),
    Assign(String, Box<Expr>),
    Number(i32),
    Identifier(String),
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
        enum SType {
            Print,
            Fun,
            Return,
            While,
            If,
            Block,
            ExpressionStatement,
        }

        // ToDo: This is a hack to pacify the borrow checker.
        // I am open to suggestions!
        let stype = match self.current() {
            Some(Token::Print) => SType::Print,
            Some(Token::Fun) => SType::Fun,
            Some(Token::Return) => SType::Return,
            Some(Token::While) => SType::While,
            Some(Token::If) => SType::If,
            Some(Token::OpenBrace) => SType::Block,
            _ => SType::ExpressionStatement,
        };

        match stype {
            SType::Print => self.print_statement(),
            SType::Fun => self.fun_statement(),
            SType::Return => self.return_statement(),
            SType::While => self.while_statement(),
            SType::If => self.if_statement(),
            SType::Block => self.block(),
            SType::ExpressionStatement => self.expression_statement(),
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
                _ => return Ok(expr)
            }
        }
    }

    fn finish_call(&mut self, expr: Expr) -> Result<Expr, String> {
        self.advance(); // (

        let mut arguments: Vec<Box<Expr>> = Vec::new();
        match self.current() {
            Some(Token::CloseParen) => {},
            _ => loop {
                let expr = self.expression()?;
                arguments.push(Box::new(expr));

                match self.current() {
                    Some(Token::Comma) => {
                        self.advance();
                    }
                    _ => break
                }
            }
        }

        match self.advance() {
            Some(Token::CloseParen) => Ok(Expr::Call(Box::new(expr), arguments)),
            _ => Err("expecting ) after calle".to_string())
        }
    }

    fn primary(&mut self) -> Result<Expr, String> {
        match self.advance() {
            Some(Token::Identifier(name)) => Ok(Expr::Identifier(name.clone())),
            Some(Token::Number(n)) => Ok(Expr::Number(*n)),
            Some(Token::True) => Ok(Expr::Boolean(true)),
            Some(Token::False) => Ok(Expr::Boolean(false)),
            t @ _ => Err(format!("Unexpected {:?}", t)),
        }
    }
}
