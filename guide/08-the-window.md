# Phase 8: The Window

## What You'll Learn

- How to wire the entire rendering pipeline together end to end
- How to display a pixel buffer in an OS window using Rust windowing crates
- How to handle the event loop: redraw, resize, and close events
- How to extract CSS from `<style>` elements in the DOM

## The Big Picture

This is the final phase. You have built every stage of the rendering pipeline
as an independent, testable piece. Now you connect them all into a single
application that reads an HTML file and displays the rendered page in a window
on your screen.

When you type `crowser page.html`, your engine will:

1. Read the HTML file from disk
2. Tokenize the HTML into a token stream
3. Parse the tokens into a DOM tree
4. Extract CSS text from `<style>` elements in the DOM
5. Tokenize the CSS into a token stream
6. Parse the CSS tokens into a stylesheet
7. Resolve styles (DOM tree + stylesheet = styled tree)
8. Compute layout (styled tree + viewport dimensions = layout tree)
9. Paint (layout tree = pixel buffer)
10. Display the pixel buffer in a window

Every step is code you wrote in the previous phases. This phase is about
orchestration: calling them in the right order, passing the output of each
stage as the input to the next, and presenting the final result in a window.

## The Full Pipeline

Here is the pipeline as a data flow, showing the type at each stage:

```
file.html  (String)
    |
    v
HTML Tokenizer  -->  Vec<HtmlToken>
    |
    v
HTML Parser     -->  DomNode (tree)
    |
    v
Extract <style> -->  String (CSS source)
    |
    v
CSS Tokenizer   -->  Vec<CssToken>
    |
    v
CSS Parser      -->  Stylesheet
    |
    v
Style Resolver  -->  StyledNode (tree)
    |
    v
Layout Engine   -->  LayoutBox (tree)
    |
    v
Painter         -->  Vec<u8> (pixel buffer, RGBA)
    |
    v
Window          -->  pixels on screen
```

Each arrow is a function call. The entire pipeline runs in well under a
millisecond for any page your engine can handle, so there is no need for
caching or incremental computation.

## Extracting CSS from the DOM

Your earlier phases accepted CSS as a separate input. Now the CSS lives inside
`<style>` elements in the HTML. You need to walk the DOM tree, find all
`<style>` elements, and extract their text content.

The algorithm:

1. Walk the DOM tree recursively.
2. When you encounter an element with tag name `"style"`, collect the text
   content of its children. (A `<style>` element typically contains a single
   text node with the CSS source.)
3. Concatenate the text from all `<style>` elements, in document order,
   separated by newlines.
4. Pass the combined string to your CSS tokenizer.

If the document contains no `<style>` elements, the CSS source is an empty
string, which produces an empty stylesheet. Every element will get default
styles only.

## Choosing a Windowing Library

You need a way to create an OS window and copy your pixel buffer into it. There
are several options in the Rust ecosystem:

**`minifb`:** The simplest option. A single crate that creates a window and
gives you a `&mut [u32]` buffer to write pixels into. Minimal API, minimal
dependencies. Good for getting started fast.

```rust
// Conceptual usage:
let mut window = Window::new("crowser", width, height, options)?;
while window.is_open() {
    window.update_with_buffer(&pixel_buffer, width, height)?;
}
```

**`winit` + `softbuffer`:** The standard Rust windowing stack. `winit` handles
window creation, event loops, and input. `softbuffer` provides a raw pixel
surface you can write to. More code than `minifb`, but more control and better
cross-platform support.

**`winit` + `pixels`:** Similar to softbuffer but uses `wgpu` under the hood for
GPU-accelerated presentation. Slightly heavier dependency tree but the API is
clean.

Pick whatever works for you. The spec does not mandate a specific library. All
three can display a pixel buffer in a window, which is all you need.

## The Event Loop

GUI applications are event-driven. Instead of running a linear sequence of steps
and exiting, you enter an **event loop** that waits for events (user input,
window management, timers) and responds to them.

The core events you need to handle:

### RedrawRequested

The OS or your code requests that the window be repainted. Copy your pixel
buffer to the window surface. This is the event that actually puts pixels on
screen.

### Resized

The user resized the window. You need to re-run layout and painting with the
new dimensions:

1. Re-run layout with the new viewport width and height.
2. Re-run painting to produce a new pixel buffer.
3. Request a redraw so the new buffer is displayed.

Layout and painting are fast for your simple engine, so doing this on every
resize event is fine.

### CloseRequested / Escape Key

The user clicked the close button or pressed Escape. Exit the event loop and
terminate the program.

### Putting It Together

The overall structure looks like this (pseudocode):

```
fn main():
    html = read_file(path)
    dom = parse_html(tokenize_html(html))
    css_source = extract_style_content(dom)
    stylesheet = parse_css(tokenize_css(css_source))

    create window with title "crowser - filename.html"
    viewport = (window_width, window_height)

    styled_tree = resolve_styles(dom, stylesheet)
    layout_tree = compute_layout(styled_tree, viewport)
    pixel_buffer = paint(layout_tree, viewport)

    run event loop:
        on RedrawRequested:
            copy pixel_buffer to window surface
        on Resized(new_width, new_height):
            viewport = (new_width, new_height)
            layout_tree = compute_layout(styled_tree, viewport)
            pixel_buffer = paint(layout_tree, viewport)
            request redraw
        on CloseRequested or Escape:
            exit
```

Notice that style resolution runs only once (the styles do not change when the
window is resized), but layout and painting run again on every resize because
positions and sizes depend on the viewport dimensions.

## The Default Subcommand

In earlier phases, every feature had an explicit subcommand: `crowser tokenize`,
`crowser parse`, `crowser style`, etc. The window is the default behavior. When
the user runs:

```
crowser page.html
```

with no subcommand, it should open the window. This is what makes crowser feel
like a real browser. You type the command, point it at an HTML file, and it
renders the page.

The `open` subcommand should also work as an explicit alternative:

```
crowser open page.html
```

Both forms do the same thing.

## Setting the Window Title

Set the window title to `"crowser - <filename>"`, where `<filename>` is the
base name of the HTML file (not the full path). For example, if the input is
`/home/user/pages/test.html`, the title should be `"crowser - test.html"`.

## Pixel Format Conversion

Your painter produces RGBA pixels (one byte each for red, green, blue, alpha).
Windowing libraries may expect a different format:

- `minifb` expects `u32` values in `0x00RRGGBB` format (no alpha, packed into a
  single integer).
- `softbuffer` expects `u32` values, typically `0x00RRGGBB`.
- `pixels` accepts RGBA byte arrays directly.

The conversion from RGBA bytes to packed u32 is straightforward:

```rust
let r = rgba[0] as u32;
let g = rgba[1] as u32;
let b = rgba[2] as u32;
let packed: u32 = (r << 16) | (g << 8) | b;
```

Handle this at the boundary between your painter and the windowing library. Keep
your painter's output format clean (RGBA bytes) and convert only when copying to
the window surface.

## Gotchas

**The event loop takes ownership.** Most windowing libraries want to own the
main thread and run the event loop themselves. Structure your code so the
initial render pipeline runs before entering the loop, or run it on the first
`RedrawRequested` event. Store the pixel buffer somewhere the event handler can
access it (a local variable captured by the closure, or a shared state struct).

**Resizing re-runs layout, not style resolution.** Styles do not depend on the
viewport size (in your engine, at least). Only layout and painting need to be
recomputed when the window is resized.

**First paint timing.** Some windowing libraries do not display anything until
the first `RedrawRequested` event is handled. Make sure you request a redraw
after the initial render, or handle the initial render inside the first
`RedrawRequested` callback.

**Error handling for the pipeline.** If the HTML is malformed or the CSS has
parse errors, you should still try to render what you can. Alternatively, print
an error and exit. The choice is yours, but make it deliberate. A browser that
crashes on bad input is not very useful.

**Cross-platform window creation.** If you use `winit`, be aware that window
creation and the event loop have some platform-specific quirks (especially on
macOS, where the event loop must run on the main thread). Follow the library's
documentation and examples.

## What You've Built

Congratulations. You have built a browser engine. Not a toy, not a mock, not a
wrapper around someone else's renderer. You wrote every stage of the pipeline:

- An HTML tokenizer that turns raw text into structured tokens
- An HTML parser that builds a DOM tree from those tokens
- A CSS tokenizer and parser that do the same for stylesheets
- A style resolver that matches selectors, computes specificity, and resolves
  inheritance
- A layout engine that computes concrete pixel positions and sizes
- A painter that rasterizes boxes, borders, and text into a pixel buffer
- A windowing layer that displays the result on screen

Give it any HTML file with inline CSS and it renders in a window. That is a real
browser engine.

## Where to Go from Here

The core pipeline is complete, but there is no shortage of ways to extend it.
The `guide/stretch/` directory covers several directions:

- **[Networking](stretch/09-networking.md):** Fetch pages over HTTP. Support
  `<link>` stylesheets. Type a URL instead of a file path.
- **[Text Rendering](stretch/10-text-rendering.md):** Real font loading, glyph
  rasterization, and line breaking. Make text look good.
- **[Interactivity](stretch/11-interactivity.md):** Mouse events, clickable
  links, hover states, and scrolling. Make it feel like a browser, not just a
  renderer.

You could also explore:

- **Inline layout.** Horizontal flow for inline elements. This is the next big
  layout feature after block layout.
- **JavaScript.** Embed a JS engine (like Boa or QuickJS bindings) and start
  handling `<script>` tags. This is where "browser engine" becomes "browser."
- **Selector combinators.** Descendant (`div p`), child (`div > p`), and
  sibling (`p + p`) selectors.
- **Incremental rendering.** Only re-layout and re-paint the parts of the tree
  that changed, instead of the entire page.

Each of these is a substantial project in its own right. Pick what interests you
and keep building.
