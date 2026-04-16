# Phase 8: The Window

## Command

```
crowser open <file.html>
```

or equivalently:

```
crowser <file.html>
```

If the first argument is a file path (not a recognized subcommand), it is treated as `crowser open <file.html>`.

## Overview

This phase integrates all previous phases into a live, windowed rendering of an HTML page. No new algorithms are introduced -- this phase is about wiring the pipeline together and displaying the result in a native OS window.

The full pipeline executes as follows:

1. **Read** the HTML file from disk
2. **Tokenize HTML** (Phase 1) -- produce a token stream
3. **Parse HTML** (Phase 2) -- build a DOM tree
4. **Extract `<style>` blocks** -- concatenate their text content
5. **Tokenize CSS** (Phase 3) -- produce a CSS token stream
6. **Parse CSS** (Phase 4) -- build a stylesheet
7. **Resolve styles** (Phase 5) -- match rules to DOM nodes, apply cascade and inheritance
8. **Compute layout** (Phase 6) -- calculate box positions and dimensions using the window's current size as the viewport
9. **Paint** (Phase 7) -- render to a pixel buffer
10. **Display** -- copy the pixel buffer to the window

## Window Behavior

### Title

The window title bar should display:

```
crowser - <filename>
```

Where `<filename>` is the basename of the input file (e.g., `page.html`, not the full path).

### Initial Size

The window opens at **800x600 pixels** by default. This matches the default viewport used in Phase 6 and 7.

Optional flags to override the initial size:

```
crowser open <file.html> --width <pixels> --height <pixels>
```

### Resizing

The window is **resizable**. When the user resizes the window:

1. The new window dimensions become the new viewport size
2. Re-run layout (Phase 6) with the new viewport dimensions
3. Re-run painting (Phase 7) with the new canvas size
4. Display the updated pixel buffer

This provides a live reflow experience -- content adjusts to the window size, just like a real browser.

### Closing

The application exits (cleanly, with exit code 0) when:

- The user clicks the window's **close button** (the OS-provided X button)
- The user presses the **Escape** key

### Event Loop

The window runs an event loop that:

1. Waits for events (resize, key press, close request)
2. Handles the event (resize triggers relayout + repaint; Escape/close triggers exit)
3. Repaints only when needed (on resize or initial display)

The event loop should not busy-wait or consume excessive CPU when idle.

## Suggested Libraries

These are suggestions, not requirements. Use whatever works.

- **`winit`** -- cross-platform windowing and event loop. Handles window creation, resizing, keyboard events, and close requests.
- **`softbuffer`** -- provides a simple pixel buffer that can be displayed in a `winit` window. Good for software rendering.
- **`pixels`** -- another option for pixel buffer rendering in a window. Wraps `wgpu` for GPU-accelerated display of a CPU-rendered buffer.
- **`tiny-skia`** -- a 2D rendering library (CPU-based). Can be used as the painting backend in Phase 7 and reused here.
- **`minifb`** -- a minimal framebuffer library. Simpler than `winit` but less flexible.

## Architecture Notes

The window phase should not duplicate logic from earlier phases. Instead, it should call the same functions:

```
file_path
  -> read_file()
  -> tokenize_html()
  -> parse_html()
  -> extract_styles()
  -> tokenize_css() + parse_css()
  -> resolve_styles()
  -> layout(viewport_width, viewport_height)
  -> paint(canvas_width, canvas_height)
  -> display_in_window(pixel_buffer)
```

On resize, only `layout()`, `paint()`, and `display_in_window()` need to run again (the DOM, stylesheet, and resolved styles do not change when the window resizes).

## Manual Test Scenarios

Since this phase is visual and interactive, there are no automated test fixtures. Instead, verify the following scenarios manually:

### Scenario 1: Basic Rendering

```
crowser open test.html
```

Where `test.html` contains a simple colored div. Verify that:
- A window opens with the title "crowser - test.html"
- The colored div is visible at the correct position
- The background is white

### Scenario 2: Resize Reflow

With the window open, drag a corner to resize it:
- Content should re-layout to fill the new width
- Elements without explicit width should expand or shrink
- The repaint should happen promptly (no visible lag for simple pages)

### Scenario 3: Styled Content

Open a file with multiple styled elements (colors, borders, padding, margin):
- The visual output should match what `crowser paint` produces for the same file at the same dimensions
- Colors, borders, and text should all render correctly

### Scenario 4: Escape to Close

With the window open:
- Press the **Escape** key
- The window should close and the process should exit with code 0

### Scenario 5: Close Button

With the window open:
- Click the **close button** (X) on the window's title bar
- The window should close and the process should exit with code 0

### Scenario 6: Display None

Open a file where some elements have `display: none`:
- Hidden elements should not be visible
- Surrounding elements should be laid out as if the hidden elements do not exist

## Out of Scope

The following are **not** handled in this phase:

- **Scrolling** (content that overflows the window is clipped)
- **Text selection**
- **Links and navigation** (clicking `<a>` tags does nothing)
- **Keyboard input** (beyond Escape to close)
- **Mouse interaction** (hover, click events)
- **JavaScript execution**
- **Hot reloading** (editing the HTML file does not auto-refresh; restart the command to see changes)
- **Multiple windows or tabs**
- **Developer tools**

## Exit Codes

- `0` -- window closed normally (close button or Escape)
- `1` -- error (parse error, rendering failure), error message printed to stderr
- `2` -- file not found, print `Error: file not found: <path>` to stderr
