//! Lexer: Tokenize ternary expressions into tokens.


/// Tokens produced by the lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// Ternary literal: -1
    Negative,
    /// Ternary literal: 0
    Zero,
    /// Ternary literal: +1
    Positive,
    /// Room keyword
    Room(String),
    /// Passage keyword (connects rooms)
    Passage(String),
    /// Gate keyword (conditional)
    Gate(String),
    /// Sequence operator `>>`
    Sequence,
    /// Parallel operator `||`
    Parallel,
    /// Branch operator `?`
    Branch,
    /// Left paren
    LParen,
    /// Right paren
    RParen,
    /// Left brace
    LBrace,
    /// Right brace
    RBrace,
    /// Arrow `->`
    Arrow,
    /// Addition `+`
    Plus,
    /// Subtraction / negative `-`
    Minus,
    /// Multiplication `*`
    Star,
    /// Negation `!`
    Bang,
    /// Assignment `=`
    Equals,
    /// Semicolon
    Semicolon,
    /// Comma
    Comma,
    /// Identifier
    Ident(String),
    /// End of input
    Eof,
}

/// Lexer error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub position: usize,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lexer error at position {}: {}", self.position, self.message)
    }
}

impl std::error::Error for LexError {}

/// The lexer tokenizes input strings into a stream of tokens.
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.input.get(self.pos).copied();
        self.pos += 1;
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_ident(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        s
    }

    /// Tokenize the entire input, returning all tokens (including trailing Eof).
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok == Token::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    /// Get the next token.
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace();

        let pos = self.pos;
        let c = match self.advance() {
            Some(c) => c,
            None => return Ok(Token::Eof),
        };

        match c {
            '(' => Ok(Token::LParen),
            ')' => Ok(Token::RParen),
            '{' => Ok(Token::LBrace),
            '}' => Ok(Token::RBrace),
            '*' => Ok(Token::Star),
            '!' => Ok(Token::Bang),
            '=' => Ok(Token::Equals),
            ';' => Ok(Token::Semicolon),
            ',' => Ok(Token::Comma),
            '?' => Ok(Token::Branch),
            '>' => {
                if self.peek() == Some('>') {
                    self.advance();
                    Ok(Token::Sequence)
                } else {
                    Err(LexError {
                        message: format!("Unexpected character: '{}'", c),
                        position: pos,
                    })
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    Ok(Token::Parallel)
                } else {
                    Err(LexError {
                        message: format!("Unexpected character: '{}'", c),
                        position: pos,
                    })
                }
            }
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    Ok(Token::Arrow)
                } else {
                    Ok(Token::Minus)
                }
            }
            '+' => Ok(Token::Plus),
            '0' => Ok(Token::Zero),
            '1' if pos > 0 && self.input.get(pos - 2) == Some(&'+') => Ok(Token::Positive),
            _ if c.is_alphabetic() || c == '_' => {
                let mut ident = String::new();
                ident.push(c);
                while let Some(nc) = self.peek() {
                    if nc.is_alphanumeric() || nc == '_' {
                        ident.push(nc);
                        self.advance();
                    } else {
                        break;
                    }
                }
                match ident.as_str() {
                    "room" => {
                        // Expect: room <name>
                        self.skip_whitespace();
                        let name = self.read_ident();
                        Ok(Token::Room(name))
                    }
                    "passage" => {
                        self.skip_whitespace();
                        let name = self.read_ident();
                        Ok(Token::Passage(name))
                    }
                    "gate" => {
                        self.skip_whitespace();
                        let name = self.read_ident();
                        Ok(Token::Gate(name))
                    }
                    "neg" | "NEG" | "Neg" => Ok(Token::Negative),
                    "zero" | "ZERO" | "Zero" => Ok(Token::Zero),
                    "pos" | "POS" | "Pos" => Ok(Token::Positive),
                    _ => Ok(Token::Ident(ident)),
                }
            }
            _ => Err(LexError {
                message: format!("Unexpected character: '{}'", c),
                position: pos,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_ternary_literals() {
        let mut lexer = Lexer::new("neg zero pos");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 4); // 3 tokens + Eof
        assert_eq!(tokens[0], Token::Negative);
        assert_eq!(tokens[1], Token::Zero);
        assert_eq!(tokens[2], Token::Positive);
    }

    #[test]
    fn test_tokenize_operators() {
        let mut lexer = Lexer::new(">> || ?");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Sequence);
        assert_eq!(tokens[1], Token::Parallel);
        assert_eq!(tokens[2], Token::Branch);
    }

    #[test]
    fn test_tokenize_room() {
        let mut lexer = Lexer::new("room lobby");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Room("lobby".to_string()));
    }

    #[test]
    fn test_tokenize_passage() {
        let mut lexer = Lexer::new("passage north_gate");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Passage("north_gate".to_string()));
    }

    #[test]
    fn test_tokenize_gate() {
        let mut lexer = Lexer::new("gate check_entry");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Gate("check_entry".to_string()));
    }

    #[test]
    fn test_tokenize_parens() {
        let mut lexer = Lexer::new("(){}");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::LParen);
        assert_eq!(tokens[1], Token::RParen);
        assert_eq!(tokens[2], Token::LBrace);
        assert_eq!(tokens[3], Token::RBrace);
    }

    #[test]
    fn test_tokenize_arithmetic() {
        let mut lexer = Lexer::new("+ - * !");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Plus);
        assert_eq!(tokens[1], Token::Minus);
        assert_eq!(tokens[2], Token::Star);
        assert_eq!(tokens[3], Token::Bang);
    }

    #[test]
    fn test_tokenize_arrow() {
        let mut lexer = Lexer::new("->");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Arrow);
    }

    #[test]
    fn test_tokenize_empty() {
        let mut lexer = Lexer::new("");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Eof);
    }

    #[test]
    fn test_tokenize_ident() {
        let mut lexer = Lexer::new("foo bar_baz");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Ident("foo".to_string()));
        assert_eq!(tokens[1], Token::Ident("bar_baz".to_string()));
    }

    #[test]
    fn test_tokenize_complex() {
        let mut lexer = Lexer::new("room start >> passage hall >> gate check");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Room("start".to_string()));
        assert_eq!(tokens[1], Token::Sequence);
        assert_eq!(tokens[2], Token::Passage("hall".to_string()));
        assert_eq!(tokens[3], Token::Sequence);
        assert_eq!(tokens[4], Token::Gate("check".to_string()));
    }
}
