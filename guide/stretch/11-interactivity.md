# Stretch Goal: Interactivity

## What You'll Learn

- How to map mouse coordinates to layout boxes (hit testing)
- How to make links clickable and navigate to new pages
- How to implement hover states with re-styling and re-rendering
- How to add scrolling support
- How to change the cursor based on what the mouse is over

## Overview

Your browser engine can render pages, but it does not respond to user input.
You cannot click a link, scroll past the viewport, or hover over an element to
see a style change. This stretch goal adds the interactive layer that turns a
renderer into something that feels like a browser.

## Hit Testing

When the user clicks at pixel position `(mx, my)`, you need to figure out which
element they clicked on. This is called **hit testing**.

The algorithm walks the layout tree and finds the deepest (most specific) box
that contains the point:

```
fn hit_test(node, x, y) -> Option<LayoutBox>:
    // Check children first (they paint on top of the parent)
    for child in node.children.reversed():
        result = hit_test(child, x, y)
        if result is Some:
            return result

    // Check if the point is inside this node's border box
    if point_in_border_box(node, x, y):
        return Some(node)

    return None
```

The border box of a node extends from the outer edge of the border to its
opposite outer edge. Given a content box at `(cx, cy)` with size `(cw, ch)`,
padding `(pt, pr, pb, pl)`, and border `(bt, br, bb, bl)`:

```
border_box_x = cx - pl - bl
border_box_y = cy - pt - bt
border_box_w = bl + pl + cw + pr + br
border_box_h = bt + pt + ch + pb + bb
```

Check children in reverse order because later children paint on top of earlier
ones (in block layout this usually does not matter, but it is the correct
general approach).

## Clickable Links

Once you can identify which element was clicked, you can handle `<a>` elements:

1. On a mouse click event, run hit testing to find the clicked layout box.
2. Walk up the DOM tree from that element to find an `<a>` ancestor (the clicked
   element might be text inside the link, not the `<a>` itself).
3. If an `<a>` element is found, read its `href` attribute.
4. Navigate to the new page: run the full pipeline (fetch if URL, read if file,
   parse, style, layout, paint) and display the result.

If you implemented networking (Phase 9), links can point to URLs. Otherwise,
they can point to local file paths. Either way, the navigation flow is: get the
HTML source, run the pipeline, replace the pixel buffer, and redraw.

You might also want to maintain a simple history stack so the user can go back
(perhaps with a keyboard shortcut like Alt+Left).

## Hover States

CSS `:hover` is a pseudo-class that changes an element's style when the mouse
is over it. Your engine does not support pseudo-classes in the style resolver,
but you can add hover support at the windowing layer:

1. On mouse move events, run hit testing to find the element under the cursor.
2. Track which element is currently "hovered."
3. When the hovered element changes, re-run style resolution with the hover
   state applied. The simplest approach: temporarily add a high-specificity rule
   that applies the `:hover` declarations to the hovered element.
4. Re-run layout and painting.
5. Request a redraw.

This is a brute-force approach (re-rendering the entire page on every hover
change), but for your simple engine it is fast enough. A real browser would use
incremental style invalidation and partial re-layout.

To avoid re-rendering on every single mouse move, only trigger a re-render when
the hovered element actually changes.

## Scrolling

When the page content is taller than the viewport, the user needs to scroll. The
basic model:

1. Maintain a `scroll_offset_y` value, starting at `0`.
2. On scroll events (mouse wheel, trackpad, keyboard arrows/Page Up/Page
   Down), adjust `scroll_offset_y`.
3. When painting, subtract `scroll_offset_y` from every y coordinate. This
   shifts the entire page upward, revealing content below the fold.
4. Clamp `scroll_offset_y` so the user cannot scroll past the top (minimum 0)
   or past the bottom (maximum = page height - viewport height, but not less
   than 0).

The scroll offset does not affect layout. Layout computes positions as if the
page has infinite vertical space. Scrolling only changes which portion of the
laid-out page is visible.

You can implement smooth scrolling by animating the offset change over several
frames, or keep it simple with instant jumps.

## Cursor Changes

Browsers change the mouse cursor to give visual feedback. The most important
cases:

- **Default cursor** over normal content.
- **Pointer cursor** (the hand icon) over clickable links.
- **Text cursor** (the I-beam) over selectable text (if you implement text
  selection).

With `winit`, you set the cursor icon on the window:

```rust
window.set_cursor(CursorIcon::Pointer);  // hand cursor
window.set_cursor(CursorIcon::Default);  // normal arrow
```

On each mouse move, run hit testing, check if the element under the cursor is
(or is inside) an `<a>` element, and set the cursor accordingly.

## Putting It All Together

The event loop from Phase 8 expands to handle these new events:

```
on MouseClick(x, y):
    element = hit_test(layout_tree, x + scroll_x, y + scroll_y)
    if element is inside an <a>:
        navigate to href
on MouseMove(x, y):
    element = hit_test(layout_tree, x + scroll_x, y + scroll_y)
    update hover state and cursor
    if hover changed: re-render and redraw
on Scroll(delta):
    scroll_offset_y += delta
    clamp scroll_offset_y
    request redraw
on RedrawRequested:
    paint with scroll offset applied
    copy to window surface
```

## What's Next

With interactivity, crowser is no longer just a renderer. It is a browser you
can navigate with. From here, the biggest remaining frontiers are JavaScript
support (embed a JS engine and handle `<script>` tags), more complete CSS
support (flexbox, grid, positioning), and performance optimization (incremental
layout, GPU-accelerated painting, caching). Each of those is a substantial
project, but you now have the foundation to build on.
