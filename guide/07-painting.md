# Phase 7: Painting

## What You'll Learn

- How to convert the abstract layout tree into concrete draw commands
- The display list approach: separating "what to draw" from "how to draw it"
- How to implement a pixel buffer and rasterize rectangles onto it
- How to handle colors (named colors, hex notation, transparency)
- How to produce a PNG image from your pixel buffer

## The Big Picture

Painting is where your browser engine produces something you can actually see.
The layout tree gives you boxes with positions, sizes, margins, borders, and
padding. The painter walks that tree and fills in pixels.

The output of this phase is a PNG image. Feed it an HTML file with embedded CSS,
and it renders the page to a file you can open in any image viewer. This is the
moment your engine stops being an abstract data pipeline and starts being a
renderer.

## The Display List Approach

Rather than painting directly while walking the layout tree, it is cleaner to
split the process into two stages:

1. **Generate draw commands.** Walk the layout tree and produce a list of
   simple drawing instructions: "fill this rectangle with this color," "draw a
   border here," "render this text at this position."

2. **Execute draw commands.** Iterate over the list and rasterize each command
   onto a pixel buffer.

This separation has practical benefits. You can inspect the draw command list for
debugging. You can sort or reorder commands if needed (for z-ordering in a more
advanced engine). And the rasterizer itself stays simple because it only handles
a few primitive operations.

The draw command list is called the **display list**. Each entry is one of:

- **FillRect(x, y, width, height, color):** fill a rectangle with a solid color
- **StrokeRect(x, y, width, height, color, thickness):** draw the outline of a
  rectangle (four thin rectangles forming the edges)
- **DrawText(x, y, text, color, font_size):** render text at a position

These three commands are enough to paint backgrounds, borders, and text.

## Painting Order

For block layout, document order is the correct painting order. You paint parent
backgrounds first, then child backgrounds on top, then text on top of
everything. For each layout box, the order is:

1. **Background.** If the element has a `background-color` that is not
   `"transparent"`, fill the background rectangle. The background covers the
   content area plus the padding area (but not the border or margin).

2. **Borders.** If `border-width` is greater than zero, draw four rectangles
   for the top, right, bottom, and left edges using the `border-color`.

3. **Children.** Recursively paint each child. This ensures children paint on
   top of the parent's background.

4. **Text.** If this is a text node, draw the text content.

In pseudocode:

```
fn paint_node(node, display_list):
    if node is a block box:
        paint_background(node, display_list)
        paint_borders(node, display_list)
        for child in node.children:
            paint_node(child, display_list)
    if node is a text box:
        paint_text(node, display_list)
```

## The Pixel Buffer

The pixel buffer is a flat array of pixels representing the rendered image. Each
pixel has four components: red, green, blue, and alpha (RGBA). Each component is
a byte (0-255).

```
buffer size = width * height * 4 bytes
```

Initialize every pixel to white (`rgba(255, 255, 255, 255)`) or to whatever
background you prefer for the viewport.

To address pixel `(x, y)`:

```
offset = (y * width + x) * 4
buffer[offset + 0] = red
buffer[offset + 1] = green
buffer[offset + 2] = blue
buffer[offset + 3] = alpha
```

The coordinate system has `(0, 0)` at the top-left corner. X increases to the
right. Y increases downward. This matches how the layout tree positions boxes.

## Implementing Draw Commands

### FillRect

The simplest operation. Iterate over every pixel in the rectangle and set it to
the given color.

```
for py in y..(y + height):
    for px in x..(x + width):
        if px < canvas_width and py < canvas_height and px >= 0 and py >= 0:
            set_pixel(px, py, color)
```

The bounds check ensures you do not write outside the pixel buffer. A box might
extend beyond the viewport (this is fine, just clip to the buffer edges).

### StrokeRect (Borders)

A border is four filled rectangles, one for each edge. Given a box with content
at `(cx, cy)` of size `(cw, ch)`, and padding `(pt, pr, pb, pl)`, and border
widths `(bt, br, bb, bl)`:

The border sits between the padding area and the margin. The outer edge of the
border rectangle is:

```
border_x = cx - pl - bl
border_y = cy - pt - bt
border_w = bl + pl + cw + pr + br
border_h = bt + pt + ch + pb + bb
```

Then the four border rectangles are:

- **Top:** `(border_x, border_y, border_w, bt)`
- **Bottom:** `(border_x, border_y + border_h - bb, border_w, bb)`
- **Left:** `(border_x, border_y + bt, bl, border_h - bt - bb)`
- **Right:** `(border_x + border_w - br, border_y + bt, br, border_h - bt - bb)`

Each is drawn using the element's `border-color`.

Note: in your engine, `border-width` is a single value applied to all four
sides. If you stored it as four separate values during style resolution, use
those. If it is one value, use it for all four sides.

### DrawText

Text rendering is genuinely difficult. Real browsers use font shaping libraries,
glyph rasterizers, and complex layout algorithms. You have several options:

**Option A: Embedded bitmap font.** Define a simple fixed-width font as a set of
hardcoded pixel patterns for ASCII characters. Each character is a small grid
(e.g., 8x16 pixels). To render text, look up each character and copy its pixel
pattern to the buffer. This is the easiest to implement and has zero external
dependencies.

**Option B: Use a font library.** Crates like `fontdue` or `ab_glyph` can
rasterize TrueType/OpenType fonts into pixel bitmaps. You load a font file, ask
the library to rasterize each glyph, and copy the results to your buffer. This
produces much better-looking output but adds a dependency.

**Option C: Skip it for now.** If you want to get pixels on screen fast, start
with backgrounds and borders only. Add text rendering later. The test suite for
this phase focuses on visual layout, not pixel-perfect text.

Pick the approach that fits your goals. A bitmap font gives you visible text with
minimal effort. A font library gives you quality. Skipping it lets you move to
Phase 8 faster.

**A note on test expectations:** The Phase 7 test suite only verifies that a PNG
file is created with non-zero size. It does not perform pixel-level comparison or
validate text rendering quality. You should implement something reasonable, but
do not over-invest in text rendering at this stage -- it will not affect whether
tests pass or fail.

## Color Resolution

Your painter needs to convert CSS color values into RGBA bytes. The supported
color formats are:

**Named colors:**

| Name          | R   | G   | B   | A   |
|---------------|-----|-----|-----|-----|
| `black`       | 0   | 0   | 0   | 255 |
| `white`       | 255 | 255 | 255 | 255 |
| `red`         | 255 | 0   | 0   | 255 |
| `green`       | 0   | 128 | 0   | 255 |
| `blue`        | 0   | 0   | 255 | 255 |
| `yellow`      | 255 | 255 | 0   | 255 |
| `orange`      | 255 | 165 | 0   | 255 |
| `purple`      | 128 | 0   | 128 | 255 |
| `gray`        | 128 | 128 | 128 | 255 |
| `transparent` | 0   | 0   | 0   | 0   |

**Hex colors:**

- `#RGB` is shorthand. Each digit is doubled: `#F0A` becomes `#FF00AA`.
- `#RRGGBB` is parsed directly: `#FF8800` is `R=255, G=136, B=0`.

Parse the hex digits and convert to integer values. All hex colors have alpha
255 (fully opaque).

**Transparent:** The special value `transparent` means alpha 0. When painting a
`transparent` background, simply do not draw anything. The parent's background
(or the viewport's white) shows through.

## The Background Rectangle

The background of an element fills the **content area plus the padding area**.
It does not extend into the border or margin.

Given a layout box with content at `(cx, cy)` of size `(cw, ch)` and padding
`(pt, pr, pb, pl)`:

```
background_x = cx - pl
background_y = cy - pt
background_w = pl + cw + pr
background_h = pt + ch + pb
```

## Saving to PNG

After rasterizing all draw commands, you have a flat RGBA pixel buffer. To save
it as a PNG file, use the `png` crate or the `image` crate.

With the `png` crate, the process is:

1. Create a file
2. Create a PNG encoder with the image dimensions
3. Set the color type to RGBA and bit depth to 8
4. Write the pixel data
5. Finish

With the `image` crate, you can construct an `RgbaImage` from the raw bytes and
call `.save("output.png")`.

The `paint` subcommand writes the PNG to a file. Check the spec for the exact
output path convention.

## Walked Example

Consider this page:

```html
<body>
  <div style="background-color: red; padding: 20px; border-width: 2px;
              border-color: black; width: 200px; height: 100px;">
    Hello
  </div>
</body>
```

Assuming the layout tree gives the `<div>` a content area at `(22, 22)` with
width `200` and height `100`, padding `20` on all sides, and border `2` on all
sides:

**Display list generation:**

1. `FillRect(2, 2, 240, 140, red)` -- background covers content + padding:
   x = 22 - 20 = 2, y = 22 - 20 = 2, w = 20 + 200 + 20 = 240,
   h = 20 + 100 + 20 = 140.

2. `FillRect(0, 0, 244, 2, black)` -- top border.
   `FillRect(0, 142, 244, 2, black)` -- bottom border.
   `FillRect(0, 2, 2, 140, black)` -- left border.
   `FillRect(242, 2, 2, 140, black)` -- right border.

3. `DrawText(22, 22, "Hello", black, 16)` -- text inside the content area.

**Rasterization:**

The rasterizer walks the display list in order. It fills the red background
first, then draws the black border rectangles on top, then places text. The
result is a red box with a thin black border and the word "Hello" inside.

## Gotchas

**Background covers content + padding, not border.** The border is painted
separately on top of the background. If you extend the background into the
border area, the border will be partially hidden behind the background (which
may look fine for opaque backgrounds but is conceptually wrong).

**Transparent means "do nothing."** If an element's background-color is
`transparent`, skip the FillRect. The parent's background shows through
naturally because it was painted first.

**Coordinate system.** (0,0) is top-left, y increases downward. Make sure your
loop iterates in the right direction. If you see output that is vertically
flipped, your y-axis is inverted.

**Bounds checking.** Always clamp pixel coordinates to the buffer dimensions.
A layout box might have a position or size that extends beyond the viewport.
Writing past the buffer boundaries will panic (or corrupt memory in unsafe code).

**Pixel format for later phases.** Your painter produces RGBA pixels, but when
you wire this into a window in Phase 8, the windowing library may expect a
different format (ARGB, BGRA, or packed u32 values). Keep this in mind, the
conversion is straightforward but easy to forget.

## What You'll Have

After this phase, your engine produces visible output. You can render any HTML
file with embedded CSS to a PNG image:

```
crowser paint page.html -o output.png
```

Open `output.png` and you will see colored boxes, borders, and (if you
implemented text rendering) readable text. You have built a renderer. Everything
from raw HTML text to colored pixels is your code.

## What's Next

You have a pixel buffer. In [Phase 8: The Window](08-the-window.md), you will
display it in an actual OS window, handle resize events, and wire the entire
pipeline together into a real application. That is the final phase.
