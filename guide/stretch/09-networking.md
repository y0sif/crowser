# Stretch Goal: Networking

## What You'll Learn

- How to fetch HTML pages over HTTP
- How to resolve and fetch external CSS stylesheets
- How to handle relative and absolute URLs
- How to integrate networking into the rendering pipeline

## Overview

Right now, crowser reads files from disk. A real browser fetches pages from the
internet. This stretch goal adds HTTP support, so you can run:

```
crowser open https://example.com
```

and see the page rendered in your window.

## HTTP Client

Rust has two popular HTTP client crates:

- **`ureq`:** A minimal, blocking HTTP client. No async runtime needed. Add it
  to your dependencies and make a GET request in a few lines. This is the
  simpler option.
- **`reqwest`:** A full-featured client with async support, redirect handling,
  cookies, and more. Uses `tokio` under the hood. More powerful, but heavier.

For a browser engine that renders one page at a time, `ureq` is sufficient.

The basic flow:

```
let response = ureq::get(url).call()?;
let html = response.into_string()?;
```

You now have the HTML source as a string. Feed it into your existing pipeline.

## Detecting URL vs. File Path

Update your argument parsing to distinguish between file paths and URLs. A
simple heuristic: if the argument starts with `http://` or `https://`, treat it
as a URL and fetch it. Otherwise, treat it as a local file path.

## Resolving External Stylesheets

Pages on the web rarely use inline `<style>` blocks exclusively. They link to
external stylesheets:

```html
<link rel="stylesheet" href="/css/style.css">
<link rel="stylesheet" href="https://cdn.example.com/reset.css">
```

To support this:

1. Walk the DOM tree looking for `<link>` elements where `rel="stylesheet"`.
2. Extract the `href` attribute.
3. Resolve the URL (it might be relative).
4. Fetch the CSS source.
5. Concatenate it with any inline `<style>` content.
6. Parse the combined CSS.

### Relative URL Resolution

A relative URL like `/css/style.css` needs a base URL to resolve against. The
base URL is the URL of the page you fetched. Resolution rules:

- `/css/style.css` (starts with `/`): replace the path in the base URL.
  `https://example.com/page.html` + `/css/style.css` = `https://example.com/css/style.css`
- `style.css` (no leading slash): resolve relative to the current path.
  `https://example.com/pages/index.html` + `style.css` = `https://example.com/pages/style.css`
- `https://cdn.example.com/reset.css` (absolute URL): use as-is.

The `url` crate handles this correctly. Use `Url::parse(base)?.join(href)?` and
it will resolve relative URLs according to the standard.

## Connection Flow

The complete fetch-and-render flow becomes:

```
1. Parse the input argument (URL or file path)
2. If URL: fetch the HTML via HTTP
   If file: read from disk
3. Tokenize and parse the HTML to get the DOM
4. Walk the DOM for <style> and <link rel="stylesheet"> elements
5. Collect inline CSS from <style> elements
6. For each <link>, resolve the href URL and fetch the CSS
7. Concatenate all CSS sources in document order
8. Tokenize and parse the CSS to get the stylesheet
9. Continue with style resolution, layout, painting, window
```

## Error Handling

Network requests fail. The server might be down, the URL might be wrong, or the
connection might time out. Handle these gracefully:

- Print a clear error message to stderr.
- Consider rendering what you can (if the HTML loaded but a stylesheet failed,
  render with default styles rather than crashing).
- Set a reasonable timeout (5-10 seconds) so crowser does not hang indefinitely.

## What's Next

With networking, crowser can browse real websites (within the limits of its CSS
and layout support). The next stretch goals, text rendering and interactivity,
will make the experience feel more like using a real browser.
