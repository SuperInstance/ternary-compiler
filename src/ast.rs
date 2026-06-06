//! AST: Abstract syntax tree for ternary programs.

use crate::Ternary;
use crate::lexer::Token;

/// A ternary expression in the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum TernaryExpr {
    /// Literal ternary value.
    Lit(Ternary),
    /// Variable reference.
    Var(String),
    /// Addition: a + b
    Add(Box<TernaryExpr>, Box<TernaryExpr>),
    /// Multiplication: a * b
    Mul(Box<TernaryExpr>, Box<TernaryExpr>),
    /// Negation: !a
    Negate(Box<TernaryExpr>),
    /// Branch: if value is Pos → first path, Neg → second path, Zero → falls through
    Branch(Box<TernaryExpr>, Box<TernaryExpr>, Box<TernaryExpr>),
    /// Sequence: a >> b (do a then b)
    Sequence(Box<TernaryExpr>, Box<TernaryExpr>),
    /// Parallel: a || b (do a and b simultaneously)
    Parallel(Box<TernaryExpr>, Box<TernaryExpr>),
    /// Room expression
    Room(RoomDef),
    /// Passage traversal
    Passage(PassageDef),
    /// Gate (conditional)
    Gate(GateDef),
    /// Block of expressions
    Block(Block),
}

/// Room definition in the AST.
#[derive(Debug, Clone, PartialEq)]
pub struct RoomDef {
    pub name: String,
    pub body: Box<TernaryExpr>,
}

/// Passage definition (connects two rooms).
#[derive(Debug, Clone, PartialEq)]
pub struct PassageDef {
    pub name: String,
    pub from: String,
    pub to: String,
}

/// Gate definition (conditional branch).
#[derive(Debug, Clone, PartialEq)]
pub struct GateDef {
    pub name: String,
    pub condition: Box<TernaryExpr>,
    pub if_neg: Box<TernaryExpr>,
    pub if_zero: Box<TernaryExpr>,
    pub if_pos: Box<TernaryExpr>,
}

/// Block of sequential expressions.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub exprs: Vec<TernaryExpr>,
}

/// Parser error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error: {}", self.message)
    }
}

impl std::error::Error for ParseError {}

/// Recursive descent parser.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), ParseError> {
        let tok = self.advance();
        if &tok == expected {
            Ok(())
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, got {:?}", expected, tok),
            })
        }
    }

    /// Parse the full input into an AST.
    pub fn parse(&mut self) -> Result<TernaryExpr, ParseError> {
        let expr = self.parse_expr()?;
        Ok(expr)
    }

    fn parse_expr(&mut self) -> Result<TernaryExpr, ParseError> {
        match self.peek().clone() {
            Token::Room(_) => self.parse_room(),
            Token::Passage(_) => self.parse_passage(),
            Token::Gate(_) => self.parse_gate(),
            Token::LBrace => self.parse_block(),
            _ => self.parse_sequence(),
        }
    }

    fn parse_room(&mut self) -> Result<TernaryExpr, ParseError> {
        let name = match self.advance() {
            Token::Room(n) => n,
            t => return Err(ParseError { message: format!("Expected room, got {:?}", t) }),
        };
        self.expect(&Token::LBrace)?;
        let body = self.parse_expr()?;
        self.expect(&Token::RBrace)?;
        Ok(TernaryExpr::Room(RoomDef {
            name,
            body: Box::new(body),
        }))
    }

    fn parse_passage(&mut self) -> Result<TernaryExpr, ParseError> {
        let name = match self.advance() {
            Token::Passage(n) => n,
            t => return Err(ParseError { message: format!("Expected passage, got {:?}", t) }),
        };
        let from = match self.advance() {
            Token::Ident(n) => n,
            t => return Err(ParseError { message: format!("Expected ident, got {:?}", t) }),
        };
        self.expect(&Token::Arrow)?;
        let to = match self.advance() {
            Token::Ident(n) => n,
            t => return Err(ParseError { message: format!("Expected ident, got {:?}", t) }),
        };
        Ok(TernaryExpr::Passage(PassageDef { name, from, to }))
    }

    fn parse_gate(&mut self) -> Result<TernaryExpr, ParseError> {
        let name = match self.advance() {
            Token::Gate(n) => n,
            t => return Err(ParseError { message: format!("Expected gate, got {:?}", t) }),
        };
        self.expect(&Token::LParen)?;
        let condition = self.parse_expr()?;
        self.expect(&Token::Comma)?;
        let if_neg = self.parse_expr()?;
        self.expect(&Token::Comma)?;
        let if_zero = self.parse_expr()?;
        self.expect(&Token::Comma)?;
        let if_pos = self.parse_expr()?;
        self.expect(&Token::RParen)?;
        Ok(TernaryExpr::Gate(GateDef {
            name,
            condition: Box::new(condition),
            if_neg: Box::new(if_neg),
            if_zero: Box::new(if_zero),
            if_pos: Box::new(if_pos),
        }))
    }

    fn parse_block(&mut self) -> Result<TernaryExpr, ParseError> {
        self.expect(&Token::LBrace)?;
        let mut exprs = Vec::new();
        while *self.peek() != Token::RBrace && *self.peek() != Token::Eof {
            exprs.push(self.parse_expr()?);
            if *self.peek() == Token::Semicolon {
                self.advance();
            }
        }
        self.expect(&Token::RBrace)?;
        Ok(TernaryExpr::Block(Block { exprs }))
    }

    fn parse_sequence(&mut self) -> Result<TernaryExpr, ParseError> {
        let mut left = self.parse_parallel()?;
        while *self.peek() == Token::Sequence {
            self.advance();
            let right = self.parse_parallel()?;
            left = TernaryExpr::Sequence(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_parallel(&mut self) -> Result<TernaryExpr, ParseError> {
        let mut left = self.parse_add()?;
        while *self.peek() == Token::Parallel {
            self.advance();
            let right = self.parse_add()?;
            left = TernaryExpr::Parallel(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_add(&mut self) -> Result<TernaryExpr, ParseError> {
        let mut left = self.parse_mul()?;
        loop {
            match self.peek() {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_mul()?;
                    left = TernaryExpr::Add(Box::new(left), Box::new(right));
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_mul()?;
                    left = TernaryExpr::Add(Box::new(left), Box::new(TernaryExpr::Negate(Box::new(right))));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<TernaryExpr, ParseError> {
        let mut left = self.parse_unary()?;
        while *self.peek() == Token::Star {
            self.advance();
            let right = self.parse_unary()?;
            left = TernaryExpr::Mul(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<TernaryExpr, ParseError> {
        if *self.peek() == Token::Bang {
            self.advance();
            let expr = self.parse_primary()?;
            Ok(TernaryExpr::Negate(Box::new(expr)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<TernaryExpr, ParseError> {
        match self.peek().clone() {
            Token::Negative => {
                self.advance();
                Ok(TernaryExpr::Lit(Ternary::Neg))
            }
            Token::Zero => {
                self.advance();
                Ok(TernaryExpr::Lit(Ternary::Zero))
            }
            Token::Positive => {
                self.advance();
                Ok(TernaryExpr::Lit(Ternary::Pos))
            }
            Token::Ident(name) => {
                self.advance();
                Ok(TernaryExpr::Var(name))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Token::Branch => {
                self.advance();
                let left = self.parse_primary()?;
                self.expect(&Token::Comma)?;
                let right = self.parse_primary()?;
                Ok(TernaryExpr::Branch(
                    Box::new(TernaryExpr::Lit(Ternary::Pos)),
                    Box::new(left),
                    Box::new(right),
                ))
            }
            t => Err(ParseError {
                message: format!("Unexpected token: {:?}", t),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Result<TernaryExpr, ParseError> {
        let mut lexer = crate::lexer::Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_parse_literal_neg() {
        let expr = parse("neg").unwrap();
        assert_eq!(expr, TernaryExpr::Lit(Ternary::Neg));
    }

    #[test]
    fn test_parse_literal_zero() {
        let expr = parse("zero").unwrap();
        assert_eq!(expr, TernaryExpr::Lit(Ternary::Zero));
    }

    #[test]
    fn test_parse_literal_pos() {
        let expr = parse("pos").unwrap();
        assert_eq!(expr, TernaryExpr::Lit(Ternary::Pos));
    }

    #[test]
    fn test_parse_addition() {
        let expr = parse("neg + pos").unwrap();
        assert_eq!(expr, TernaryExpr::Add(
            Box::new(TernaryExpr::Lit(Ternary::Neg)),
            Box::new(TernaryExpr::Lit(Ternary::Pos)),
        ));
    }

    #[test]
    fn test_parse_multiplication() {
        let expr = parse("neg * pos").unwrap();
        assert_eq!(expr, TernaryExpr::Mul(
            Box::new(TernaryExpr::Lit(Ternary::Neg)),
            Box::new(TernaryExpr::Lit(Ternary::Pos)),
        ));
    }

    #[test]
    fn test_parse_negation() {
        let expr = parse("!pos").unwrap();
        assert_eq!(expr, TernaryExpr::Negate(Box::new(TernaryExpr::Lit(Ternary::Pos))));
    }

    #[test]
    fn test_parse_sequence() {
        let expr = parse("neg >> pos").unwrap();
        assert_eq!(expr, TernaryExpr::Sequence(
            Box::new(TernaryExpr::Lit(Ternary::Neg)),
            Box::new(TernaryExpr::Lit(Ternary::Pos)),
        ));
    }

    #[test]
    fn test_parse_room() {
        let expr = parse("room start { pos }").unwrap();
        match expr {
            TernaryExpr::Room(r) => {
                assert_eq!(r.name, "start");
                assert_eq!(*r.body, TernaryExpr::Lit(Ternary::Pos));
            }
            _ => panic!("Expected Room"),
        }
    }

    #[test]
    fn test_parse_passage() {
        let expr = parse("passage north start -> hall").unwrap();
        match expr {
            TernaryExpr::Passage(p) => {
                assert_eq!(p.name, "north");
                assert_eq!(p.from, "start");
                assert_eq!(p.to, "hall");
            }
            _ => panic!("Expected Passage"),
        }
    }
}
