use indexmap::IndexMap;
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

use crate::style::{NodeInfo, StyledNode};

#[derive(Debug, Clone)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Serialize for Rect {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("Rect", 4)?;
        s.serialize_field("x", &F1(self.x))?;
        s.serialize_field("y", &F1(self.y))?;
        s.serialize_field("width", &F1(self.width))?;
        s.serialize_field("height", &F1(self.height))?;
        s.end()
    }
}

#[derive(Debug, Clone)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Serialize for Edges {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("Edges", 4)?;
        s.serialize_field("top", &F1(self.top))?;
        s.serialize_field("right", &F1(self.right))?;
        s.serialize_field("bottom", &F1(self.bottom))?;
        s.serialize_field("left", &F1(self.left))?;
        s.end()
    }
}

/// Wrapper to serialize f32 with exactly one decimal place
struct F1(f32);

impl Serialize for F1 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let rounded = (self.0 * 10.0).round() / 10.0;
        serializer.serialize_f64(rounded as f64)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum LayoutBox {
    Block {
        tag: String,
        #[serde(rename = "box")]
        box_dims: Rect,
        padding: Edges,
        border: Edges,
        margin: Edges,
        children: Vec<LayoutBox>,
    },
    Text {
        content: String,
        #[serde(rename = "box")]
        box_dims: Rect,
    },
}

fn parse_px(value: &str) -> f32 {
    if value == "auto" || value == "0" {
        return 0.0;
    }
    value
        .strip_suffix("px")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0)
}

fn get_style(styles: &IndexMap<String, String>, prop: &str) -> String {
    styles.get(prop).cloned().unwrap_or_default()
}

/// Find the <body> element in the styled tree, or use the root directly
pub fn find_body(styled: &StyledNode) -> Option<&StyledNode> {
    match &styled.node {
        NodeInfo::Element { tag, .. } => {
            if tag == "body" {
                return Some(styled);
            }
            if tag == "html" {
                for child in &styled.children {
                    if let Some(body) = find_body(child) {
                        return Some(body);
                    }
                }
            }
            // If not html/body, this might be the root element itself
            None
        }
        _ => None,
    }
}

pub fn layout(styled: &StyledNode, viewport_width: f32, _viewport_height: f32) -> Option<LayoutBox> {
    // Find <body> or use root
    let root = find_body(styled).unwrap_or(styled);

    let containing = Rect {
        x: 0.0,
        y: 0.0,
        width: viewport_width,
        height: 0.0, // height is bottom-up
    };

    layout_block(root, &containing, 0.0)
}

fn layout_block(styled: &StyledNode, containing: &Rect, cursor_y_start: f32) -> Option<LayoutBox> {
    match &styled.node {
        NodeInfo::Element { tag, .. } => {
            // Skip non-renderable elements
            if tag == "head" || tag == "style" {
                return None;
            }

            let styles = &styled.styles;
            let display = get_style(styles, "display");
            if display == "none" {
                return None;
            }

            // Parse box model values
            let margin = Edges {
                top: parse_px(&get_style(styles, "margin-top")),
                right: parse_px(&get_style(styles, "margin-right")),
                bottom: parse_px(&get_style(styles, "margin-bottom")),
                left: parse_px(&get_style(styles, "margin-left")),
            };

            let border_width = parse_px(&get_style(styles, "border-width"));
            let border = Edges {
                top: border_width,
                right: border_width,
                bottom: border_width,
                left: border_width,
            };

            let padding = Edges {
                top: parse_px(&get_style(styles, "padding-top")),
                right: parse_px(&get_style(styles, "padding-right")),
                bottom: parse_px(&get_style(styles, "padding-bottom")),
                left: parse_px(&get_style(styles, "padding-left")),
            };

            // 1. Calculate width
            let width_value = get_style(styles, "width");
            let content_width = if width_value != "auto" {
                parse_px(&width_value)
            } else {
                containing.width
                    - margin.left
                    - border.left
                    - padding.left
                    - padding.right
                    - border.right
                    - margin.right
            };

            // 2. Calculate position
            let content_x =
                containing.x + margin.left + border.left + padding.left;
            let content_y =
                cursor_y_start + margin.top + border.top + padding.top;

            // 3. Layout children
            let my_content_rect = Rect {
                x: content_x,
                y: content_y,
                width: content_width,
                height: 0.0, // will be calculated
            };

            let mut child_cursor_y = content_y;
            let mut layout_children = Vec::new();

            let font_size = parse_px(&get_style(styles, "font-size"));

            for child in &styled.children {
                match &child.node {
                    NodeInfo::Element { tag: child_tag, .. } => {
                        if child_tag == "head" || child_tag == "style" {
                            continue;
                        }
                        let child_display = get_style(&child.styles, "display");
                        if child_display == "none" {
                            continue;
                        }
                        if let Some(child_box) =
                            layout_block(child, &my_content_rect, child_cursor_y)
                        {
                            // Advance cursor past this child
                            match &child_box {
                                LayoutBox::Block {
                                    box_dims,
                                    padding: cp,
                                    border: cb,
                                    margin: cm,
                                    ..
                                } => {
                                    child_cursor_y = box_dims.y
                                        + box_dims.height
                                        + cp.bottom
                                        + cb.bottom
                                        + cm.bottom;
                                }
                                LayoutBox::Text { box_dims, .. } => {
                                    child_cursor_y =
                                        box_dims.y + box_dims.height;
                                }
                            }
                            layout_children.push(child_box);
                        }
                    }
                    NodeInfo::Text { content } => {
                        // Text node layout
                        if content.trim().is_empty() {
                            continue;
                        }
                        let text_box = LayoutBox::Text {
                            content: content.clone(),
                            box_dims: Rect {
                                x: content_x,
                                y: child_cursor_y,
                                width: content_width,
                                height: font_size,
                            },
                        };
                        child_cursor_y += font_size;
                        layout_children.push(text_box);
                    }
                }
            }

            // 4. Calculate height
            let height_value = get_style(styles, "height");
            let content_height = if height_value != "auto" {
                parse_px(&height_value)
            } else {
                child_cursor_y - content_y
            };

            Some(LayoutBox::Block {
                tag: tag.clone(),
                box_dims: Rect {
                    x: content_x,
                    y: content_y,
                    width: content_width,
                    height: content_height,
                },
                padding,
                border,
                margin,
                children: layout_children,
            })
        }
        NodeInfo::Text { content } => {
            if content.trim().is_empty() {
                return None;
            }
            Some(LayoutBox::Text {
                content: content.clone(),
                box_dims: Rect {
                    x: containing.x,
                    y: cursor_y_start,
                    width: containing.width,
                    height: 16.0, // default font-size
                },
            })
        }
    }
}
