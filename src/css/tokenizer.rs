use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum CssToken {
    Ident { value: String },
    Hash { value: String },
    Dot,
    String { value: String },
    Number { value: String },
    Dimension { value: String, unit: String },
    Percentage { value: String },
    Colon,
    Semicolon,
    Comma,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    Star,
}

pub struct CssTokenizer {
    input: Vec<char>,
    pos: usize,
}

impl CssTokenizer {
    pub fn new(input: &str) -> Self {
        CssTokenizer {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<CssToken> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            tokens.push(token);
        }
        tokens
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.input.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while let Some(ch) = self.peek() {
                if ch.is_whitespace() {
                    self.advance();
                } else {
                    break;
                }
            }
            // Skip comments
            if self.peek() == Some('/') && self.peek_at(1) == Some('*') {
                self.pos += 2;
                while self.pos < self.input.len() {
                    if self.peek() == Some('*') && self.peek_at(1) == Some('/') {
                        self.pos += 2;
                        break;
                    }
                    self.pos += 1;
                }
                continue; // Check for more whitespace/comments
            }
            break;
        }
    }

    fn read_while(&mut self, pred: impl Fn(char) -> bool) -> String {
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if pred(ch) {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    fn is_ident_start(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '-' || c == '_'
    }

    fn is_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '-' || c == '_'
    }

    fn read_ident(&mut self) -> String {
        self.read_while(Self::is_ident_char).to_ascii_lowercase()
    }

    fn read_hash_value(&mut self) -> String {
        // Read ident chars but also allow digits at the start (for hex colors)
        self.read_while(Self::is_ident_char)
    }

    fn read_number(&mut self) -> String {
        let mut result = String::new();
        // Optional leading minus
        if self.peek() == Some('-') {
            result.push('-');
            self.advance();
        }
        // Digits before decimal point
        result.push_str(&self.read_while(|c| c.is_ascii_digit()));
        // Decimal point and digits after
        if self.peek() == Some('.') && self.peek_at(1).map_or(false, |c| c.is_ascii_digit()) {
            result.push('.');
            self.advance();
            result.push_str(&self.read_while(|c| c.is_ascii_digit()));
        }
        result
    }

    fn read_string(&mut self, quote: char) -> String {
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if ch == quote {
                self.advance();
                return result;
            }
            result.push(ch);
            self.advance();
        }
        result // unterminated string
    }

    fn next_token(&mut self) -> Option<CssToken> {
        self.skip_whitespace_and_comments();

        let ch = self.peek()?;

        match ch {
            '{' => { self.advance(); Some(CssToken::OpenBrace) }
            '}' => { self.advance(); Some(CssToken::CloseBrace) }
            '(' => { self.advance(); Some(CssToken::OpenParen) }
            ')' => { self.advance(); Some(CssToken::CloseParen) }
            ':' => { self.advance(); Some(CssToken::Colon) }
            ';' => { self.advance(); Some(CssToken::Semicolon) }
            ',' => { self.advance(); Some(CssToken::Comma) }
            '*' => { self.advance(); Some(CssToken::Star) }
            '#' => {
                self.advance(); // consume #
                let value = self.read_hash_value();
                Some(CssToken::Hash { value })
            }
            '.' => {
                // Check if this is a decimal number like .5
                if self.peek_at(1).map_or(false, |c| c.is_ascii_digit()) {
                    let num = self.read_number();
                    self.finish_number(num)
                } else {
                    self.advance();
                    Some(CssToken::Dot)
                }
            }
            '"' | '\'' => {
                let quote = ch;
                self.advance(); // consume opening quote
                let value = self.read_string(quote);
                Some(CssToken::String { value })
            }
            '-' if self.peek_at(1).map_or(false, |c| c.is_ascii_digit()) => {
                let num = self.read_number();
                self.finish_number(num)
            }
            '-' if self.peek_at(1) == Some('.') && self.peek_at(2).map_or(false, |c| c.is_ascii_digit()) => {
                let num = self.read_number();
                self.finish_number(num)
            }
            c if c.is_ascii_digit() => {
                let num = self.read_number();
                self.finish_number(num)
            }
            c if Self::is_ident_start(c) => {
                let value = self.read_ident();
                Some(CssToken::Ident { value })
            }
            _ => {
                // Unknown character, skip
                self.advance();
                self.next_token()
            }
        }
    }

    fn finish_number(&mut self, num: String) -> Option<CssToken> {
        // Check what follows the number
        if self.peek() == Some('%') {
            self.advance();
            Some(CssToken::Percentage { value: num })
        } else if self.peek().map_or(false, |c| c.is_ascii_alphabetic()) {
            let unit = self.read_ident();
            Some(CssToken::Dimension { value: num, unit })
        } else {
            Some(CssToken::Number { value: num })
        }
    }
}
