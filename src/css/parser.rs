use serde::Serialize;
use super::tokenizer::CssToken;

#[derive(Debug, Clone, Serialize)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Selector {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Declaration {
    pub property: String,
    pub value: String,
}

pub struct CssParser {
    tokens: Vec<CssToken>,
    pos: usize,
}

impl CssParser {
    pub fn new(tokens: Vec<CssToken>) -> Self {
        CssParser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&CssToken> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&CssToken> {
        let token = self.tokens.get(self.pos);
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    pub fn parse_stylesheet(&mut self) -> Stylesheet {
        let mut rules = Vec::new();
        while !self.at_end() {
            if let Some(rule) = self.parse_rule() {
                rules.push(rule);
            }
        }
        Stylesheet { rules }
    }

    fn parse_rule(&mut self) -> Option<Rule> {
        let selectors = self.parse_selectors();

        // Expect and consume OpenBrace
        if matches!(self.peek(), Some(CssToken::OpenBrace)) {
            self.advance();
        }

        let declarations = self.parse_declarations();

        // Expect and consume CloseBrace
        if matches!(self.peek(), Some(CssToken::CloseBrace)) {
            self.advance();
        }

        Some(Rule {
            selectors,
            declarations,
        })
    }

    fn parse_selectors(&mut self) -> Vec<Selector> {
        let mut selectors = Vec::new();

        loop {
            if self.at_end() || matches!(self.peek(), Some(CssToken::OpenBrace)) {
                break;
            }

            let selector = self.parse_selector();
            selectors.push(selector);

            // Check for comma
            if matches!(self.peek(), Some(CssToken::Comma)) {
                self.advance(); // consume comma
            }
        }

        selectors
    }

    fn parse_selector(&mut self) -> Selector {
        let mut tag = None;
        let mut id = None;
        let mut classes = Vec::new();

        // First token might be a tag name, star, hash, or dot
        loop {
            match self.peek() {
                Some(CssToken::Ident { .. }) if tag.is_none() && classes.is_empty() && id.is_none() => {
                    if let Some(CssToken::Ident { value }) = self.advance().cloned() {
                        tag = Some(value);
                    }
                }
                Some(CssToken::Star) if tag.is_none() => {
                    self.advance();
                    tag = Some("*".to_string());
                }
                Some(CssToken::Hash { .. }) => {
                    if let Some(CssToken::Hash { value }) = self.advance().cloned() {
                        id = Some(value.to_ascii_lowercase());
                    }
                }
                Some(CssToken::Dot) => {
                    self.advance(); // consume dot
                    if let Some(CssToken::Ident { value }) = self.advance().cloned() {
                        classes.push(value);
                    }
                }
                _ => break,
            }
        }

        Selector { tag, id, classes }
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();

        loop {
            if self.at_end() || matches!(self.peek(), Some(CssToken::CloseBrace)) {
                break;
            }

            if let Some(decl) = self.parse_declaration() {
                declarations.push(decl);
            }
        }

        declarations
    }

    fn parse_declaration(&mut self) -> Option<Declaration> {
        // Property name (Ident)
        let property = match self.peek() {
            Some(CssToken::Ident { .. }) => {
                if let Some(CssToken::Ident { value }) = self.advance().cloned() {
                    value
                } else {
                    return None;
                }
            }
            _ => {
                // Skip unexpected token
                self.advance();
                return None;
            }
        };

        // Expect colon
        if matches!(self.peek(), Some(CssToken::Colon)) {
            self.advance();
        }

        // Read value tokens until semicolon or close brace
        let mut value_parts = Vec::new();
        loop {
            match self.peek() {
                None | Some(CssToken::Semicolon) | Some(CssToken::CloseBrace) => break,
                Some(token) => {
                    value_parts.push(token_to_string(token));
                    self.advance();
                }
            }
        }

        // Consume semicolon if present
        if matches!(self.peek(), Some(CssToken::Semicolon)) {
            self.advance();
        }

        let value = value_parts.join(" ");

        Some(Declaration { property, value })
    }
}

fn token_to_string(token: &CssToken) -> String {
    match token {
        CssToken::Ident { value } => value.clone(),
        CssToken::Hash { value } => format!("#{}", value),
        CssToken::Number { value } => value.clone(),
        CssToken::Dimension { value, unit } => format!("{}{}", value, unit),
        CssToken::Percentage { value } => format!("{}%", value),
        CssToken::String { value } => value.clone(),
        CssToken::Star => "*".to_string(),
        CssToken::Comma => ",".to_string(),
        _ => String::new(),
    }
}
