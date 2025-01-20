#![allow(dead_code)]

use std::{fmt, iter::Peekable, str::FromStr};

/// Represents different token types for the lexer
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    Let,

    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Equals,

    // Types
    Type(Type),

    // Punctuation
    LeftParen,
    RightParen,
    Comma,

    // Identifiers
    Identifier(String),

    // Literals
    Number(i64),
    StringLiteral(String),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Pos {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PositionedToken {
    pub token: Token,
    pub position: Pos,
}

pub type Tokens = Vec<PositionedToken>;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    Long,
    Float,
    Double,
    Str,
    None,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = match self {
            Type::Int => "int",
            Type::Long => "long",
            Type::Float => "float",
            Type::Double => "double",
            Type::Str => "str",
            Type::None => "none",
        };
        write!(f, "{}", type_str)
    }
}

pub fn type_check(var_type: &Type, value: &PositionedExpr) -> bool {
    matches!(
        (var_type, &value.expr),
        (Type::Str, Expr::StringLiteral(_))
            | (Type::Int, Expr::Number(_) | Expr::BinExpr(_))
            | (Type::Float, Expr::Number(_) | Expr::BinExpr(_))
            | (Type::Double, Expr::Number(_) | Expr::BinExpr(_))
    )
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
            Token::Type(Type::Int) => 3,
            Token::Type(Type::Long) => 4,
            Token::Type(Type::Float) => 5,
            Token::Type(Type::Double) => 6,
            Token::Type(Type::Str) => 3,
            Token::Type(Type::None) => 0,
        }
    }
}

impl FromStr for Token {
    type Err = ();

    /// Parses a string into a Token, if possible
    fn from_str(s: &str) -> Result<Token, ()> {
        // Helper for parsing keywords followed by whitespace
        fn parse_keyword(s: &str, keyword: &str, token: &Token) -> Option<Token> {
            if s.starts_with(keyword) && s[keyword.len()..].starts_with(|c: char| c.is_whitespace())
            {
                Some(token.clone())
            } else {
                None
            }
        }

        // Keywords and types
        let keywords = [
            ("let", Token::Let),
            ("int", Token::Type(Type::Int)),
            ("long", Token::Type(Type::Long)),
            ("float", Token::Type(Type::Float)),
            ("double", Token::Type(Type::Double)),
            ("str", Token::Type(Type::Str)),
        ];

        for &(keyword, ref token) in &keywords {
            if let Some(parsed_token) = parse_keyword(s, keyword, token) {
                return Ok(parsed_token);
            }
        }

        // Parse numeric tokens
        if let Some(c) = s.chars().next() {
            if c.is_ascii_digit() {
                let number_str: String = s.chars().take_while(|ch| ch.is_ascii_digit()).collect();
                if let Ok(number) = number_str.parse::<i64>() {
                    return Ok(Token::Number(number));
                }
            }
        }

        // String literal
        if let Some(stripped) = s.strip_prefix('"') {
            if let Some(end_quote) = stripped.find('"') {
                let string_content = &stripped[..end_quote];
                return Ok(Token::StringLiteral(string_content.to_string()));
            }
        }

        // Identifier
        if s.starts_with(|c: char| c.is_alphabetic()) {
            let identifier: String = s.chars().take_while(|c| c.is_alphanumeric()).collect();
            return Ok(Token::Identifier(identifier));
        }

        // Single-character tokens
        let single_char_tokens = [
            ('+', Token::Plus),
            ('-', Token::Minus),
            ('*', Token::Multiply),
            ('/', Token::Divide),
            ('(', Token::LeftParen),
            (')', Token::RightParen),
            ('=', Token::Equals),
            (',', Token::Comma),
        ];

        if let Some(&c) = s.chars().next().as_ref() {
            if let Some((_, token)) = single_char_tokens.iter().find(|&&(ch, _)| ch == c) {
                return Ok(token.clone());
            }
        }

        Err(())
    }
}

/// Prepares source code for further processing
pub fn preprocess(code: &str) -> String {
    // Handle comments
    code.lines()
        .filter(|l| !l.starts_with("//"))
        .map(|l| l.split("//").next().unwrap())
        .collect::<Vec<&str>>()
        .join("\n")
}

/// Converts input text into a vector of tokens
pub fn lexer(input: &str) -> Result<Tokens, String> {
    let mut tokens = Vec::new();
    let mut line_number = 1;

    for line in input.lines() {
        let mut remaining = line.trim();
        let mut column_number = 1;

        while !remaining.is_empty() {
            match Token::from_str(remaining) {
                Ok(token) => {
                    let token_length = token.len();
                    remaining = remaining[token_length..].trim_start();

                    let position = Pos {
                        line: line_number,
                        column: column_number,
                    };
                    tokens.push(PositionedToken { token, position });
                    column_number += token_length;
                }
                Err(_) => return Err(format!("Unexpected token: {remaining}")),
            }
        }

        line_number += 1;
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
    pub typ: Option<Type>,
    pub value: PositionedExpr,
}

/// Represents a function call in the AST
#[derive(Debug, Clone)]
pub struct FuncCall {
    pub name: String,
    pub arguments: Vec<PositionedExpr>,
}

/// Represents a binary expression in the AST
#[derive(Debug, Clone)]
pub struct BinExpr {
    pub lhs: PositionedExpr,
    pub rhs: PositionedExpr,
    pub kind: BinOpKind,
}

/// Represents kinds of binary operators
#[derive(Debug, Clone)]
pub enum BinOpKind {
    Plus,
    Minus,
    Multiply,
    Divide,
}

impl BinOpKind {
    /// Returns a string representation of the binary operation
    pub fn to_str(&self) -> &str {
        match self {
            BinOpKind::Plus => "add",
            BinOpKind::Minus => "sub",
            BinOpKind::Multiply => "mul",
            BinOpKind::Divide => "div",
        }
    }
}

/// Represents variables in the AST
// NOTE: Can we store just Expr in the variables? it would allow storing eg functions into vars
#[derive(Debug, Clone)]
pub enum Variable {
    Number(i64),
    StringLiteral(String),
}

#[derive(Debug, Clone)]
pub struct PositionedExpr {
    pub expr: Expr,
    pub position: Pos,
}

/// Type alias for the AST, a list of expressions
pub type Ast = Vec<PositionedExpr>;

/// Parses tokens into expressions and builds an AST
pub struct Parser<'a> {
    iter: Peekable<std::slice::Iter<'a, PositionedToken>>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser instance from a list of tokens
    pub fn new(tokens: &'a [PositionedToken]) -> Self {
        Parser {
            iter: tokens.iter().peekable(),
        }
    }

    /// Advances the iterator and returns the next token, or an error if there are no more tokens.
    fn next_token(&mut self) -> Result<PositionedToken, String> {
        self.iter
            .next()
            .cloned()
            .ok_or_else(|| "Unexpected end of input".to_string())
    }

    /// Peeks at the next token without advancing the iterator, or returns an error if there are no more tokens.
    fn peek_token(&mut self) -> Result<PositionedToken, String> {
        self.iter
            .peek()
            .cloned()
            .ok_or_else(|| "Unexpected end of input while peeking".to_string())
            .cloned() // so many cloooones
    }

    /// Parses primary expressions (numbers, identifiers, etc.)
    pub fn parse_primary(&mut self) -> Result<PositionedExpr, String> {
        let next = self.next_token()?;
        match &next.token {
            Token::Number(n) => Ok(PositionedExpr {
                expr: Expr::Number(*n),
                position: next.position,
            }),
            Token::Identifier(name) => {
                if matches!(self.peek_token()?.token, Token::LeftParen) {
                    self.parse_func_call(name, next.position)
                } else {
                    Ok(PositionedExpr {
                        expr: Expr::Identifier(name.to_owned()),
                        position: next.position,
                    })
                }
            }
            Token::StringLiteral(s) => Ok(PositionedExpr {
                expr: Expr::StringLiteral(s.to_owned()),
                position: next.position,
            }),
            Token::LeftParen => {
                let expr = self.parse_expr()?;
                let next = self.next_token()?;
                if next.token != Token::RightParen {
                    return Err("Expected ')'".to_string());
                }
                Ok(expr)
            }
            token => Err(format!("Unexpected token: {token:?}")),
        }
    }

    /// Parses function calls
    fn parse_func_call(&mut self, name: &str, position: Pos) -> Result<PositionedExpr, String> {
        let mut args = Vec::new();

        // Consume the opening parenthesis '('
        let next = self.next_token()?;
        if next.token != Token::LeftParen {
            return Err("Expected '(' after function name".to_string());
        }

        // Parse arguments until a closing parenthesis ')'
        loop {
            let peek = self.peek_token()?;
            match &peek.token {
                Token::RightParen => {
                    self.iter.next(); // Consume the closing parenthesis ')'
                    break; // Exit the loop after finding the closing parenthesis
                }
                Token::Comma => {
                    self.iter.next(); // Consume the comma and continue parsing arguments
                }
                _ => {
                    // Parse the next argument in the function call
                    args.push(self.parse_expr()?);
                }
            }
        }

        // Return the function call expression with arguments
        Ok(PositionedExpr {
            expr: Expr::FuncCall(FuncCall {
                name: name.to_string(),
                arguments: args,
            }),
            position,
        })
    }

    /// Parses binary expressions (e.g., addition, multiplication)
    pub fn parse_binary(&mut self, operators: &[Token]) -> Result<PositionedExpr, String> {
        let mut left = self.parse_primary()?;

        while self.peek_token().is_ok() {
            let op = self.peek_token()?;
            if operators.contains(&op.token) {
                let operator = match op.token {
                    Token::Plus => BinOpKind::Plus,
                    Token::Minus => BinOpKind::Minus,
                    Token::Multiply => BinOpKind::Multiply,
                    Token::Divide => BinOpKind::Divide,
                    _ => unreachable!(),
                };
                self.iter.next(); // Consume operator
                let right = self.parse_primary()?;
                left = PositionedExpr {
                    expr: Expr::BinExpr(Box::new(BinExpr {
                        lhs: left,
                        kind: operator,
                        rhs: right,
                    })),
                    position: op.position,
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// Parses general expressions
    pub fn parse_expr(&mut self) -> Result<PositionedExpr, String> {
        let peek = self.peek_token()?;
        match peek.token {
            Token::Let => self.parse_variable_declaration(),
            _ => self.parse_binary(&[Token::Multiply, Token::Divide, Token::Plus, Token::Minus]),
        }
    }

    /// Parses variable declarations
    pub fn parse_variable_declaration(&mut self) -> Result<PositionedExpr, String> {
        self.iter.next(); // Consume `Token::Let`

        let peek = self.peek_token()?;
        let typ = if let Token::Type(t) = peek.token {
            let t = t.clone();
            self.iter.next(); // Consume the type token
            Some(t)
        } else {
            None
        };

        let identifier = self
            .iter
            .next()
            .and_then(|token| match &token.token {
                Token::Identifier(id) => Some(id),
                _ => None,
            })
            .ok_or("Expected identifier after variable type")?;

        let next = self.next_token()?;
        if next.token != Token::Equals {
            return Err("Expected '=' after variable name".to_string());
        }

        Ok(PositionedExpr {
            expr: Expr::VariableDeclaration(Box::new(VariableDeclaration {
                identifier: identifier.to_string(),
                typ,
                value: self.parse_expr()?,
            })),
            position: next.position,
        })
    }

    /// Parses a complete program into an AST
    pub fn parse(&mut self) -> Result<Ast, String> {
        let mut ast = Vec::new();
        while let Some(_positioned_token) = self.iter.peek() {
            let expr = self.parse_expr()?;
            ast.push(expr);
        }
        Ok(ast)
    }
}
