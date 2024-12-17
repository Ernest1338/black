use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Token {
    Print,
    LeftParen,
    RightParen,
    StringLiteral(String),
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
                    let token_length = match &token {
                        Token::Print => "print".len(),
                        Token::LeftParen => "(".len(),
                        Token::RightParen => ")".len(),
                        Token::StringLiteral(s) => s.len() + 2, // Including quotes
                    };
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
                if let Some(Token::LeftParen) = iter.next() {
                    if let Some(Token::StringLiteral(s)) = iter.next() {
                        if let Some(Token::RightParen) = iter.next() {
                            ast_nodes.push(ASTNode::Print(s.clone()));
                        } else {
                            return Err("Expected ')'".to_string());
                        }
                    } else {
                        return Err("Expected string literal".to_string());
                    }
                } else {
                    return Err("Expected '(' after 'print'".to_string());
                }
            }
            _ => return Err(format!("Invalid token: {:?}", token)),
        }
    }

    Ok(ast_nodes)
}
