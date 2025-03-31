#![allow(dead_code)]

use crate::utils::ErrorType;
use std::{fmt, iter::Peekable, slice::Iter, str::FromStr};

/// Represents different token types for the lexer
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    Let,
    If,
    True,
    False,

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
    LeftBrace,
    RightBrace,
    Comma,

    // Identifiers
    Identifier(String),

    // Literals
    Number(i64),
    StringLiteral(String),
    Bool(Bool),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    Long,
    Float,
    Double,
    Bool,
    Str,
    None,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Bool {
    True,
    False,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = match self {
            Type::Int => "int",
            Type::Long => "long",
            Type::Float => "float",
            Type::Double => "double",
            Type::Str => "str",
            Type::Bool => "bool",
            Type::None => "none",
        };
        write!(f, "{}", type_str)
    }
}

pub fn type_check(var_type: &Type, value: &Expr) -> bool {
    matches!(
        (var_type, value),
        (Type::Str, Expr::StringLiteral(_))
            | (Type::Int, Expr::Number(_) | Expr::BinExpr(_))
            | (Type::Float, Expr::Number(_) | Expr::BinExpr(_))
            | (Type::Double, Expr::Number(_) | Expr::BinExpr(_))
            | (Type::Bool, Expr::Bool(_))
    )
}

impl Token {
    /// Returns the length of the token as it appears in the source
    fn len(&self) -> usize {
        match self {
            Token::Let => 3,
            Token::If => 2,

            Token::StringLiteral(s) => s.len() + 2, // Includes quotes
            Token::Type(Type::Str) => 3,

            Token::Number(n) => n.to_string().len(),
            Token::Identifier(s) => s.len(),

            Token::LeftParen
            | Token::RightParen
            | Token::LeftBrace
            | Token::RightBrace
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

            Token::Type(Type::Bool) => 4,
            Token::Bool(v) => match v {
                Bool::True => 4,
                Bool::False => 5,
            },
            Token::True => 4,
            Token::False => 5,

            Token::Type(Type::None) => 0,
        }
    }
}

impl FromStr for Token {
    type Err = ();

    /// Parses a string into a Token, if possible
    fn from_str(s: &str) -> Result<Token, ()> {
        // println!("fromstr: {s}");
        // Helper for parsing keywords followed by whitespace
        fn parse_keyword(s: &str, keyword: &str, token: &Token) -> Option<Token> {
            if s.starts_with(keyword) {
                let rest = &s[keyword.len()..];
                if rest.is_empty()
                    || rest
                        .chars()
                        .next()
                        .map(|c| !c.is_alphanumeric())
                        .unwrap_or(false)
                {
                    return Some(token.clone());
                }
            }
            None
        }

        // Keywords and types
        // NOTE: sort by length for faster tokenization
        let keywords = [
            ("let", Token::Let),
            ("if", Token::If),
            ("int", Token::Type(Type::Int)),
            ("str", Token::Type(Type::Str)),
            ("bool", Token::Type(Type::Bool)),
            ("long", Token::Type(Type::Long)),
            ("true", Token::True),
            ("false", Token::False),
            ("float", Token::Type(Type::Float)),
            ("double", Token::Type(Type::Double)),
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

        // Identifier (allow underscores in names)
        if s.starts_with(|c: char| c.is_alphabetic() || c == '_') {
            let identifier: String = s
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
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
            ('{', Token::LeftBrace),
            ('}', Token::RightBrace),
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
pub fn lexer(input: &str) -> Result<Vec<Token>, ErrorType> {
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
                Err(_) => {
                    return Err(ErrorType::SyntaxError(format!(
                        "Unexpected token: {remaining}"
                    )))
                }
            }
        }
    }

    Ok(tokens)
}

/// Represents a parsed expression in the abstract syntax tree (AST)
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum Expr {
    FuncCall(FuncCall),
    IfStatement(Box<IfStatement>),
    VariableDeclaration(Box<VariableDeclaration>),
    BinExpr(Box<BinExpr>),
    Block(Vec<Expr>),
    Number(i64),
    Bool(Bool),
    Identifier(String),
    StringLiteral(String),
}

/// Represents a if statement in the AST
#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
    pub comparison: Expr,
    pub block: Vec<Expr>,
}

/// Represents a variable declaration in the AST
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclaration {
    pub identifier: String,
    pub typ: Option<Type>,
    pub value: Expr,
}

/// Represents a function call in the AST
#[derive(Debug, Clone, PartialEq)]
pub struct FuncCall {
    pub name: String,
    pub arguments: Vec<Expr>,
}

/// Represents a binary expression in the AST
#[derive(Debug, Clone, PartialEq)]
pub struct BinExpr {
    pub lhs: Expr,
    pub rhs: Expr,
    pub kind: BinOpKind,
}

/// Represents kinds of binary operators
#[derive(Debug, Clone, PartialEq)]
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
    Bool(Bool),
}

/// Type alias for the AST, a list of expressions
pub type Ast = Vec<Expr>;

/// Parses tokens into expressions and builds an AST
pub struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser instance from a list of tokens
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser {
            tokens: tokens.iter().peekable(),
        }
    }

    /// Parses primary expressions (numbers, identifiers, etc.)
    pub fn parse_primary(&mut self) -> Result<Expr, ErrorType> {
        match self.tokens.next() {
            Some(Token::Number(n)) => Ok(Expr::Number(*n)),
            Some(Token::Bool(v)) => match v {
                Bool::True => Ok(Expr::Bool(Bool::True)),
                Bool::False => Ok(Expr::Bool(Bool::False)),
            },
            Some(Token::True) => Ok(Expr::Bool(Bool::True)),
            Some(Token::False) => Ok(Expr::Bool(Bool::False)),
            Some(Token::StringLiteral(s)) => Ok(Expr::StringLiteral(s.to_owned())), // Handle StringLiteral
            Some(Token::LeftParen) => {
                let expr = self.parse_expr()?;
                if self.tokens.next() != Some(&Token::RightParen) {
                    return Err(ErrorType::SyntaxError("Expected ')'".to_string()));
                }
                Ok(expr)
            }
            Some(Token::LeftBrace) => self.parse_block(), // Handle code block start
            Some(Token::Identifier(name)) => {
                if let Some(Token::LeftParen) = self.tokens.peek() {
                    self.parse_func_call(name)
                } else {
                    Ok(Expr::Identifier(name.to_owned()))
                }
            }
            Some(token) => Err(ErrorType::SyntaxError(format!(
                "Unexpected token: {token:?}",
            ))),
            None => Err(ErrorType::SyntaxError(
                "Unexpected end of input".to_string(),
            )),
        }
    }

    /// Parses a code block: `{ expr1; expr2; ... }`
    pub fn parse_block(&mut self) -> Result<Expr, ErrorType> {
        let mut expressions = Vec::new();

        // Continue parsing until we reach a RightBrace '}'
        while let Some(token) = self.tokens.peek() {
            if **token == Token::RightBrace {
                self.tokens.next(); // Consume the '}'
                break;
            }
            // Allow semicolons or newlines to separate expressions (optional)
            expressions.push(self.parse_expr()?);
        }

        Ok(Expr::Block(expressions))
    }

    /// Parses function calls
    fn parse_func_call(&mut self, name: &str) -> Result<Expr, ErrorType> {
        let mut args = Vec::new();

        // Consume the opening parenthesis '('
        if self.tokens.next() != Some(&Token::LeftParen) {
            return Err(ErrorType::SyntaxError(
                "Expected '(' after function name".to_string(),
            ));
        }

        // Parse arguments until a closing parenthesis ')'
        loop {
            match self.tokens.peek() {
                Some(Token::RightParen) => {
                    self.tokens.next(); // Consume the closing parenthesis ')'
                    break; // Exit the loop after finding the closing parenthesis
                }
                Some(Token::Comma) => {
                    self.tokens.next(); // Consume the comma and continue parsing arguments
                }
                Some(_) => {
                    // Parse the next argument in the function call
                    args.push(self.parse_expr()?);
                }
                None => {
                    return Err(ErrorType::SyntaxError(
                        "Unexpected end of input, expected ')'".to_string(),
                    ));
                }
            }
        }

        // Return the function call expression with arguments
        Ok(Expr::FuncCall(FuncCall {
            name: name.to_string(),
            arguments: args,
        }))
    }

    /// Parses variable declarations
    pub fn parse_variable_declaration(&mut self) -> Result<Expr, ErrorType> {
        self.tokens.next(); // Consume `Token::Let`

        let typ = if let Some(Token::Type(t)) = self.tokens.peek() {
            let t = t.clone();
            self.tokens.next(); // Consume the type token
            Some(t)
        } else {
            None
        };

        let identifier = self
            .tokens
            .next()
            .and_then(|token| match token {
                Token::Identifier(id) => Some(id),
                _ => None,
            })
            .ok_or(ErrorType::SyntaxError(
                "Expected identifier after variable type".to_string(),
            ))?;

        if self.tokens.next() != Some(&Token::Equals) {
            return Err(ErrorType::SyntaxError(
                "Expected '=' after variable name".to_string(),
            ));
        }

        Ok(Expr::VariableDeclaration(Box::new(VariableDeclaration {
            identifier: identifier.to_string(),
            typ,
            value: self.parse_expr()?,
        })))
    }

    // pub fn parse_if_statement(&mut self) -> Result<Expr, ErrorType> {
    //     self.tokens.next(); // Consume `Token::If`
    //
    //     Ok(Expr::IfStatement(Box::new(IfStatement {
    //         comparison: identifier.to_string(),
    //         block,
    //     })))
    // }

    /// Parses binary expressions (e.g., addition, multiplication)
    pub fn parse_binary(&mut self, operators: &[Token]) -> Result<Expr, ErrorType> {
        let mut left = self.parse_primary()?;

        while let Some(op) = self.tokens.peek() {
            if operators.contains(op) {
                let operator = match op {
                    Token::Plus => BinOpKind::Plus,
                    Token::Minus => BinOpKind::Minus,
                    Token::Multiply => BinOpKind::Multiply,
                    Token::Divide => BinOpKind::Divide,
                    _ => unreachable!(),
                };
                self.tokens.next(); // Consume operator

                let right = self.parse_primary()?;

                left = Expr::BinExpr(Box::new(BinExpr {
                    lhs: left,
                    kind: operator,
                    rhs: right,
                }));
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// Parses general expressions
    pub fn parse_expr(&mut self) -> Result<Expr, ErrorType> {
        let peek = match self.tokens.peek() {
            Some(peek) => peek,
            None => {
                return Err(ErrorType::SyntaxError(
                    "Unexpected end of input".to_string(),
                ))
            }
        };

        match peek {
            Token::Let => self.parse_variable_declaration(),
            // Token::If => self.parse_if_statement(),
            _ => self.parse_binary(&[Token::Multiply, Token::Divide, Token::Plus, Token::Minus]),
        }
    }

    /// Parses a complete program into an AST
    pub fn parse(&mut self) -> Result<Ast, ErrorType> {
        let mut ast = Vec::new();

        while self.tokens.peek().is_some() {
            let expr = self.parse_expr()?;
            ast.push(expr);
        }

        Ok(ast)
    }
}
