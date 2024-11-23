// src/parser.rs

use crate::ast::{Insert, Query, Value};

// Define the Token enum representing different types of tokens
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    StringLiteral(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
    Date(String),
    Time(String),
    Timestamp(String),
    Interval(String),
    LeftParen,
    RightParen,
    Comma,
    SemiColon,
    Whitespace(String),
    // Add other tokens as needed
}

// Lexer struct responsible for tokenizing the input string
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,      // Current position in input (points to current char)
    read_position: usize, // Current reading position in input (after current char)
    ch: Option<char>,     // Current char under examination
}

impl<'a> Lexer<'a> {
    /// Creates a new Lexer instance and initializes the first character.
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input,
            position: 0,
            read_position: 0,
            ch: None,
        };
        lexer.read_char(); // Initialize the first character
        lexer
    }

    /// Reads the next character and advances positions.
    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = None; // End of input
        } else {
            self.ch = Some(self.input[self.read_position..].chars().next().unwrap());
        }
        self.position = self.read_position;
        if let Some(c) = self.ch {
            self.read_position += c.len_utf8();
        }
    }

    /// Skips over any whitespace characters.
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.ch {
            if !c.is_whitespace() {
                break;
            }
            self.read_char();
        }
    }

    /// Reads a string literal enclosed in single quotes.
    fn read_string_literal(&mut self) -> String {
        self.read_char(); // Consume the opening quote
        let mut literal = String::new();
        while let Some(c) = self.ch {
            if c == '\'' {
                break;
            }
            literal.push(c);
            self.read_char();
        }
        self.read_char(); // Consume the closing quote
        literal
    }

    /// Reads a numeric literal (integer or float).
    fn read_number(&mut self) -> Token {
        let mut number = String::new();
        while let Some(c) = self.ch {
            if !c.is_ascii_digit() && c != '.' {
                break;
            }
            number.push(c);
            self.read_char();
        }
        if number.contains('.') {
            if let Ok(f) = number.parse::<f64>() {
                Token::Float(f)
            } else {
                // Handle parse error if needed
                Token::Float(0.0)
            }
        } else {
            if let Ok(i) = number.parse::<i64>() {
                Token::Integer(i)
            } else {
                // Handle parse error if needed
                Token::Integer(0)
            }
        }
    }

    /// Reads an identifier or keyword.
    fn read_identifier_or_keyword(&mut self) -> Token {
        let mut ident = String::new();
        while let Some(c) = self.ch {
            if !Self::is_identifier_part(c) {
                break;
            }
            ident.push(c);
            self.read_char();
        }
        if Self::is_keyword(&ident) {
            match ident.to_uppercase().as_str() {
                "INSERT" => Token::Keyword(ident.to_uppercase()),
                "INTO" => Token::Keyword(ident.to_uppercase()),
                "VALUES" => Token::Keyword(ident.to_uppercase()),
                "NULL" => Token::Null,
                "TRUE" => Token::Boolean(true),
                "FALSE" => Token::Boolean(false),
                "DATE" => Token::Keyword(ident.to_uppercase()),
                "TIME" => Token::Keyword(ident.to_uppercase()),
                "TIMESTAMP" => Token::Keyword(ident.to_uppercase()),
                "INTERVAL" => Token::Keyword(ident.to_uppercase()),
                _ => Token::Keyword(ident.to_uppercase()),
            }
        } else {
            Token::Identifier(ident)
        }
    }

    /// Checks if a character can start an identifier.
    fn is_identifier_start(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    /// Checks if a character can be part of an identifier.
    fn is_identifier_part(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    /// Checks if a string is a SQL keyword.
    fn is_keyword(ident: &str) -> bool {
        matches!(
            ident.to_uppercase().as_str(),
            "INSERT"
                | "INTO"
                | "VALUES"
                | "DATE"
                | "TIME"
                | "TIMESTAMP"
                | "INTERVAL"
                | "NULL"
                | "TRUE"
                | "FALSE"
        )
    }

    /// Returns the next token from the input.
    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        let token = match self.ch {
            Some('(') => {
                self.read_char();
                Token::LeftParen
            }
            Some(')') => {
                self.read_char();
                Token::RightParen
            }
            Some(',') => {
                self.read_char();
                Token::Comma
            }
            Some(';') => {
                self.read_char();
                Token::SemiColon
            }
            Some('\'') => Token::StringLiteral(self.read_string_literal()),
            Some(c) if c.is_ascii_digit() => self.read_number(),
            Some(c) if Self::is_identifier_start(c) => self.read_identifier_or_keyword(),
            Some(_) => {
                // Handle unknown characters
                self.read_char();
                return None;
            }
            None => {
                // End of input
                return None;
            }
        };
        Some(token)
    }
}

// Parser struct responsible for parsing tokens into an abstract syntax tree
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Option<Token>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser instance.
    pub fn new(input: &'a str) -> Result<Self, String> {
        let mut lexer = Lexer::new(input);
        let first_token = lexer.next_token();
        Ok(Parser {
            lexer,
            current_token: first_token,
        })
    }

    /// Advances to the next token.
    fn next_token(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    /// Matches and consumes the current token if it matches the expected token.
    fn match_token(&mut self, token: &Token) -> bool {
        if let Some(ref current) = self.current_token {
            if current == token {
                self.next_token();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Matches and consumes a keyword.
    fn match_keyword(&mut self, keyword: &str) -> bool {
        if let Some(Token::Keyword(ref kw)) = self.current_token {
            if kw.eq_ignore_ascii_case(keyword) {
                self.next_token();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Parses the entire query.
    pub fn parse(&mut self) -> Result<Query, String> {
        match self.current_token {
            Some(Token::Keyword(ref kw)) if kw.eq_ignore_ascii_case("INSERT") => {
                self.parse_insert()
            }
            _ => Err("Unsupported query type.".to_string()),
        }
    }

    /// Parses an INSERT statement.
    fn parse_insert(&mut self) -> Result<Query, String> {
        // Consume 'INSERT'
        if !self.match_keyword("INSERT") {
            return Err("Expected 'INSERT' keyword.".to_string());
        }

        // Consume 'INTO'
        if !self.match_keyword("INTO") {
            return Err("Expected 'INTO' keyword.".to_string());
        }

        // Parse table name
        let table = if let Some(Token::Identifier(ref name)) = self.current_token {
            let table_name = name.clone();
            self.next_token();
            table_name
        } else {
            return Err("Expected table name.".to_string());
        };

        // Consume '('
        if !self.match_token(&Token::LeftParen) {
            return Err("Expected '('.".to_string());
        }

        // Parse column names
        let mut columns = Vec::new();
        loop {
            if let Some(Token::Identifier(ref col)) = self.current_token {
                columns.push(col.clone());
                self.next_token();
            } else {
                return Err("Expected column name.".to_string());
            }

            if self.match_token(&Token::Comma) {
                continue;
            } else {
                break;
            }
        }

        // Consume ')'
        if !self.match_token(&Token::RightParen) {
            return Err("Expected ')'.".to_string());
        }

        // Consume 'VALUES'
        if !self.match_keyword("VALUES") {
            return Err("Expected 'VALUES' keyword.".to_string());
        }

        // Consume '('
        if !self.match_token(&Token::LeftParen) {
            return Err("Expected '('.".to_string());
        }

        // Parse values
        let mut values = Vec::new();
        loop {
            self.consume_whitespace_and_comments();

            let value = match self.current_token.clone() {
                Some(Token::Integer(i)) => {
                    self.next_token();
                    Value::Integer(i)
                }
                Some(Token::Float(f)) => {
                    self.next_token();
                    Value::Float(f)
                }
                Some(Token::StringLiteral(s)) => {
                    self.next_token();
                    Value::Text(s)
                }
                Some(Token::Null) => {
                    self.next_token();
                    Value::Null
                }
                Some(Token::Boolean(b)) => {
                    self.next_token();
                    Value::Boolean(b)
                }
                Some(Token::Keyword(ref kw)) if kw.eq_ignore_ascii_case("DATE") => {
                    self.next_token();
                    if let Some(Token::StringLiteral(s)) = self.current_token.clone() {
                        self.next_token();
                        Value::Date(s)
                    } else {
                        return Err("Failed to parse 'DATE' literal.".to_string());
                    }
                }
                Some(Token::Keyword(ref kw)) if kw.eq_ignore_ascii_case("TIME") => {
                    self.next_token();
                    if let Some(Token::StringLiteral(s)) = self.current_token.clone() {
                        self.next_token();
                        Value::Time(s)
                    } else {
                        return Err("Failed to parse 'TIME' literal.".to_string());
                    }
                }
                Some(Token::Keyword(ref kw)) if kw.eq_ignore_ascii_case("TIMESTAMP") => {
                    self.next_token();
                    if let Some(Token::StringLiteral(s)) = self.current_token.clone() {
                        self.next_token();
                        Value::Timestamp(s)
                    } else {
                        return Err("Failed to parse 'TIMESTAMP' literal.".to_string());
                    }
                }
                Some(Token::Keyword(ref kw)) if kw.eq_ignore_ascii_case("INTERVAL") => {
                    self.next_token();
                    if let Some(Token::StringLiteral(s)) = self.current_token.clone() {
                        self.next_token();
                        Value::Interval(s)
                    } else {
                        return Err("Failed to parse 'INTERVAL' literal.".to_string());
                    }
                }
                _ => return Err("Failed to parse value.".to_string()),
            };

            values.push(value);

            self.consume_whitespace_and_comments();

            if self.match_token(&Token::Comma) {
                continue;
            } else {
                break;
            }
        }

        // Consume ')'
        if !self.match_token(&Token::RightParen) {
            return Err("Expected ')'.".to_string());
        }

        // Consume optional ';'
        self.match_token(&Token::SemiColon);

        Ok(Query::Insert(Insert {
            table,
            columns,
            values,
        }))
    }

    /// Consumes any whitespace and comments.
    fn consume_whitespace_and_comments(&mut self) {
        while let Some(Token::Whitespace(_)) = self.current_token {
            self.next_token();
        }
        // Add comment handling if necessary
    }
}
