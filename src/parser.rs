use std::{iter::Peekable, str::FromStr};

/// Represents different token types for the lexer
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Let,
    LeftParen,
    RightParen,
    Plus,
    Minus,
    Multiply,
    Divide,
    Comma,
    Equals,
    Number(i64),
    StringLiteral(String),
    Identifier(String),
}

impl Token {
    /// Returns the length of the token as it appears in the source
    fn len(&self) -> usize {
        match self {
            Token::Let => 3,
            Token::StringLiteral(s) => s.len() + 2, // Includes quotes
            Token::Number(n) => n.to_string().len(),
            Token::Identifier(s) => s.len(),
            Token::LeftParen
            | Token::RightParen
            | Token::Plus
            | Token::Minus
            | Token::Multiply
            | Token::Divide
            | Token::Equals
            | Token::Comma => 1,
        }
    }
}

impl FromStr for Token {
    type Err = ();

    /// Parses a string into a Token, if possible
    fn from_str(s: &str) -> Result<Token, ()> {
        // Token::Number
        if let Some(c) = s.chars().next() {
            if c.is_ascii_digit() {
                let number_str: String = s.chars().take_while(|ch| ch.is_ascii_digit()).collect();

                if let Ok(number) = number_str.parse::<i64>() {
                    return Ok(Token::Number(number));
                }
            }
        }

        // Token::Let
        if s.starts_with("let") && s[Token::Let.len()..].starts_with(|c: char| c.is_whitespace()) {
            return Ok(Token::Let);
        }

        // Token::StringLiteral
        if let Some(stripped) = s.strip_prefix('"') {
            if let Some(end_quote) = stripped.find('"') {
                let string_content = &stripped[..end_quote];
                return Ok(Token::StringLiteral(string_content.to_string()));
            }
        }

        // Token::Identifier
        if s.starts_with(|c: char| c.is_alphabetic()) {
            let identifier: String = s.chars().take_while(|c| c.is_alphanumeric()).collect();
            return Ok(Token::Identifier(identifier));
        }

        // Single char tokens
        match s.chars().next() {
            Some('+') => Ok(Token::Plus),
            Some('-') => Ok(Token::Minus),
            Some('*') => Ok(Token::Multiply),
            Some('/') => Ok(Token::Divide),
            Some('(') => Ok(Token::LeftParen),
            Some(')') => Ok(Token::RightParen),
            Some('=') => Ok(Token::Equals),
            Some(',') => Ok(Token::Comma),
            _ => Err(()),
        }
    }
}

/// Prepares source code for further processing
pub fn preprocess(code: &str) -> String {
    // Handle comments
    code.lines()
        .filter(|l| !l.starts_with("//"))
        .map(|l| l.split("//").next().unwrap())
        .collect::<Vec<&str>>()
        .join(" ")
}

/// Converts input text into a vector of tokens
pub fn lexer(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    for line in input.lines() {
        let mut remaining = line.trim();
        while !remaining.is_empty() {
            match Token::from_str(remaining) {
                Ok(token) => {
                    let token_length = token.len();
                    remaining = remaining[token_length..].trim_start();
                    tokens.push(token);
                }
                Err(_) => return Err(format!("Unexpected token: {}", remaining)),
            }
        }
    }
    Ok(tokens)
}

/// Represents a parsed expression in the abstract syntax tree (AST)
#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum Expr {
    FuncCall(FuncCall),
    VariableDeclaration(Box<VariableDeclaration>),
    BinExpr(Box<BinExpr>),
    Number(i64),
    Identifier(String),
    StringLiteral(String),
}

/// Represents a variable declaration in the AST
#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub identifier: String,
    pub value: Expr,
}

/// Represents a function call in the AST
#[derive(Debug, Clone)]
pub struct FuncCall {
    pub name: String,
    pub arguments: Vec<Expr>,
}

/// Represents a binary expression in the AST
#[derive(Debug, Clone)]
pub struct BinExpr {
    pub lhs: Expr,
    pub kind: BinOpKind,
    pub rhs: Expr,
}

/// Represents kinds of binary operators
#[derive(Debug, Clone)]
pub enum BinOpKind {
    Plus,
    Minus,
    Multiply,
    Divide,
}

/// Represents variables in the AST
#[derive(Debug, Clone)]
pub enum Variable {
    Number(i64),
    StringLiteral(String),
}

/// Type alias for the AST, a list of expressions
pub type Ast = Vec<Expr>;

/// Parses tokens into expressions and builds an AST
pub struct Parser<'a> {
    iter: Peekable<std::slice::Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser instance from a list of tokens
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser {
            iter: tokens.iter().peekable(),
        }
    }

    /// Parses primary expressions (numbers, identifiers, etc.)
    pub fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.iter.next() {
            Some(Token::Number(n)) => Ok(Expr::Number(*n)),
            Some(Token::Identifier(name)) => {
                if let Some(Token::LeftParen) = self.iter.peek() {
                    self.parse_func_call(name)
                } else {
                    Ok(Expr::Identifier(name.to_owned()))
                }
            }
            Some(Token::StringLiteral(s)) => Ok(Expr::StringLiteral(s.to_owned())), // Handle StringLiteral
            Some(Token::LeftParen) => {
                let expr = self.parse_expr()?;
                if self.iter.next() != Some(&Token::RightParen) {
                    return Err("Expected ')'".to_string());
                }
                Ok(expr)
            }
            Some(token) => Err(format!("Unexpected token: {:?}", token)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    /// Parses function calls
    fn parse_func_call(&mut self, name: &str) -> Result<Expr, String> {
        let mut args = Vec::new();

        // Consume the opening parenthesis '('
        if self.iter.next() != Some(&Token::LeftParen) {
            return Err("Expected '(' after function name".to_string());
        }

        // Parse arguments until a closing parenthesis ')'
        loop {
            match self.iter.peek() {
                Some(Token::RightParen) => {
                    self.iter.next(); // Consume the closing parenthesis ')'
                    break; // Exit the loop after finding the closing parenthesis
                }
                Some(Token::Comma) => {
                    self.iter.next(); // Consume the comma and continue parsing arguments
                }
                Some(_) => {
                    // Parse the next argument in the function call
                    args.push(self.parse_expr()?);
                }
                None => {
                    return Err("Unexpected end of input, expected ')'".to_string());
                }
            }
        }

        // Return the function call expression with arguments
        Ok(Expr::FuncCall(FuncCall {
            name: name.to_string(),
            arguments: args,
        }))
    }

    /// Parses binary expressions (e.g., addition, multiplication)
    pub fn parse_binary(
        &mut self,
        parse_operand: fn(&mut Self) -> Result<Expr, String>,
        operators: &[Token],
        make_node: fn(Box<Expr>, BinOpKind, Box<Expr>) -> Expr,
    ) -> Result<Expr, String> {
        let mut left = parse_operand(self)?;
        while let Some(op) = self.iter.peek() {
            if operators.contains(op) {
                let operator = match op {
                    Token::Plus => BinOpKind::Plus,
                    Token::Minus => BinOpKind::Minus,
                    Token::Multiply => BinOpKind::Multiply,
                    Token::Divide => BinOpKind::Divide,
                    _ => unreachable!(),
                };
                self.iter.next(); // Consume operator
                let right = parse_operand(self)?;
                left = make_node(Box::new(left), operator, Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// Parses variable declarations
    pub fn parse_variable_declaration(&mut self) -> Result<Expr, String> {
        self.iter.next(); // Consume `Token::Let`
        if let Some(Token::Identifier(name)) = self.iter.next() {
            if let Some(Token::Equals) = self.iter.next() {
                let value = self.parse_expr()?;
                Ok(Expr::VariableDeclaration(Box::new(VariableDeclaration {
                    identifier: name.clone(),
                    value,
                })))
            } else {
                Err("Expected '=' after variable name".to_string())
            }
        } else {
            Err("Expected identifier after 'let'".to_string())
        }
    }

    /// Parses general expressions
    pub fn parse_expr(&mut self) -> Result<Expr, String> {
        match self.iter.peek() {
            Some(Token::Let) => self.parse_variable_declaration(),
            _ => self.parse_binary(
                |parser| parser.parse_primary(),
                &[Token::Multiply, Token::Divide, Token::Plus, Token::Minus],
                |lhs, kind, rhs| {
                    Expr::BinExpr(Box::new(BinExpr {
                        lhs: *lhs,
                        kind,
                        rhs: *rhs,
                    }))
                },
            ),
        }
    }

    /// Parses a complete program into an AST
    pub fn parse(&mut self) -> Result<Ast, String> {
        let mut ast = Vec::new();
        while let Some(_token) = self.iter.peek() {
            let expr = self.parse_expr()?;
            ast.push(expr);
        }
        Ok(ast)
    }
}
