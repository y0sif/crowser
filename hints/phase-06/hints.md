# Phase 6 Hints: Layout

## Suggested Types

```rust
use indexmap::IndexMap;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct LayoutBox {
    #[serde(rename = "type")]
    pub box_type: BoxType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub box_dims: Rect,       // serialized as "box"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<Edges>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border: Option<Edges>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin: Option<Edges>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<LayoutBox>>,
    #[serde(skip)]
    pub styles: IndexMap<String, String>,  // for the painter, never serialized
}

#[derive(Debug, Clone, Serialize)]
pub enum BoxType {
    Block,
    Text,
    Anonymous,
}

#[derive(Debug, Clone, Serialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}
```

Note: the spec says the JSON field is `"box"`, not `"box_dims"`. Use `#[serde(rename = "box")]` on the field. Also, `"box"` is a reserved keyword in Rust, so you cannot name the field `box`. The rename attribute handles this.

Text nodes have a different shape in the output: they have `content` and `box` but no `padding`, `border`, `margin`, or `children`. Using `Option` with `skip_serializing_if` handles this, but you might prefer separate structs or an enum with `#[serde(untagged)]` for cleaner code.

The `styles` field is not part of the Phase 6 output. Layout itself does not need it, but the painter in Phase 7 reads `background-color`, `border-color`, and `color` from each box, so it is easiest to copy the resolved styles onto the box now. For a text node, copy the parent element's styles, since text nodes have none of their own. `#[serde(skip)]` keeps the field out of the JSON. This matters because the tests compare the whole object, and any extra key is a failure.

## Architecture

```rust
/// Entry point: takes the styled tree and viewport, returns the layout tree.
pub fn layout(styled: &StyledNode, viewport_width: f32, viewport_height: f32) -> LayoutBox {
    let containing = Rect {
        x: 0.0,
        y: 0.0,
        width: viewport_width,
        height: viewport_height,
    };
    layout_block(styled, &containing)
}

/// Layout a block-level element.
fn layout_block(styled: &StyledNode, containing: &Rect) -> LayoutBox {
    // 1. Calculate width
    // 2. Calculate x position
    // 3. Layout children (which determines content height)
    // 4. Calculate height (explicit or from children)
    ...
}
```

The order matters. Width must be calculated before children are laid out, because children need to know the parent's content width. Height is calculated after children, because auto height depends on children's total size.

## Rust Tips

**Parsing CSS values to floats.** You will frequently need to extract numeric values from CSS strings like `"200px"`, `"0px"`, `"auto"`. A small helper:

```rust
fn parse_px(value: &str) -> f32 {
    if value == "auto" || value == "0" {
        return 0.0;
    }
    value.strip_suffix("px")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0)
}
```

**Use `f32` throughout.** The spec requires one-decimal-place output. Using `f32` avoids integer rounding issues during intermediate calculations. For serialization, you can implement a custom serializer or round at the end. A simple approach: round all values to one decimal place before serializing.

**Custom float serialization.** Serde will serialize `800.0_f32` as `800.0`, which matches the spec. But `10.0_f32` might serialize as `10` (no decimal point) in some configurations. To guarantee one decimal place, you can use a custom serializer:

```rust
fn serialize_f1<S: serde::Serializer>(val: &f32, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_f64(((*val * 10.0).round() / 10.0) as f64)
}
```

Or format floats manually with `format!("{:.1}", val)` and serialize the result. Test your output to make sure it matches.

**The y cursor.** When laying out children, maintain a `cursor_y` that tracks where the next child should start. Initialize it to the parent's content area y. For each child:

```
cursor_y += child.margin.top + child.border.top + child.padding.top;
child.box.y = cursor_y;
// ... layout child's own children ...
cursor_y = child.box.y + child.box.height
    + child.padding.bottom + child.border.bottom + child.margin.bottom;
```

After all children, the parent's auto height is `cursor_y - parent.box.y`.

**Excluded nodes.** Skip `<html>`, `<head>`, and `<style>` during layout. The root layout box corresponds to `<body>` (or the first renderable element if there is no `<body>`). When walking the styled tree, you need to "unwrap" the `<html>` and find the `<body>` child.

**display: none.** Elements with `display: none` produce no layout box and take no space. Check the resolved `display` property before creating a layout box. Skip the element and all its descendants.

## Common Pitfalls

**Width before children, height after.** If you calculate height before laying out children, auto-height elements will all be 0. The algorithm is: set width, set x, set y, lay out children (which sets their positions and sizes), then calculate own height from children.

**Forgetting to advance the y cursor.** After laying out each child, advance `cursor_y` past the child's total box (content + padding + border + margin). If you forget the bottom margin/border/padding, the next child will overlap.

**Content x position.** The content area x is not just `parent.x + margin.left`. It is `parent_content_x + own_margin_left + own_border_left + own_padding_left`. The content x accounts for all three layers of the box model.

**Text node height.** Text nodes use the inherited `font-size` as their height. The default is `16.0`. Their width equals the parent's content width (they fill the line). Make sure you grab `font-size` from the parent styled node's resolved styles, not from the text node itself (text nodes have empty styles).

**Float precision in output.** The spec wants `22.0`, not `22` or `22.00`. Check your serialized JSON. If serde drops the decimal for whole numbers, you will need custom serialization.

**Auto width calculation.** When width is `"auto"`, the element expands to fill the containing block minus its own margins, borders, and padding. This is **not** the same as the containing block's width. The formula:

```
auto_width = containing.width - margin.left - margin.right
    - border.left - border.right - padding.left - padding.right
```

If you forget to subtract borders or padding, elements will overflow their container.
