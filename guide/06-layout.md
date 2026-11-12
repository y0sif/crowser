# Phase 6: Layout

## What You'll Learn

- The CSS box model and how content, padding, border, and margin relate
- How block layout works: width flows top-down, height flows bottom-up
- The concrete algorithm for computing positions and sizes of every element
- How text nodes fit into the layout model
- How `display: none` elements are handled

## The Big Picture

Style resolution told you *what* each element looks like: its colors, its
margins, its font size. Layout answers the next question: *where* does each
element go, and how big is it?

Layout takes the styled tree and produces a **layout tree**, where every node
has a concrete position (x, y) and concrete dimensions (width, height) in
pixels. This is the bridge between abstract CSS properties and the pixel grid
that the painter will draw on.

In a full browser engine, layout is by far the most complex phase. It handles
block flow, inline flow, floats, flexbox, grid, tables, and dozens of
interactions between them. Your engine implements **block layout only**, which is
the simplest and most fundamental model: elements stack vertically, top to
bottom. This is enough to lay out real pages with divs, headings, paragraphs,
and nested containers.

## The Box Model

The box model is the most important concept in CSS layout. Every element
generates a rectangular box with four nested areas:

```
+--------------------------------------------------------------+
|                          MARGIN                              |
|  +--------------------------------------------------------+  |
|  |                        BORDER                          |  |
|  |  +--------------------------------------------------+  |  |
|  |  |                     PADDING                      |  |  |
|  |  |  +--------------------------------------------+  |  |  |
|  |  |  |                                            |  |  |  |
|  |  |  |                 CONTENT                    |  |  |  |
|  |  |  |            (width x height)                |  |  |  |
|  |  |  |                                            |  |  |  |
|  |  |  +--------------------------------------------+  |  |  |
|  |  |                                                  |  |  |
|  |  +--------------------------------------------------+  |  |
|  |                                                        |  |
|  +--------------------------------------------------------+  |
|                                                              |
+--------------------------------------------------------------+
```

- **Content area:** where the element's actual content lives. This is what
  `width` and `height` control. In the layout tree, `box.x`, `box.y`,
  `box.width`, and `box.height` describe this area.
- **Padding:** transparent space between the content and the border. Background
  color fills the padding area too.
- **Border:** the visible edge of the element. It has its own width.
- **Margin:** transparent space between this element's border and neighboring
  elements. Margins are invisible and never painted.

The **total width** an element occupies horizontally is:

```
margin-left + border-left + padding-left + content-width + padding-right + border-right + margin-right
```

The **total height** it occupies vertically is:

```
margin-top + border-top + padding-top + content-height + padding-bottom + border-bottom + margin-bottom
```

These totals matter because they determine how much space a child takes up
inside its parent, which affects where the next sibling is positioned.

## The Containing Block

Every element is laid out relative to its **containing block**, which is the
content area of its parent layout box. The containing block constrains the
element's width and determines the coordinate space for its position.

For the root element, the containing block is the **viewport**. By default, the
viewport is 800 pixels wide and 600 pixels tall. These values can be overridden
with the `--width` and `--height` flags.

## The Block Layout Algorithm

Block layout has four steps, and the order matters. Width is calculated first
(top-down), then position, then children are laid out recursively, and finally
height is calculated (bottom-up).

### Step 1: Calculate Width

If the element's CSS sets an explicit `width` (like `width: 200px`), use that
value as `box.width`.

Otherwise, the content width is whatever is left after subtracting margins,
borders, and padding from the containing block's width:

```
box.width = containing_width
          - margin_left - border_left - padding_left
          - padding_right - border_right - margin_right
```

This is why a `<div>` with no width set fills the entire width of its parent
(minus its own margin/border/padding). Width flows **top-down**: the parent's
width constrains the child.

### Step 2: Calculate Position

The position (`box.x`, `box.y`) is the top-left corner of the content area.

**Horizontal position:**

```
box.x = parent_content_x + margin_left + border_left + padding_left
```

Where `parent_content_x` is the parent's `box.x` (the parent's content area
already accounts for the parent's own padding and border).

**Vertical position:**

Block-level children stack vertically. You keep a **cursor** that tracks the y
position where the next child should go. The cursor starts at the parent's
content area y (`parent.box.y`).

For each child:

```
box.y = cursor_y + margin_top + border_top + padding_top
```

After laying out the child, advance the cursor past the child's total height:

```
cursor_y = box.y + box.height + padding_bottom + border_bottom + margin_bottom
```

This places each child directly below the previous one, with margins between
them.

### Step 3: Lay Out Children

Recursively apply the layout algorithm to each child, in document order. The
child's containing block is the current element's content area.

This is where the recursion happens. You calculate the current element's width
and position first, then lay out children inside it. The children may have
children of their own, all the way down the tree.

### Step 4: Calculate Height

If the element's CSS sets an explicit `height` (like `height: 100px`), use that
value as `box.height`.

Otherwise, the content height is the total vertical space occupied by all
children:

```
box.height = cursor_y - box.y
```

Where `cursor_y` is the final cursor position after laying out all children. If
there are no children, `box.height` is `0.0`.

Height flows **bottom-up**: children determine the parent's height. This is the
opposite of width, and it is the fundamental asymmetry of block layout. You
cannot know an element's height until you have laid out all of its descendants.

## A Concrete Example

Consider this page:

```html
<body>
  <div style="margin: 10px; padding: 5px; border-width: 2px;">
    <p style="margin: 5px;">Hello</p>
    <p style="margin: 5px;">World</p>
  </div>
</body>
```

Viewport: 800 x 600. Walk through the layout.

**`<body>` (root layout box):**

All margins, borders, and padding are 0 (defaults).

```
width  = 800 - 0 - 0 - 0 - 0 - 0 - 0 = 800
box.x  = 0 + 0 + 0 + 0 = 0
box.y  = 0 + 0 + 0 + 0 = 0
```

**`<div>` (child of body):**

Margin: 10 on all sides. Border: 2 on all sides. Padding: 5 on all sides.

```
width  = 800 - 10 - 2 - 5 - 5 - 2 - 10 = 766
box.x  = 0 + 10 + 2 + 5 = 17
box.y  = 0 + 10 + 2 + 5 = 17
```

The cursor inside the div starts at `cursor_y = 17` (the div's `box.y`).

**First `<p>` (child of div):**

Margin: 5 on all sides. Border and padding: 0.

```
width  = 766 - 5 - 0 - 0 - 0 - 0 - 5 = 756
box.x  = 17 + 5 + 0 + 0 = 22
box.y  = 17 + 5 + 0 + 0 = 22
```

The `<p>` has one text child. Text width = parent content width = 756. Text
height = font-size = 16 (the default). So `box.height` of the `<p>` is
`16.0` (sum of children's total heights, and the text node has no margin,
border, or padding).

Advance cursor: `cursor_y = 22 + 16 + 0 + 0 + 5 = 43`

**Second `<p>` (child of div):**

```
width  = 766 - 5 - 0 - 0 - 0 - 0 - 5 = 756
box.x  = 17 + 5 + 0 + 0 = 22
box.y  = 43 + 5 + 0 + 0 = 48
```

Same height calculation: `box.height = 16.0`.

Advance cursor: `cursor_y = 48 + 16 + 0 + 0 + 5 = 69`

**Back to `<div>`, height calculation:**

```
box.height = cursor_y - box.y = 69 - 17 = 52
```

**Back to `<body>`, height calculation:**

The div's total height is: `10 + 2 + 5 + 52 + 5 + 2 + 10 = 86`. The body's
cursor went from 0 to 86, so `box.height = 86`.

## Text Nodes

Text layout in a real browser is enormously complex (font metrics, line
breaking, word wrapping, bidirectional text). Your engine uses a deliberate
simplification:

- Text width = parent's content width (text fills the full line)
- Text height = the resolved `font-size` value (e.g., `16.0` for `16px`)

Text nodes produce layout boxes of type `"Text"`. They carry no padding, border,
or margin. Their `box.x` and `box.y` are positioned by the parent's cursor, same
as any other child.

This is a rough approximation, but it is enough to produce visible, recognizable
output. Proper text rendering is a stretch goal.

## Display None

Elements with `display: none` are **excluded entirely** from the layout tree.
They produce no layout box and consume no space. When iterating over children
during layout, skip any child whose resolved `display` is `"none"`.

## Excluded Elements

The layout tree does not include everything from the styled tree. The following
are excluded:

- Elements with `display: none`
- `<head>` elements and their children (not visual content)
- `<style>` elements (they carry CSS, not renderable content)
- Comment nodes (already dropped in Phase 5)

The root of the layout tree corresponds to `<body>` (or the outermost renderable
element if there is no `<body>`). Skip `<html>` and `<head>` when walking the
styled tree to find the layout root.

## Parsing CSS Values for Layout

You will need to extract numeric values from CSS strings:

- `"200px"` becomes `200.0`
- `"0px"` becomes `0.0`
- `"auto"` triggers the auto-sizing algorithm (described above for width and
  height)
- `"16px"` for font-size becomes `16.0` (used as text height)

All numeric values in the output use one decimal place: `800.0`, not `800` or
`800.00`.

## Gotchas

**Width is top-down, height is bottom-up.** You must calculate the width before
laying out children (because children need to know their containing block's
width). But you cannot calculate the height until after all children are laid
out (because the height depends on how much vertical space the children
consumed). This ordering is fundamental to the algorithm.

**Do not forget border and padding in position calculations.** A common mistake
is calculating `box.x` as `parent.box.x + margin_left` and forgetting that
border and padding also offset the content area. The content area starts
*inside* the border and padding, not at the edge of the margin.

**The cursor tracks the total space consumed.** After each child, the cursor
advances past the child's content, padding, border, and margin. Not just the
content height.

**Text layout is intentionally simplified.** Real browsers handle line breaking,
word wrapping, font metrics, and inline formatting contexts. Your engine treats
each text node as a single rectangle. This is fine for this phase.

**Float precision.** Output all numbers with one decimal place. This keeps test
comparisons predictable.

**The painter will need styles too.** The layout JSON holds geometry only, but
in Phase 7 the painter needs each box's `background-color`, `border-color`, and
`color`. Keep the resolved styles on your layout box internally (a text box
takes them from its parent element), and leave them out of the JSON output. The
tests compare the whole object, so an extra field in the output makes them fail.

## What You'll Have

After this phase, your engine can take an HTML file with embedded CSS and produce
a layout tree with concrete pixel positions and sizes for every element. You can
inspect this with the `layout` subcommand:

```
crowser layout page.html
crowser layout page.html --width 1024 --height 768
```

The JSON output tells you exactly where every box will be painted. If something
looks wrong in the final rendered output, you can check the layout tree to see
whether the positions are correct before even looking at the painter.

## What's Next

You know what each element looks like (Phase 5) and where it goes (Phase 6).
In [Phase 7: Painting](07-painting.md), you will convert this layout tree into
actual pixels, producing a visible image of the page.
