use image::{ImageBuffer, Rgba};

use crate::layout::LayoutBox;
use crate::style::{NodeInfo, StyledNode};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub struct Canvas {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            pixels: vec![255; (width * height * 4) as usize],
            width,
            height,
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height || color.a == 0 {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        self.pixels[idx] = color.r;
        self.pixels[idx + 1] = color.g;
        self.pixels[idx + 2] = color.b;
        self.pixels[idx + 3] = color.a;
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        if color.a == 0 {
            return;
        }
        let x0 = x.round().max(0.0) as u32;
        let y0 = y.round().max(0.0) as u32;
        let x1 = (x + width).round().min(self.width as f32) as u32;
        let y1 = (y + height).round().min(self.height as f32) as u32;

        for py in y0..y1 {
            for px in x0..x1 {
                self.set_pixel(px, py, color);
            }
        }
    }

    pub fn save_png(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let img = ImageBuffer::from_fn(self.width, self.height, |x, y| {
            let idx = ((y * self.width + x) * 4) as usize;
            Rgba([
                self.pixels[idx],
                self.pixels[idx + 1],
                self.pixels[idx + 2],
                self.pixels[idx + 3],
            ])
        });
        img.save(path)?;
        Ok(())
    }
}

fn parse_color(value: &str) -> Option<Color> {
    let value = value.trim().to_ascii_lowercase();
    match value.as_str() {
        "black" => Some(Color { r: 0, g: 0, b: 0, a: 255 }),
        "white" => Some(Color { r: 255, g: 255, b: 255, a: 255 }),
        "red" => Some(Color { r: 255, g: 0, b: 0, a: 255 }),
        "green" => Some(Color { r: 0, g: 128, b: 0, a: 255 }),
        "blue" => Some(Color { r: 0, g: 0, b: 255, a: 255 }),
        "yellow" => Some(Color { r: 255, g: 255, b: 0, a: 255 }),
        "orange" => Some(Color { r: 255, g: 165, b: 0, a: 255 }),
        "purple" => Some(Color { r: 128, g: 0, b: 128, a: 255 }),
        "gray" | "grey" => Some(Color { r: 128, g: 128, b: 128, a: 255 }),
        "transparent" => Some(Color { r: 0, g: 0, b: 0, a: 0 }),
        s if s.starts_with('#') => parse_hex_color(s),
        _ => None,
    }
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(Color { r, g, b, a: 255 })
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Color { r, g, b, a: 255 })
        }
        _ => None,
    }
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

/// Paint the layout tree using styles from the styled tree
pub fn paint(
    layout_box: &LayoutBox,
    styled: &StyledNode,
    canvas: &mut Canvas,
) {
    match layout_box {
        LayoutBox::Block {
            tag,
            box_dims,
            padding,
            border,
            children,
            ..
        } => {
            // Find the corresponding styled node
            let styled_node = find_styled_element(styled, tag, box_dims.x, box_dims.y);
            let styles = styled_node.map(|s| &s.styles);

            // 1. Paint background
            let bg_color_str = styles
                .and_then(|s| s.get("background-color"))
                .map(|s| s.as_str())
                .unwrap_or("transparent");
            if let Some(bg_color) = parse_color(bg_color_str) {
                if bg_color.a > 0 {
                    let bg_x = box_dims.x - padding.left;
                    let bg_y = box_dims.y - padding.top;
                    let bg_w = padding.left + box_dims.width + padding.right;
                    let bg_h = padding.top + box_dims.height + padding.bottom;
                    canvas.fill_rect(bg_x, bg_y, bg_w, bg_h, bg_color);
                }
            }

            // 2. Paint borders
            let border_width = border.top; // uniform
            if border_width > 0.0 {
                let border_color_str = styles
                    .and_then(|s| s.get("border-color"))
                    .map(|s| s.as_str())
                    .unwrap_or("black");
                if let Some(border_color) = parse_color(border_color_str) {
                    let bx = box_dims.x - padding.left - border.left;
                    let by = box_dims.y - padding.top - border.top;
                    let bw = border.left + padding.left + box_dims.width + padding.right + border.right;
                    let bh = border.top + padding.top + box_dims.height + padding.bottom + border.bottom;

                    // Top border
                    canvas.fill_rect(bx, by, bw, border.top, border_color);
                    // Bottom border
                    canvas.fill_rect(bx, by + bh - border.bottom, bw, border.bottom, border_color);
                    // Left border
                    canvas.fill_rect(bx, by + border.top, border.left, bh - border.top - border.bottom, border_color);
                    // Right border
                    canvas.fill_rect(bx + bw - border.right, by + border.top, border.right, bh - border.top - border.bottom, border_color);
                }
            }

            // 3. Paint children
            // We need to match layout children with styled children
            let styled_children: Vec<&StyledNode> = if let Some(sn) = styled_node {
                sn.children.iter().collect()
            } else {
                styled.children.iter().collect()
            };

            let mut styled_idx = 0;
            for child in children {
                match child {
                    LayoutBox::Block { tag: child_tag, .. } => {
                        // Find matching styled child
                        while styled_idx < styled_children.len() {
                            if let NodeInfo::Element { tag: st, .. } = &styled_children[styled_idx].node {
                                if st == child_tag {
                                    paint(child, styled_children[styled_idx], canvas);
                                    styled_idx += 1;
                                    break;
                                }
                            }
                            styled_idx += 1;
                        }
                    }
                    LayoutBox::Text { content, box_dims: text_box } => {
                        // Get text color and font-size from parent
                        let text_color_str = styles
                            .and_then(|s| s.get("color"))
                            .map(|s| s.as_str())
                            .unwrap_or("black");
                        let font_size = styles
                            .and_then(|s| s.get("font-size"))
                            .map(|s| parse_px(s))
                            .unwrap_or(16.0);
                        let text_color = parse_color(text_color_str)
                            .unwrap_or(Color { r: 0, g: 0, b: 0, a: 255 });

                        // Simple text rendering: draw each character as a small rectangle
                        let char_width = font_size * 0.6;
                        let mut cx = text_box.x;
                        for _ch in content.chars() {
                            if cx + char_width > canvas.width as f32 {
                                break;
                            }
                            canvas.fill_rect(cx, text_box.y, char_width, font_size, text_color);
                            cx += char_width;
                        }

                        // Advance styled_idx for text nodes
                        while styled_idx < styled_children.len() {
                            if let NodeInfo::Text { content: sc } = &styled_children[styled_idx].node {
                                if sc == content {
                                    styled_idx += 1;
                                    break;
                                }
                            }
                            styled_idx += 1;
                        }
                    }
                }
            }
        }
        LayoutBox::Text { .. } => {
            // Text nodes at root level (shouldn't normally happen)
        }
    }
}

/// Find a styled element node matching a tag name (best effort)
fn find_styled_element<'a>(
    styled: &'a StyledNode,
    _tag: &str,
    _x: f32,
    _y: f32,
) -> Option<&'a StyledNode> {
    // Just return the styled node itself - the caller should pass the correct one
    Some(styled)
}
