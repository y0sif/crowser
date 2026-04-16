# Phase 0: Introduction

## What You'll Learn

- Why building a browser engine is a worthwhile exercise
- The full rendering pipeline from HTML text to pixels on screen
- How to set up the crowser project and CLI skeleton
- The structure of the eight phases ahead of you

## Why Build a Browser Engine?

The web browser is the most complex piece of software most people use every
day. It parses two different languages (HTML and CSS -- three if you count
JavaScript), constructs multiple tree data structures, runs a constraint-based
layout algorithm, and paints the result to the screen at 60 frames per second.
And yet, to most developers, it is a black box. HTML goes in, a page comes out,
and everything in between is someone else's problem.

Building a browser engine from scratch changes that. Not because you will
produce something that rivals Chrome or Firefox -- you will not -- but because
you will understand the *mechanics*. When you write a tokenizer, you learn why
`<br />` and `<br/>` are the same thing. When you write a parser, you
understand why the DOM is a tree and not a list. When you write a layout
engine, you finally internalize what `margin: auto` actually does.

This understanding transfers directly to your daily work. Debugging CSS layout
issues becomes reasoning about box model arithmetic instead of trial and error.
Performance questions become tractable when you know which stage of the pipeline
is doing the work. The browser stops being magic and starts being a machine you
could have built -- because you did.

## A Brief History

The web started simple. Tim Berners-Lee's original proposal was a hypertext
document system: static documents linked together. HTML was just a way to mark
up text with structure -- headings, paragraphs, lists, links. The first web
browser was also the first web editor. There was no CSS. There was no
JavaScript. A "browser engine" was barely more than a text formatter that
understood a few tags.

Then things got complicated. CSS arrived in 1996, giving authors control over
presentation. JavaScript made pages interactive. The DOM became a programming
interface. Standards accumulated. Browser engines grew from thousands of lines
to millions. Today, Chromium's rendering engine alone is larger than most
operating systems.

But the core pipeline has not changed. It is still: parse the HTML, parse the
CSS, match styles to elements, compute layout, paint pixels. Every browser
engine in existence does these steps. The details vary enormously, but the
architecture is the same.

That is the pipeline you are going to build.

## The Eight Phases

Each phase builds on the previous one. Here is what you will construct and what
each phase teaches you:

| Phase | Name                | What You Build                                        | Core Concept                        |
|:-----:|---------------------|-------------------------------------------------------|-------------------------------------|
|   1   | HTML Tokenizer      | A state machine that breaks HTML into tokens          | Lexical analysis, state machines    |
|   2   | HTML Parser         | A tree builder that assembles the DOM                 | Tree construction, stacks           |
|   3   | CSS Tokenizer       | A lexer for CSS syntax                                | A different grammar, same technique |
|   4   | CSS Parser          | A parser that produces structured stylesheets         | Selector/declaration parsing        |
|   5   | Style Resolution    | An engine that matches CSS rules to DOM elements      | Selector matching, specificity      |
|   6   | Layout              | A layout engine implementing the CSS box model        | Coordinate geometry, constraints    |
|   7   | Painting            | A renderer that produces actual pixel output           | Rasterization, image generation     |
|   8   | The Window          | An OS window that displays the rendered page          | Platform graphics, event loops      |

The phases are sequential. Phase 2 consumes the output of Phase 1. Phase 5
combines the outputs of Phase 2 and Phase 4. Phase 7 builds on Phase 6. You
cannot skip ahead -- each phase gives you the foundation for the next.

## Setting Up Your Project

Initialize a new Rust project:

```
cargo init crowser
cd crowser
```

You will need a CLI framework for subcommand parsing. `clap` with the `derive`
feature is a good choice:

```toml
# Cargo.toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

You will also want `serde` and `serde_json` from the start -- every phase
outputs JSON, and being able to derive `Serialize` on your data structures
saves significant boilerplate.

## Suggested Project Structure

You can organize your code however you like, but here is a structure that maps
cleanly to the phases:

```
src/
├── main.rs          # CLI entry point with subcommands
├── html/
│   ├── mod.rs
│   ├── tokenizer.rs # Phase 1
│   └── parser.rs    # Phase 2
├── css/
│   ├── mod.rs
│   ├── tokenizer.rs # Phase 3
│   └── parser.rs    # Phase 4
├── style.rs         # Phase 5
├── layout.rs        # Phase 6
├── paint.rs         # Phase 7
└── window.rs        # Phase 8
```

Each module handles one stage of the pipeline. `main.rs` dispatches to the
right module based on the subcommand. This keeps the codebase navigable as it
grows.

## Your First Task: The CLI Skeleton

Before you write any parsing logic, set up the subcommand structure. Your
binary should accept all eight subcommands and print a placeholder message for
each. Something like this in pseudocode:

```rust
// Conceptual sketch -- not complete code
enum Command {
    Tokenize { file: PathBuf },
    Parse { file: PathBuf },
    TokenizeCss { file: PathBuf },
    ParseCss { file: PathBuf },
    Style { file: PathBuf },
    Layout { file: PathBuf },
    Paint { file: PathBuf, output: PathBuf },
    Open { file: PathBuf },
}

fn main() {
    let command = parse_args();
    match command {
        Command::Tokenize { file } => { /* Phase 1 */ }
        Command::Parse { file }    => { /* Phase 2 */ }
        // ... and so on
    }
}
```

Read the CLI spec at `spec/cli-interface.md` for the exact subcommand names,
flags, and exit code conventions. Key points:

- JSON output goes to **stdout**
- Error messages go to **stderr**
- Exit code `0` means success, `1` means parse error, `2` means file not found
- The `paint` subcommand takes a `-o` flag for the output file path
- If the first argument is a file path (not a recognized subcommand), treat it
  as `open`

Get this skeleton compiling and responding to `--help`. Once you can run
`crowser tokenize somefile.html` and see a placeholder message, you have the
foundation in place and you are ready for Phase 1.

## What's Next

Phase 1: the HTML tokenizer. You will build a state machine that reads raw HTML
character by character and produces a stream of typed tokens. It is the first
real piece of the pipeline, and everything else builds on top of it.

Continue to [01-html-tokenizer.md](01-html-tokenizer.md).
