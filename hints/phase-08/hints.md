# Phase 8 Hints: The Window

## Suggested Approach

Use `winit` for cross-platform window creation and event handling, paired with `softbuffer` for pixel buffer display. This combination gives you a window with a CPU-rendered pixel buffer and no GPU dependency.

Alternative: the `minifb` crate provides a simpler all-in-one API. Fewer lines of code, but less control over event handling and window behavior.

```toml
# Cargo.toml
winit = "0.30"
softbuffer = "0.4"
```

## Architecture

Phase 8 does not introduce new algorithms. It wires together all previous phases and adds a window event loop. The key insight: on resize, you only need to re-run layout and paint. The DOM, stylesheet, and resolved styles are unchanged.

```
                  +-- one-time, on startup --+
                  |                          |
file -> tokenize HTML -> parse HTML -> extract <style> content
                                            |
                                    tokenize CSS -> parse CSS
                                            |
                                    resolve styles
                                            |
                  +-- on every resize -------+
                  |                          |
                  +-> layout(width, height) -> paint(width, height) -> display
```

Structure your code to separate the one-time parsing from the repeatable layout/paint:

```rust
fn main() -> Result<(), Box<dyn Error>> {
    let path = parse_args()?;
    let html = read_file(&path)?;

    // One-time: parse and style
    let tokens = tokenize_html(&html);
    let dom = parse_html(tokens);
    let css_text = extract_style_content(&dom);
    let css_tokens = tokenize_css(&css_text);
    let stylesheet = parse_css(css_tokens);
    let styled = resolve_styles(&dom, &stylesheet);

    // Window setup
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title(format!("crowser - {}", filename(&path)))
        .with_inner_size(LogicalSize::new(800, 600))
        .build(&event_loop)?;

    let context = softbuffer::Context::new(&window)?;
    let mut surface = softbuffer::Surface::new(&context, &window)?;

    // Initial render
    let size = window.inner_size();
    let mut pixels = render_pipeline(&styled, size.width, size.height);

    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                // Copy pixels to surface buffer and present
                ...
            }
            Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } => {
                // Re-run layout + paint with new dimensions
                pixels = render_pipeline(&styled, new_size.width, new_size.height);
                window.request_redraw();
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                target.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. }, ..
            } => {
                if event.logical_key == Key::Named(NamedKey::Escape) {
                    target.exit();
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}
```

## Rust Tips

**Pixel format conversion.** Your Phase 7 canvas stores pixels as `[u8]` in RGBA order. `softbuffer` expects `u32` values in `0x00RRGGBB` format (no alpha channel, RGB packed into 32 bits):

```rust
fn rgba_to_softbuffer(canvas: &Canvas) -> Vec<u32> {
    (0..canvas.width * canvas.height)
        .map(|i| {
            let idx = (i * 4) as usize;
            let r = canvas.pixels[idx] as u32;
            let g = canvas.pixels[idx + 1] as u32;
            let b = canvas.pixels[idx + 2] as u32;
            (r << 16) | (g << 8) | b
        })
        .collect()
}
```

**Surface buffer writing.** With softbuffer 0.4:

```rust
let (width, height) = (size.width, size.height);
surface.resize(
    NonZeroU32::new(width).unwrap(),
    NonZeroU32::new(height).unwrap(),
)?;
let mut buffer = surface.buffer_mut()?;
let pixel_data = rgba_to_softbuffer(&canvas);
buffer.copy_from_slice(&pixel_data);
buffer.present()?;
```

**winit 0.30 event loop.** The `winit` 0.30 API uses `ApplicationHandler` trait or the closure-based `event_loop.run()`. The closure approach is simpler for a single-window app. The closure captures your state by move.

**Ownership in the event loop.** The event loop closure takes ownership of everything it captures. Your styled tree, the current pixel buffer, and the surface all need to live inside the closure. One pattern:

```rust
let styled = Arc::new(styled);  // if you need to share
// or just move it into the closure directly
```

For a single-window app, moving everything into the closure is simplest. No `Arc` or `Mutex` needed.

**Extracting `<style>` content from the DOM.** Walk the DOM tree, find elements with `tag == "style"`, collect their text children's content, and concatenate. A recursive function works:

```rust
fn extract_style_content(node: &Node) -> String {
    match node {
        Node::Element { tag, children, .. } if tag == "style" => {
            children.iter().filter_map(|child| {
                if let Node::Text { content } = child {
                    Some(content.as_str())
                } else {
                    None
                }
            }).collect::<Vec<_>>().join("")
        }
        Node::Element { children, .. } | Node::Document { children } => {
            children.iter().map(extract_style_content).collect::<Vec<_>>().join("")
        }
        _ => String::new(),
    }
}
```

**Default subcommand.** The spec says `crowser page.html` (no subcommand) should behave like `crowser open page.html`. With `clap`, you can handle this by checking if the first argument looks like a file path (does not match any known subcommand). One approach:

```rust
// After clap fails to match a subcommand, check if arg is a file path
if let Some(path) = std::env::args().nth(1) {
    if Path::new(&path).extension().is_some() {
        // Treat as: crowser open <path>
    }
}
```

Or configure clap with a default subcommand using `SubcommandRequired(false)` and a positional argument fallback.

**Resize debouncing.** On some platforms, resizing fires many events in rapid succession. Re-running layout and paint on every resize event can feel sluggish. A simple optimization: only re-render on `RedrawRequested`, and call `window.request_redraw()` from the resize handler. This coalesces multiple resizes into one repaint.

## Common Pitfalls

**Zero-sized window.** When the window is minimized, its size may become (0, 0). `softbuffer` and `NonZeroU32` will panic. Guard against this:

```rust
if new_size.width == 0 || new_size.height == 0 {
    return;
}
```

**Surface must be resized before writing.** If you write to the surface buffer without calling `surface.resize()` first (or if the dimensions do not match), you will get a panic or garbled output. Always resize the surface when the window size changes.

**Event loop blocks.** `event_loop.run()` does not return (or returns `!` / `ExitStatus`). Any code after it will not execute. Do all setup before entering the event loop.

**Platform differences.** On Wayland (common on modern Linux), window behavior differs from X11. `winit` handles most of this, but initial window size and title behavior can vary. Test on your target platform.

**Escape key detection.** With `winit` 0.30, keyboard events use `KeyEvent` with `logical_key`. Match against `Key::Named(NamedKey::Escape)`. Older winit versions used `VirtualKeyCode::Escape`. Check the version you are using.
