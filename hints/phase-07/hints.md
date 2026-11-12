# Phase 7 Hints: Painting

## Suggested Types

```rust
pub struct Canvas {
    pub pixels: Vec<u8>,  // RGBA format, length = width * height * 4
    pub width: u32,
    pub height: u32,
}

pub enum DisplayCommand {
    FillRect {
        rect: FillArea,
        color: Color,
    },
    DrawText {
        x: f32,
        y: f32,
        text: String,
        color: Color,
        font_size: f32,
    },
}

pub struct FillArea {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
```

## Architecture

The painting phase has three stages:

```rust
/// Stage 1: Walk the layout tree and produce a flat list of draw commands.
/// The colors come from the styles stored on each layout box in Phase 6.
pub fn build_display_list(layout: &LayoutBox) -> Vec<DisplayCommand> { ... }

/// Stage 2: Execute draw commands onto a pixel buffer.
pub fn render(commands: &[DisplayCommand], canvas: &mut Canvas) { ... }

/// Stage 3: Write the pixel buffer to a PNG file.
pub fn save_png(canvas: &Canvas, path: &str) -> Result<(), Box<dyn Error>> { ... }
```

The display list approach separates "what to draw" from "how to draw it." This makes it easy to add new rendering backends later (window display in Phase 8 reuses the same display list).

**Building the display list.** Traverse the layout tree in pre-order (parent before children). For each layout box, emit commands in this order:

1. Background color (FillRect on the padding box area)
2. Border rectangles (four FillRects, one per side)
3. Recurse into children
4. Text content (DrawText for Text nodes)

Painting the background before recursing into children ensures children paint on top of their parent's background.

## Rust Tips

**The `image` crate for PNG output.** The simplest way to write a PNG:

```rust
use image::{ImageBuffer, Rgba};

fn save_png(canvas: &Canvas, path: &str) -> Result<(), Box<dyn Error>> {
    let img = ImageBuffer::from_fn(canvas.width, canvas.height, |x, y| {
        let idx = ((y * canvas.width + x) * 4) as usize;
        Rgba([
            canvas.pixels[idx],
            canvas.pixels[idx + 1],
            canvas.pixels[idx + 2],
            canvas.pixels[idx + 3],
        ])
    });
    img.save(path)?;
    Ok(())
}
```

Add to `Cargo.toml`:

```toml
image = { version = "0.25", default-features = false, features = ["png"] }
```

Disabling default features and enabling only `png` keeps compile times short.

**Canvas initialization.** Fill the entire canvas with white (the spec says the default background is `#FFFFFF`):

```rust
impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            pixels: vec![255; (width * height * 4) as usize],  // white, fully opaque
            width,
            height,
        }
    }
}
```

**Pixel writing.** A helper to set a single pixel, with bounds checking:

```rust
impl Canvas {
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height {
            return;  // clip out-of-bounds
        }
        let idx = ((y * self.width + x) * 4) as usize;
        self.pixels[idx] = color.r;
        self.pixels[idx + 1] = color.g;
        self.pixels[idx + 2] = color.b;
        self.pixels[idx + 3] = color.a;
    }
}
```

**FillRect with clamping.** When filling a rectangle, convert float coordinates to integer pixel coordinates and clamp to canvas bounds:

```rust
fn fill_rect(canvas: &mut Canvas, area: &FillArea, color: Color) {
    let x0 = area.x.round().max(0.0) as u32;
    let y0 = area.y.round().max(0.0) as u32;
    let x1 = (area.x + area.width).round().min(canvas.width as f32) as u32;
    let y1 = (area.y + area.height).round().min(canvas.height as f32) as u32;

    for y in y0..y1 {
        for x in x0..x1 {
            canvas.set_pixel(x, y, color);
        }
    }
}
```

**Color parsing.** You need a function to convert CSS color strings to `Color` values:

```rust
fn parse_color(value: &str) -> Option<Color> {
    match value {
        "black" => Some(Color { r: 0, g: 0, b: 0, a: 255 }),
        "white" => Some(Color { r: 255, g: 255, b: 255, a: 255 }),
        "red" => Some(Color { r: 255, g: 0, b: 0, a: 255 }),
        "green" => Some(Color { r: 0, g: 128, b: 0, a: 255 }),
        "blue" => Some(Color { r: 0, g: 0, b: 255, a: 255 }),
        "yellow" => Some(Color { r: 255, g: 255, b: 0, a: 255 }),
        "orange" => Some(Color { r: 255, g: 165, b: 0, a: 255 }),
        "purple" => Some(Color { r: 128, g: 0, b: 128, a: 255 }),
        "gray" => Some(Color { r: 128, g: 128, b: 128, a: 255 }),
        "transparent" => Some(Color { r: 0, g: 0, b: 0, a: 0 }),
        s if s.starts_with('#') => parse_hex_color(s),
        _ => None,
    }
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        3 => {
            // #RGB -> #RRGGBB (double each digit)
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
```

**Border painting.** Four separate rectangles, not a stroked outline. Given the border box `(bx, by, bw, bh)` and border widths `(top, right, bottom, left)`:

```
Top:    (bx, by, bw, top)
Bottom: (bx, by + bh - bottom, bw, bottom)
Left:   (bx, by + top, left, bh - top - bottom)
Right:  (bx + bw - right, by + top, right, bh - top - bottom)
```

Note that the left and right borders start below the top border and end above the bottom border, to avoid double-painting the corners.

**Text rendering.** For the simplest approach, use the `fontdue` crate to rasterize individual glyphs from a bundled font. Or go even simpler: embed a hardcoded bitmap font (8x16 pixels per character). Each character is a fixed-width rectangle of pixels. This avoids any font loading complexity and is enough to pass the visual tests.

If you want something between "no text" and "real font rendering," character-sized colored rectangles (one rectangle per character) at least show where text would appear.

## Common Pitfalls

**Background covers the padding box, not just the content box.** The background rectangle starts at `(box.x - padding.left, box.y - padding.top)` and has size `(padding.left + box.width + padding.right, padding.top + box.height + padding.bottom)`. If you paint only the content area, elements with padding will have a white gap between the content and the border.

**Transparent means "do nothing."** When `background-color` is `"transparent"`, skip the fill entirely. Do not paint transparent black pixels over the existing canvas content.

**Accessing styles from the layout tree.** The layout tree (Phase 6) does not carry the full styled node. You need the resolved styles to know colors and border-color. Either store the resolved styles in the layout box, or pass the styled tree alongside the layout tree and correlate them. The simplest approach is the first one: the Phase 6 hints suggest a `styles` field on the layout box marked `#[serde(skip)]`, so the `layout` output stays unchanged. If you did not add it in Phase 6, add it now.

**Pixel format.** Your canvas is RGBA (4 bytes per pixel). The `image` crate expects RGBA. If you later display in a window (Phase 8), the window library may expect a different format (e.g., `0x00RRGGBB` as `u32`). Keep the canvas in RGBA and convert at the display boundary.

**Out-of-bounds writes.** Always clamp pixel coordinates to `[0, width)` and `[0, height)`. A layout box can have negative margins or extend past the canvas edge. Writing out-of-bounds will panic or corrupt memory.
