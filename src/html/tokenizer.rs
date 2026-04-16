use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Token {
    StartTag {
        name: String,
        attributes: HashMap<String, String>,
    },
    EndTag {
        name: String,
    },
    SelfClosingTag {
        name: String,
        attributes: HashMap<String, String>,
    },
    Text {
        content: String,
    },
    Comment {
        content: String,
    },
}

pub struct Tokenizer {
    input: Vec<char>,
    pos: usize,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Tokenizer {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while self.pos < self.input.len() {
            if let Some(token) = self.next_token() {
                tokens.push(token);
            }
        }
        tokens
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn starts_with(&self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        if self.pos + chars.len() > self.input.len() {
            return false;
        }
        for (i, c) in chars.iter().enumerate() {
            if self.input[self.pos + i] != *c {
                return false;
            }
        }
        true
    }

    fn starts_with_ci(&self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        if self.pos + chars.len() > self.input.len() {
            return false;
        }
        for (i, c) in chars.iter().enumerate() {
            if self.input[self.pos + i].to_ascii_lowercase() != c.to_ascii_lowercase() {
                return false;
            }
        }
        true
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

    fn next_token(&mut self) -> Option<Token> {
        if self.pos >= self.input.len() {
            return None;
        }

        if self.peek() == Some('<') {
            // Check for comment
            if self.starts_with("<!--") {
                return Some(self.read_comment());
            }
            // Check for DOCTYPE
            if self.starts_with_ci("<!doctype") {
                self.skip_doctype();
                return self.next_token();
            }
            // Check for end tag
            if self.starts_with("</") {
                return Some(self.read_end_tag());
            }
            // Start tag or self-closing tag
            return Some(self.read_start_tag());
        }

        // Text content
        Some(self.read_text())
    }

    fn read_comment(&mut self) -> Token {
        // Skip "<!--"
        self.pos += 4;
        let mut content = String::new();
        while self.pos < self.input.len() {
            if self.starts_with("-->") {
                self.pos += 3;
                return Token::Comment { content };
            }
            content.push(self.input[self.pos]);
            self.pos += 1;
        }
        // Unterminated comment - return what we have
        Token::Comment { content }
    }

    fn skip_doctype(&mut self) {
        // Skip everything until '>'
        while let Some(ch) = self.advance() {
            if ch == '>' {
                return;
            }
        }
    }

    fn read_end_tag(&mut self) -> Token {
        // Skip "</"
        self.pos += 2;
        let name = self.read_while(|c| c != '>').to_ascii_lowercase();
        // Skip ">"
        self.advance();
        Token::EndTag { name: name.trim().to_string() }
    }

    fn read_start_tag(&mut self) -> Token {
        // Skip "<"
        self.pos += 1;
        let name = self.read_while(|c| !c.is_whitespace() && c != '>' && c != '/').to_ascii_lowercase();
        let mut attributes = HashMap::new();

        // Parse attributes
        loop {
            // Skip whitespace
            self.read_while(|c| c.is_whitespace());

            match self.peek() {
                Some('>') => {
                    self.advance();
                    return Token::StartTag { name, attributes };
                }
                Some('/') => {
                    self.advance();
                    // Expect '>'
                    if self.peek() == Some('>') {
                        self.advance();
                    }
                    return Token::SelfClosingTag { name, attributes };
                }
                None => {
                    return Token::StartTag { name, attributes };
                }
                _ => {
                    // Read attribute
                    let attr_name = self.read_while(|c| !c.is_whitespace() && c != '=' && c != '>' && c != '/').to_ascii_lowercase();
                    if attr_name.is_empty() {
                        // Safety: advance to avoid infinite loop
                        self.advance();
                        continue;
                    }

                    // Skip whitespace
                    self.read_while(|c| c.is_whitespace());

                    if self.peek() == Some('=') {
                        self.advance(); // skip '='
                        // Skip whitespace
                        self.read_while(|c| c.is_whitespace());

                        let value = if self.peek() == Some('"') || self.peek() == Some('\'') {
                            let quote = self.advance().unwrap();
                            let val = self.read_while(move |c| c != quote);
                            self.advance(); // skip closing quote
                            val
                        } else {
                            // Unquoted value
                            self.read_while(|c| !c.is_whitespace() && c != '>')
                        };
                        attributes.insert(attr_name, value);
                    } else {
                        // Boolean attribute
                        attributes.insert(attr_name, String::new());
                    }
                }
            }
        }
    }

    fn read_text(&mut self) -> Token {
        let content = self.read_while(|c| c != '<');
        Token::Text { content }
    }
}
