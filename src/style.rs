use indexmap::IndexMap;
use serde::Serialize;
use std::collections::HashMap;

use crate::css::parser::{Declaration, Selector, Stylesheet};
use crate::html::parser::Node;

#[derive(Debug, Clone, Serialize)]
pub struct StyledNode {
    pub node: NodeInfo,
    pub styles: IndexMap<String, String>,
    pub children: Vec<StyledNode>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum NodeInfo {
    Element {
        tag: String,
        attributes: HashMap<String, String>,
    },
    Text {
        content: String,
    },
}

const PROPERTY_ORDER: &[&str] = &[
    "display",
    "color",
    "background-color",
    "width",
    "height",
    "margin-top",
    "margin-right",
    "margin-bottom",
    "margin-left",
    "padding-top",
    "padding-right",
    "padding-bottom",
    "padding-left",
    "border-width",
    "border-color",
    "font-size",
];

fn default_display(tag: &str) -> &'static str {
    match tag {
        "span" | "a" | "strong" | "em" | "b" | "i" | "u" => "inline",
        _ => "block",
    }
}

fn default_value(property: &str, tag: &str) -> String {
    match property {
        "display" => default_display(tag).to_string(),
        "color" => "black".to_string(),
        "background-color" => "transparent".to_string(),
        "width" | "height" => "auto".to_string(),
        "margin-top" | "margin-right" | "margin-bottom" | "margin-left" => "0px".to_string(),
        "padding-top" | "padding-right" | "padding-bottom" | "padding-left" => "0px".to_string(),
        "border-width" => "0px".to_string(),
        "border-color" => "black".to_string(),
        "font-size" => "16px".to_string(),
        _ => String::new(),
    }
}

fn is_inherited(property: &str) -> bool {
    matches!(property, "color" | "font-size")
}

fn specificity(selector: &Selector) -> u32 {
    let mut score = 0u32;
    if let Some(ref id) = selector.id {
        let _ = id;
        score += 100;
    }
    score += selector.classes.len() as u32 * 10;
    if let Some(ref tag) = selector.tag {
        if tag != "*" {
            score += 1;
        }
    }
    score
}

fn selector_matches(
    selector: &Selector,
    tag: &str,
    id: Option<&str>,
    classes: &[&str],
) -> bool {
    // Check tag
    if let Some(ref sel_tag) = selector.tag {
        if sel_tag != "*" && sel_tag != tag {
            return false;
        }
    }
    // Check id
    if let Some(ref sel_id) = selector.id {
        match id {
            Some(elem_id) if elem_id == sel_id => {}
            _ => return false,
        }
    }
    // Check classes
    for cls in &selector.classes {
        if !classes.contains(&cls.as_str()) {
            return false;
        }
    }
    true
}

/// Expand shorthand properties (margin, padding) into longhand declarations
fn expand_declarations(declarations: &[Declaration]) -> Vec<Declaration> {
    let mut result = Vec::new();
    for decl in declarations {
        match decl.property.as_str() {
            "margin" | "padding" => {
                let prefix = &decl.property;
                let parts: Vec<&str> = decl.value.split_whitespace().collect();
                let (top, right, bottom, left) = match parts.len() {
                    1 => (parts[0], parts[0], parts[0], parts[0]),
                    2 => (parts[0], parts[1], parts[0], parts[1]),
                    3 => (parts[0], parts[1], parts[2], parts[1]),
                    4 => (parts[0], parts[1], parts[2], parts[3]),
                    _ => (parts[0], parts[0], parts[0], parts[0]),
                };
                result.push(Declaration {
                    property: format!("{}-top", prefix),
                    value: top.to_string(),
                });
                result.push(Declaration {
                    property: format!("{}-right", prefix),
                    value: right.to_string(),
                });
                result.push(Declaration {
                    property: format!("{}-bottom", prefix),
                    value: bottom.to_string(),
                });
                result.push(Declaration {
                    property: format!("{}-left", prefix),
                    value: left.to_string(),
                });
            }
            _ => {
                result.push(decl.clone());
            }
        }
    }
    result
}

pub fn resolve_styles(
    node: &Node,
    stylesheet: &Stylesheet,
    parent_styles: Option<&IndexMap<String, String>>,
) -> Option<StyledNode> {
    match node {
        Node::Element {
            tag,
            attributes,
            children,
        } => {
            let elem_id = attributes.get("id").map(|s| s.as_str());
            let elem_classes: Vec<&str> = attributes
                .get("class")
                .map(|c| c.split_whitespace().collect())
                .unwrap_or_default();

            // Collect matching declarations with specificity and source order
            let mut matched: Vec<(String, String, u32, usize)> = Vec::new(); // (property, value, specificity, order)
            let mut order = 0usize;

            for rule in &stylesheet.rules {
                let mut rule_matches = false;
                let mut best_spec = 0u32;

                for selector in &rule.selectors {
                    if selector_matches(selector, tag, elem_id, &elem_classes) {
                        rule_matches = true;
                        let spec = specificity(selector);
                        if spec > best_spec {
                            best_spec = spec;
                        }
                    }
                }

                if rule_matches {
                    let expanded = expand_declarations(&rule.declarations);
                    for decl in &expanded {
                        matched.push((
                            decl.property.clone(),
                            decl.value.clone(),
                            best_spec,
                            order,
                        ));
                        order += 1;
                    }
                }
            }

            // Resolve: for each property, find the winning value
            let mut resolved: HashMap<String, String> = HashMap::new();
            for prop in PROPERTY_ORDER {
                // Find all matched declarations for this property
                let mut best_value: Option<(String, u32, usize)> = None;
                for (p, v, spec, ord) in &matched {
                    if p == prop {
                        match &best_value {
                            None => {
                                best_value = Some((v.clone(), *spec, *ord));
                            }
                            Some((_, best_spec, best_ord)) => {
                                if *spec > *best_spec
                                    || (*spec == *best_spec && *ord >= *best_ord)
                                {
                                    best_value = Some((v.clone(), *spec, *ord));
                                }
                            }
                        }
                    }
                }

                if let Some((value, _, _)) = best_value {
                    resolved.insert(prop.to_string(), value);
                } else if is_inherited(prop) {
                    // Inherit from parent
                    if let Some(parent) = parent_styles {
                        if let Some(parent_value) = parent.get(*prop) {
                            resolved.insert(prop.to_string(), parent_value.clone());
                        } else {
                            resolved.insert(prop.to_string(), default_value(prop, tag));
                        }
                    } else {
                        // Root element
                        resolved.insert(prop.to_string(), default_value(prop, tag));
                    }
                } else {
                    resolved.insert(prop.to_string(), default_value(prop, tag));
                }
            }

            // Build ordered styles map
            let mut styles = IndexMap::new();
            for prop in PROPERTY_ORDER {
                if let Some(value) = resolved.get(*prop) {
                    styles.insert(prop.to_string(), value.clone());
                }
            }

            // Resolve children
            let is_style_element = tag == "style";
            let styled_children: Vec<StyledNode> = if is_style_element {
                Vec::new() // Style elements have no styled children
            } else {
                children
                    .iter()
                    .filter_map(|child| resolve_styles(child, stylesheet, Some(&styles)))
                    .collect()
            };

            Some(StyledNode {
                node: NodeInfo::Element {
                    tag: tag.clone(),
                    attributes: attributes.clone(),
                },
                styles,
                children: styled_children,
            })
        }
        Node::Text { content } => {
            // Drop whitespace-only text nodes
            if content.trim().is_empty() {
                return None;
            }
            Some(StyledNode {
                node: NodeInfo::Text {
                    content: content.clone(),
                },
                styles: IndexMap::new(),
                children: Vec::new(),
            })
        }
        Node::Comment { .. } => {
            // Comments are excluded from styled tree
            None
        }
        Node::Document { children } => {
            // Shouldn't normally happen for well-formed HTML with a single root
            let styled_children: Vec<StyledNode> = children
                .iter()
                .filter_map(|child| resolve_styles(child, stylesheet, parent_styles))
                .collect();
            if styled_children.len() == 1 {
                Some(styled_children.into_iter().next().unwrap())
            } else {
                // Just return the first child or handle as needed
                styled_children.into_iter().next()
            }
        }
    }
}

/// Extract CSS text from <style> elements in the DOM tree
pub fn extract_css(node: &Node) -> String {
    let mut css = String::new();
    match node {
        Node::Element {
            tag, children, ..
        } => {
            if tag == "style" {
                for child in children {
                    if let Node::Text { content } = child {
                        css.push_str(content);
                        css.push('\n');
                    }
                }
            } else {
                for child in children {
                    css.push_str(&extract_css(child));
                }
            }
        }
        Node::Document { children } => {
            for child in children {
                css.push_str(&extract_css(child));
            }
        }
        _ => {}
    }
    css
}
