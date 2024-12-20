use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Print, // TODO: refactor into a function call
    LeftParen,
    RightParen,
    Plus,
    Minus,
    Mult,
    Comma,
    Equals,
    Number(i64),
    StringLiteral(String),
    Identifier(String),
}

impl Token {
    fn len(&self) -> usize {
        match self {
            Token::Print => 5,
            Token::StringLiteral(s) => s.len() + 2, // Includes quotes
            Token::Number(n) => n.to_string().len(),
            Token::Identifier(s) => s.len(),
            Token::LeftParen
            | Token::RightParen
            | Token::Plus
            | Token::Minus
            | Token::Mult
            | Token::Equals
            | Token::Comma => 1,
        }
    }
}

impl FromStr for Token {
    type Err = ();

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

        // Token::Print
        if s.starts_with("print") && s[5..].starts_with(|c: char| c.is_whitespace() || c == '(') {
            return Ok(Token::Print);
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
            Some('*') => Ok(Token::Mult),
            Some('(') => Ok(Token::LeftParen),
            Some(')') => Ok(Token::RightParen),
            Some('=') => Ok(Token::Equals),
            Some(',') => Ok(Token::Comma),
            _ => Err(()),
        }
    }
}

pub fn preprocess(code: &str) -> String {
    // handle comments
    code.lines().filter(|x| !x.starts_with("//")).collect()
}

pub fn lexer(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();

    // FIXME: join into one line instead of processing each separately.
    //        Language should not care about new line characters
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

#[derive(Debug, Clone)]
pub enum ASTNode {
    Print(String),
}

pub type Ast = Vec<ASTNode>;

pub fn parser(tokens: &[Token]) -> Result<Vec<ASTNode>, String> {
    let mut iter = tokens.iter();
    let mut ast_nodes = Vec::new();

    while let Some(token) = iter.next() {
        match token {
            Token::Print => {
                if iter.next() != Some(&Token::LeftParen) {
                    return Err("Expected '(' after 'print'".to_string());
                }

                let string_literal = match iter.next() {
                    Some(Token::StringLiteral(s)) => s.clone(),
                    _ => return Err("Expected string literal".to_string()),
                };

                if iter.next() != Some(&Token::RightParen) {
                    return Err("Expected ')'".to_string());
                }

                ast_nodes.push(ASTNode::Print(string_literal));
            }
            _ => return Err(format!("Invalid token: {:?}", token)),
        }
    }

    Ok(ast_nodes)
}
