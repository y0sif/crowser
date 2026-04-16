# Stretch Goal: Text Rendering

## What You'll Learn

- Why proper text rendering is significantly harder than box rendering
- How to use a font rasterization library to render glyphs
- How to implement a basic line-breaking algorithm
- The fundamentals of inline layout

## Why Text Is Hard

In the core phases, you treated each text node as a single rectangle: full
parent width, font-size height. That was a useful simplification, but it
produces output that does not look like real text. Proper text rendering
involves several problems that interact with each other:

**Font loading.** You need a font file (TrueType or OpenType). The operating
system has system fonts, but finding and loading them is platform-specific. You
can also bundle a font file with your binary.

**Glyph rasterization.** A font file contains vector outlines for each
character. To display text, you rasterize those outlines into pixel bitmaps at
the requested size. Each character produces a small grayscale image (the glyph
bitmap) that you composite onto your pixel buffer.

**Metrics and advance widths.** Each glyph has a width (how far the cursor
advances after drawing it), a height, and positioning offsets (ascent, descent,
bearing). You need these to place characters correctly next to each other.

**Line breaking.** When text is wider than its container, it must wrap to the
next line. This requires measuring words, deciding where to break, and
positioning each line vertically.

**Unicode.** Real text is not just ASCII. Unicode brings variable-width
characters, combining marks, right-to-left scripts, ligatures, and complex
shaping rules. Full Unicode support is a deep rabbit hole.

## Font Rasterization with fontdue

The `fontdue` crate is a fast, pure-Rust font rasterizer. It can load a font
file and rasterize individual glyphs at any size.

The basic workflow:

1. Load a font file (`.ttf` or `.otf`) into a `fontdue::Font`.
2. For each character you want to render, call `font.rasterize(char, size)`.
   This returns a `Metrics` struct (width, height, advance) and a `Vec<u8>`
   bitmap (grayscale coverage values, 0-255).
3. Composite the glyph bitmap onto your pixel buffer at the correct position,
   using the text color and the coverage value for anti-aliasing.

Compositing with anti-aliasing means blending the text color with the background
based on the coverage value:

```
output_r = (text_r * coverage + bg_r * (255 - coverage)) / 255
```

This produces smooth, anti-aliased text edges.

The `ab_glyph` crate is another option with a similar API. Both work well.

## Line Breaking

The simplest line-breaking approach is the **greedy algorithm**:

1. Split the text into words (on whitespace).
2. Start a new line.
3. For each word, measure its pixel width (sum of glyph advance widths).
4. If the word fits on the current line (current x + word width <= container
   width), append it.
5. If it does not fit, start a new line and place the word there.
6. After each word, advance the cursor by a space width.

This is not optimal (it can produce ragged right edges), but it handles the
common case well and is straightforward to implement.

You will need a function that measures the pixel width of a string at a given
font size. With fontdue, iterate over the characters, look up each glyph's
advance width, and sum them.

## Inline Layout Basics

Proper text rendering connects to a bigger layout concept: **inline layout**.
In block layout, elements stack vertically. In inline layout, elements flow
horizontally, wrapping to the next line when they run out of space. Text is the
primary inline content.

A basic inline layout model:

1. Maintain a cursor (x, y) that tracks where the next piece of content goes.
2. For each inline element or text run, place it at the cursor and advance x
   by its width.
3. When x exceeds the container width, move to a new line: reset x to the left
   edge, advance y by the line height.
4. The line height is the maximum height of any element on that line.

This is a significant addition to the layout engine. Start with text-only
inline layout (no inline elements like `<span>`) and expand from there.

## Bundling a Font

To avoid the complexity of system font discovery, bundle a font file with your
binary. Good options for open-source, freely distributable fonts:

- **Roboto** (sans-serif, clean, widely used)
- **Liberation Sans/Mono** (metrically compatible with Arial/Courier)
- **Noto Sans** (extensive Unicode coverage)

Include the font file in your binary with `include_bytes!()`:

```rust
const FONT_DATA: &[u8] = include_bytes!("../assets/Roboto-Regular.ttf");
```

This compiles the font into the executable, so there is no runtime file path to
worry about.

## What's Next

With real text rendering, your browser produces output that looks like a
browser. The next stretch goal, [Interactivity](11-interactivity.md), adds mouse
events, clickable links, and scrolling to make it feel like one too.
