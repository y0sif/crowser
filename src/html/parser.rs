use serde::Serialize;
use std::collections::HashMap;
use super::tokenizer::Token;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Node {
    Element {
        tag: String,
        attributes: HashMap<String, String>,
        children: Vec<Node>,
    },
    Text {
        content: String,
    },
    Comment {
        content: String,
    },
    Document {
        children: Vec<Node>,
    },
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Node {
        // Stack of child lists - each entry collects children for an open element
        let mut stack: Vec<Vec<Node>> = vec![Vec::new()]; // root level
        // Parallel stack of (tag_name, attributes) for open elements
        let mut tag_stack: Vec<(String, HashMap<String, String>)> = Vec::new();

        for token in &self.tokens {
            match token {
                Token::StartTag { name, attributes } => {
                    stack.push(Vec::new());
                    tag_stack.push((name.clone(), attributes.clone()));
                }
                Token::EndTag { name: _ } => {
                    let children = stack.pop().unwrap_or_default();
                    let (tag, attributes) = tag_stack.pop().unwrap_or_default();
                    let node = Node::Element {
                        tag,
                        attributes,
                        children,
                    };
                    if let Some(parent) = stack.last_mut() {
                        parent.push(node);
                    }
                }
                Token::SelfClosingTag { name, attributes } => {
                    let node = Node::Element {
                        tag: name.clone(),
                        attributes: attributes.clone(),
                        children: Vec::new(),
                    };
                    if let Some(parent) = stack.last_mut() {
                        parent.push(node);
                    }
                }
                Token::Text { content } => {
                    let node = Node::Text {
                        content: content.clone(),
                    };
                    if let Some(parent) = stack.last_mut() {
                        parent.push(node);
                    }
                }
                Token::Comment { content } => {
                    let node = Node::Comment {
                        content: content.clone(),
                    };
                    if let Some(parent) = stack.last_mut() {
                        parent.push(node);
                    }
                }
            }
        }

        // Close any unclosed elements
        while tag_stack.len() > 0 {
            let children = stack.pop().unwrap_or_default();
            let (tag, attributes) = tag_stack.pop().unwrap_or_default();
            let node = Node::Element {
                tag,
                attributes,
                children,
            };
            if let Some(parent) = stack.last_mut() {
                parent.push(node);
            }
        }

        let root_nodes = stack.into_iter().next().unwrap_or_default();

        if root_nodes.len() == 1 {
            root_nodes.into_iter().next().unwrap()
        } else {
            Node::Document {
                children: root_nodes,
            }
        }
    }
}
