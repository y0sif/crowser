# Phase 5: Style Resolution

## What You'll Learn

- How selector matching works: given an element and a CSS rule, determine if the
  rule applies
- The specificity scoring system and how the cascade resolves conflicts
- Which CSS properties inherit from parent to child, and which use defaults
- How shorthand properties like `margin` expand into longhand properties
- How to walk the DOM and stylesheet together to produce a styled tree

## The Big Picture

Up to this point, your DOM tree and stylesheet exist in separate worlds. The DOM
knows about elements, attributes, and text. The stylesheet knows about
selectors, properties, and values. Neither knows about the other.

Style resolution is where they meet. You walk the DOM tree, and for each
element, you ask: "Which CSS rules apply to this element, and what are the final
values for every property?" The output is a **styled tree**, a structure that
mirrors the DOM but carries the resolved CSS property values on every node.

This is a three-step process for each element:

1. **Selector matching.** Check every rule in the stylesheet to see which
   selectors match this element. Collect all the declarations from matching
   rules.
2. **Specificity and cascade.** When multiple declarations set the same
   property, pick the winner using specificity (and source order as a
   tiebreaker).
3. **Inheritance and defaults.** For any property that no rule set, either
   inherit from the parent or fall back to a default value.

After processing every element, you have a complete styled tree. That tree is
the input to layout in Phase 6.

## Selector Matching

A selector is a pattern that describes which elements a rule applies to. Your
engine supports four kinds of simple selectors and the ability to combine them.

### Tag Selector

A bare name like `p` or `div`. It matches an element when the element's tag name
equals the selector name.

```
Selector: p
Element:  <p class="intro">...</p>
Match?    Yes -- tag is "p"

Selector: p
Element:  <div>...</div>
Match?    No -- tag is "div", not "p"
```

### Class Selector

A dot followed by a name, like `.main`. It matches an element when the
element's `class` attribute contains that name. Remember that the `class`
attribute can hold multiple space-separated class names:

```html
<div class="container main highlighted">...</div>
```

This element has three classes: `container`, `main`, and `highlighted`. The
selector `.main` matches because `"main"` is in the list. The selector `.sidebar`
does not match because `"sidebar"` is not in the list.

To check: split the element's `class` attribute on whitespace, then see if the
selector's class name appears in the resulting list.

### ID Selector

A hash followed by a name, like `#header`. It matches when the element's `id`
attribute equals the name.

```
Selector: #header
Element:  <div id="header">...</div>
Match?    Yes

Selector: #header
Element:  <div id="footer">...</div>
Match?    No
```

### Universal Selector

The asterisk `*` matches every element, unconditionally.

### Compound Selectors

A compound selector combines multiple simple selectors with no separator:
`div.main#content`. This means "a `div` element that also has class `main` and
id `content`." **All parts must match** for the compound selector to match.

Your CSS parser from Phase 4 already handles these. A compound selector is a
list of simple selectors (at most one tag, at most one ID, zero or more
classes). To check if a compound selector matches an element, verify that every
simple selector in the list matches.

```
Selector: p.highlight
Element:  <p class="highlight intro">...</p>
Match?    Yes -- tag is "p" AND "highlight" is in the class list

Selector: p.highlight
Element:  <span class="highlight">...</span>
Match?    No -- tag is "span", not "p"
```

### The Matching Algorithm

For each element in the DOM tree, iterate over every rule in the stylesheet. For
each rule, iterate over its selectors (a rule can have multiple selectors
separated by commas). If any selector matches the element, collect all of that
rule's declarations.

In pseudocode:

```
for each element in the DOM:
    matched_declarations = []
    for each rule in the stylesheet:
        for each selector in rule.selectors:
            if selector_matches(element, selector):
                for each declaration in rule.declarations:
                    matched_declarations.push((declaration, specificity(selector)))
                break  // no need to check other selectors in same rule
```

## Specificity and the Cascade

When multiple rules set the same property on the same element, you need a way to
pick the winner. CSS uses **specificity**, a scoring system.

### The Scoring System

Specificity is a three-component score: `(id_count, class_count, tag_count)`.
Each component counts how many of that selector type appear in the selector.

| Selector          | IDs | Classes | Tags | Score as number |
|-------------------|-----|---------|------|-----------------|
| `*`               |  0  |    0    |   0  |       0         |
| `p`               |  0  |    0    |   1  |       1         |
| `.main`           |  0  |    1    |   0  |      10         |
| `p.main`          |  0  |    1    |   1  |      11         |
| `#header`         |  1  |    0    |   0  |     100         |
| `div.main#header` |  1  |    1    |   1  |     111         |
| `.a.b.c`          |  0  |    3    |   0  |      30         |

The scoring works like a base-10 number: IDs are worth 100 each, classes are
worth 10 each, and tags are worth 1 each. Compare specificity values as
integers: higher wins.

### Resolving Conflicts

When two declarations set the same property on the same element:

1. The declaration with higher specificity wins.
2. If specificities are equal, the declaration that appears **later in source
   order** wins. ("Source order" means position in the stylesheet. A declaration
   from the second rule in the stylesheet comes after one from the first rule.)

This means you should process rules in order, tracking both the value and its
specificity for each property. When you encounter a new declaration for a
property that already has a value, replace it only if the new specificity is
greater than or equal to the existing one (equal because later-in-source wins
ties).

## Inheritance and Defaults

After cascade resolution, some properties may still have no value because no
matching rule set them. What happens next depends on the property.

### Inheriting Properties

Some CSS properties **inherit**. If no rule sets an inheriting property on an
element, the element takes its parent's computed value for that property. This
continues up the tree. At the root element, inherited properties use their
initial values.

In your engine, two properties inherit:

- **`color`** -- initial value: `"black"`
- **`font-size`** -- initial value: `"16px"`

This is why you can set `color: blue` on the `<body>` and every element inside
inherits that color without you writing a rule for each one. It is one of CSS's
most useful features.

### Non-inheriting Properties

All other supported properties do **not** inherit. If no rule sets them, they
get their default value regardless of what the parent has.

### Default Values

Here is the complete list of supported properties with their defaults:

| Property           | Default           | Inherits? |
|--------------------|-------------------|-----------|
| `display`          | varies by tag     | No        |
| `color`            | `"black"` (root)  | Yes       |
| `background-color` | `"transparent"`   | No        |
| `width`            | `"auto"`          | No        |
| `height`           | `"auto"`          | No        |
| `margin-top`       | `"0px"`           | No        |
| `margin-right`     | `"0px"`           | No        |
| `margin-bottom`    | `"0px"`           | No        |
| `margin-left`      | `"0px"`           | No        |
| `padding-top`      | `"0px"`           | No        |
| `padding-right`    | `"0px"`           | No        |
| `padding-bottom`   | `"0px"`           | No        |
| `padding-left`     | `"0px"`           | No        |
| `border-width`     | `"0px"`           | No        |
| `border-color`     | `"black"`         | No        |
| `font-size`        | `"16px"` (root)   | Yes       |

The `display` property's default depends on the element's tag:

- **Block by default:** `div`, `p`, `h1`-`h6`, `section`, `article`, `header`,
  `footer`, `main`, `ul`, `ol`, `li`, `html`, `body`, `head`
- **Inline by default:** `span`, `a`, `strong`, `em`, `b`, `i`, `u`
- **Everything else:** `"block"`

### Resolution Order

Putting it all together, the resolution for each property on each element is:

1. Did any matching rule set this property? If so, use the winning value (after
   specificity/cascade).
2. Does this property inherit? If so, use the parent's resolved value.
3. Otherwise, use the default.

This is a top-down tree walk. You resolve the root element first (using initial
values for inheritance), then resolve children (using the parent's resolved
values for inheritance), and so on recursively.

## Shorthand Expansion

The `margin` and `padding` shorthand properties expand to their four longhand
forms. Your CSS parser may have already handled this in Phase 4, or you can
expand them during style resolution. Either way, the styled tree should contain
only longhand properties.

### Expansion Rules

**One value:** `margin: 10px` sets all four sides to `10px`.

```
margin-top:    10px
margin-right:  10px
margin-bottom: 10px
margin-left:   10px
```

**Two values:** `margin: 10px 20px` sets top/bottom to the first value and
left/right to the second.

```
margin-top:    10px
margin-right:  20px
margin-bottom: 10px
margin-left:   20px
```

**Three values:** `margin: 10px 20px 30px` sets top, left/right, and bottom.

```
margin-top:    10px
margin-right:  20px
margin-bottom: 30px
margin-left:   20px
```

**Four values:** `margin: 10px 20px 30px 40px` sets top, right, bottom, left
(clockwise from the top).

```
margin-top:    10px
margin-right:  20px
margin-bottom: 30px
margin-left:   40px
```

The same rules apply to `padding`.

### Shorthand vs. Longhand Conflicts

Shorthands and longhands follow the same specificity and source-order rules. If
a rule has both `margin: 10px` and `margin-left: 20px`, the longhand appears
after the shorthand within the same rule, so it overrides just the left value.
The result is `10px` on three sides and `20px` on the left.

## Worked Example

Consider this HTML and CSS:

```html
<html>
<head>
  <style>
    p { color: blue; margin: 5px; }
    .highlight { background-color: yellow; color: green; }
    #title { font-size: 32px; }
  </style>
</head>
<body>
  <div>
    <p id="title" class="highlight">Hello</p>
    <p>World</p>
  </div>
</body>
</html>
```

**Resolving styles for `<p id="title" class="highlight">`:**

Step 1: Selector matching.
- Rule `p { ... }` -- selector `p` matches (it is a `<p>` tag). Specificity: 1.
  Declarations: `color: blue`, `margin: 5px`.
- Rule `.highlight { ... }` -- selector `.highlight` matches (element has class
  `"highlight"`). Specificity: 10. Declarations: `background-color: yellow`,
  `color: green`.
- Rule `#title { ... }` -- selector `#title` matches (element has id `"title"`).
  Specificity: 100. Declaration: `font-size: 32px`.

Step 2: Cascade.
- `color` is set by two rules: `blue` (specificity 1) and `green` (specificity
  10). The `.highlight` rule wins. Resolved: `green`.
- `background-color`: only set by `.highlight` (specificity 10). Resolved:
  `yellow`.
- `margin` shorthand from `p` rule (specificity 1) expands to all four sides:
  `5px` each.
- `font-size`: set by `#title` (specificity 100). Resolved: `32px`.

Step 3: Inheritance and defaults.
- `display`: no rule sets it. Not inherited. Default for `<p>` is `"block"`.
- `width`, `height`: no rule sets them. Not inherited. Default: `"auto"`.
- `padding-*`: no rule sets them. Not inherited. Default: `"0px"`.
- `border-width`: not set. Default: `"0px"`.
- `border-color`: not set. Default: `"black"`.

**Resolving styles for `<p>World</p>`:**

Step 1: Only the `p { ... }` rule matches (specificity 1).
Step 2: `color: blue`, `margin: 5px` on all four sides.
Step 3:
- `font-size`: no rule sets it. It inherits. The parent `<div>` also has no rule
  for `font-size`, so it inherits from its parent, all the way up to the root's
  initial value of `"16px"`.
- `background-color`: not set, not inherited. Default: `"transparent"`.

## Gotchas

**Multiple matching rules.** An element can match many rules. You need to
collect declarations from all of them, not just the first or most specific. The
cascade resolves conflicts per-property, not per-rule.

**Multi-class attributes.** The `class` attribute `"foo bar baz"` means three
separate classes. A selector `.foo` matches, and a selector `.bar` also matches.
Split on whitespace to get the class list.

**Tag-dependent display defaults.** A `<div>` defaults to `display: block` and a
`<span>` defaults to `display: inline`. Do not hardcode a single default for all
elements.

**Style elements in the output.** `<style>` elements appear in the styled tree
with their default property values, but their text children (the CSS source
code) are excluded. The `children` array of a `<style>` node is empty.

**Text nodes carry no styles.** Text nodes in the styled tree have an empty
`styles` object. They do not participate in selector matching or inheritance.
(In later phases, text nodes will implicitly use their parent's resolved styles
for rendering.)

## What You'll Have

After completing this phase, your engine can take an HTML file with embedded CSS
and produce a styled tree where every element carries its fully resolved CSS
properties. You can inspect this with the `style` subcommand:

```
crowser style page.html
```

This prints the styled tree as JSON. You can see exactly which styles resolved
to which values on every element, which is invaluable for debugging the later
phases.

## What's Next

The styled tree tells you *what* each element looks like, but not *where* it
goes on the page. In [Phase 6: Layout](06-layout.md), you will take these style
values and compute concrete positions and sizes for every element, turning the
abstract tree into a geometric description of the page.
