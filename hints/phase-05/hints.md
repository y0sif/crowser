# Phase 5 Hints: Style Resolution

## Suggested Types

```rust
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct StyledNode {
    pub node: NodeInfo,
    pub styles: IndexMap<String, String>,  // ordered map for property ordering
    pub children: Vec<StyledNode>,
}

/// The DOM node without children (children live in StyledNode::children).
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
```

For specificity:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity(u32, u32, u32); // (id count, class count, tag count)
```

## Architecture

The style resolution phase ties together Phases 1-4 and adds three new operations:

1. **Selector matching** -- does a given selector match a given element?
2. **Specificity calculation** -- how specific is a selector?
3. **Style resolution** -- for each element, collect all matching declarations, resolve conflicts, apply inheritance and defaults.

```rust
/// Does this selector match this element?
fn matches(selector: &Selector, tag: &str, id: Option<&str>, classes: &[String]) -> bool { ... }

/// Compute the specificity of a selector.
fn specificity(selector: &Selector) -> Specificity { ... }

/// Resolve styles for the entire tree.
fn resolve_styles(
    node: &Node,
    stylesheet: &Stylesheet,
    parent_styles: Option<&IndexMap<String, String>>,
) -> StyledNode { ... }
```

The `resolve_styles` function is recursive. For each element node:

1. Collect all declarations from all rules where at least one selector matches.
2. Tag each declaration with the specificity of the matching selector and its source order index.
3. Sort by (specificity, source order). Last one wins for each property.
4. Build the `styles` map: start with defaults, apply inheritance from parent, then apply the matched declarations.

## Rust Tips

**Use `indexmap::IndexMap` instead of `HashMap` for styles.** The spec requires properties in a specific order in the output (`display`, `color`, `background-color`, `width`, `height`, `margin-top`, ...). An `IndexMap` preserves insertion order. Insert properties in the required order when building the styles map. Add `indexmap` to your dependencies with the `serde` feature:

```toml
indexmap = { version = "2", features = ["serde"] }
```

**The full property list with defaults.** Every element must output all 16 supported properties. Build them in order:

```rust
const PROPERTY_ORDER: &[&str] = &[
    "display", "color", "background-color",
    "width", "height",
    "margin-top", "margin-right", "margin-bottom", "margin-left",
    "padding-top", "padding-right", "padding-bottom", "padding-left",
    "border-width", "border-color", "font-size",
];
```

For each property, determine the resolved value by checking (in priority order): matched declarations, inheritance (for `color` and `font-size`), then the default value.

**Default `display` values.** Maintain a function or match expression that returns `"block"` or `"inline"` depending on the tag name:

```rust
fn default_display(tag: &str) -> &str {
    match tag {
        "span" | "a" | "strong" | "em" | "b" | "i" | "u" => "inline",
        _ => "block",
    }
}
```

The spec lists specific block and inline elements. Everything not listed defaults to `"block"`.

**Shorthand expansion.** When you encounter a `margin` or `padding` declaration, expand it into the four longhand properties before inserting into the styles map. Split the value by whitespace and map according to CSS shorthand rules:

- 1 value: all four sides
- 2 values: vertical (top/bottom), horizontal (left/right)
- 3 values: top, horizontal, bottom
- 4 values: top, right, bottom, left

**Class matching.** An element's `class` attribute is a space-separated list. When checking if a selector's class matches, split the attribute value:

```rust
let element_classes: Vec<&str> = attributes
    .get("class")
    .map(|c| c.split_whitespace().collect())
    .unwrap_or_default();
```

Then check that every class in the selector is present in the element's class list.

**Specificity comparison.** Implementing `Ord` on a tuple struct `Specificity(u32, u32, u32)` with `#[derive(Ord)]` gives you lexicographic comparison, which is exactly CSS specificity ordering: compare IDs first, then classes, then tags.

## Common Pitfalls

**Forgetting to output all properties.** The spec says every element's `styles` object must include all 16 supported properties. If you only output the properties that were explicitly set, your tests will fail.

**Inheritance applies only to `color` and `font-size`.** All other properties use their default values when not set. Do not inherit `display`, `background-color`, margins, padding, or `border-width`.

**Root element defaults.** At the root, `color` defaults to `"black"` and `font-size` defaults to `"16px"`. These are the initial values for inherited properties. Non-root elements inherit from their parent.

**Text nodes have empty styles.** Text nodes always have `"styles": {}` and `"children": []`. Do not try to resolve styles for them.

**`<style>` elements have empty children.** The `<style>` element appears in the styled tree (with resolved default styles), but its CSS text content is not included as child text nodes. Set `children: []` for `<style>` elements.

**Comment nodes are excluded.** Drop them entirely during style resolution. They do not appear in the styled tree.

**Selector matching for compound selectors.** `div.highlight` means: tag must be `div` AND element must have class `highlight`. All parts of the compound selector must match. Do not treat them as alternatives.

**Source order for specificity ties.** When two declarations have equal specificity and target the same property, the one appearing later in the stylesheet wins. Track the rule index and declaration index to establish source order.
