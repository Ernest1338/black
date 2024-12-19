use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Token {
    Print,
    LeftParen,
    RightParen,
    StringLiteral(String),
}

impl Token {
    fn len(&self) -> usize {
        match self {
            Token::Print => 5,
            Token::LeftParen => 1,
            Token::RightParen => 1,
            Token::StringLiteral(s) => s.len() + 2, // Includes quotes
        }
    }
}

impl FromStr for Token {
    type Err = ();

    fn from_str(s: &str) -> Result<Token, ()> {
        if s.starts_with("print") && s[5..].starts_with(|c: char| c.is_whitespace() || c == '(') {
            return Ok(Token::Print);
        } else if s.starts_with('(') {
            return Ok(Token::LeftParen);
        } else if s.starts_with(')') {
            return Ok(Token::RightParen);
        } else if let Some(stripped) = s.strip_prefix('"') {
            if let Some(end_quote) = stripped.find('"') {
                let string_content = &stripped[..end_quote];
                return Ok(Token::StringLiteral(string_content.to_string()));
            }
        }
        Err(())
    }
}

pub fn lexer(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();

    // Process each line separately
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
