# crowser: Build a Browser Engine from Scratch

This tutorial guides you through building a real browser engine in Rust -- one
that reads HTML and CSS, resolves styles, computes layout, and paints pixels to
a window on your screen. It is split into eight phases. Each phase produces a
working piece of the pipeline that you can inspect and test independently.

## What You're Building

A browser engine is a pipeline. Raw text goes in one end; pixels come out the
other. Your engine, `crowser`, will implement every stage of that pipeline:

```
HTML text ──> Tokenizer ──> Tokens ──> Parser ──> DOM Tree
                                                      |
CSS text  ──> Tokenizer ──> Tokens ──> Parser ──> Stylesheet
                                                      |
                              Style Resolution <──────┘
                                    |
                              Styled Tree
                                    |
                              Layout Engine
                                    |
                              Layout Tree
                                    |
                               Painter
                                    |
                              Pixel Buffer ──> Window
```

Each box in this diagram corresponds to a phase. Each phase has a CLI
subcommand that exposes the intermediate representation so you can see exactly
what your engine is doing at every step:

| Phase | Subcommand       | What It Exposes                |
|:-----:|------------------|--------------------------------|
|   1   | `tokenize`       | HTML token stream (JSON)       |
|   2   | `parse`          | DOM tree (JSON)                |
|   3   | `tokenize-css`   | CSS token stream (JSON)        |
|   4   | `parse-css`      | Stylesheet structure (JSON)    |
|   5   | `style`          | Styled DOM tree (JSON)         |
|   6   | `layout`         | Layout tree with positions (JSON) |
|   7   | `paint`          | Rendered pixels (PNG)          |
|   8   | `open`           | Live window on screen          |

This is not a simplified toy. By the end, you will have written every line of
code that stands between `<html>` and the pixels on your monitor.

## How to Use This Tutorial

Every phase follows the same loop:

1. **Read the guide** for the phase (`guide/XX-topic.md`). This is the chapter
   you are reading now -- it teaches the concepts, explains the approach, and
   walks through the key ideas. It does not give you the code.

2. **Read the spec** (`spec/phase-XX/SPEC.md`). This is the contract. It
   defines the exact input format, output format, and edge cases your
   implementation must handle. If your output matches the spec, you have built
   it correctly.

3. **Implement it your way.** The guide and spec tell you *what* to build, not
   *how*. Design your data structures. Choose your approach. Write the code.

4. **Run the tests:**
   ```
   cargo build
   ./tools/test-runner.sh <phase> ./target/debug/crowser
   ```
   The test runner feeds your binary real input files and compares the output
   against expected results. JSON is compared structurally (key order and
   formatting do not matter).

5. **All tests pass?** Move to the next phase.

6. **Stuck?** Check the hints directory (`hints/phase-XX/`) for Rust-specific
   guidance, suggested data structures, and implementation nudges. Use them as
   little or as much as you need.

## Prerequisites

You should be comfortable with:

- **Rust fundamentals.** You know how to use `cargo`, you understand ownership
  and borrowing, and you can work with enums, structs, `Vec`, `String`, and
  pattern matching. You do not need to be an expert -- this project will make
  you better at Rust by doing.

- **HTML and CSS at a user level.** You have written HTML and CSS before. You
  know what tags, attributes, selectors, and properties are. You do not need to
  know the formal specifications.

- **The command line.** You can navigate directories, run commands, and read
  error messages.

If you are new to Rust, work through the first few chapters of [The Rust
Book](https://doc.rust-lang.org/book/) before starting. The concepts here build
on top of Rust's core features, and fighting the language while also learning
browser internals is a rough time.

## The Pipeline in Detail

Here is how data flows through your engine, and which phase handles each
transformation:

```
                    Phase 1                Phase 2
  "<div>hi</div>" ───────> [StartTag,  ───────> Element("div")
                            Text,                  |
                            EndTag]               Text("hi")

                    Phase 3                Phase 4
  "p { color: red }" ─────> [Ident,    ───────> Rule {
                              OpenBrace,           selectors: [p],
                              Ident,               declarations: [color: red]
                              Colon,             }
                              Ident,
                              Semicolon,
                              CloseBrace]

                    Phase 5
  DOM + Stylesheet ───────> Styled Tree (each node carries its resolved styles)

                    Phase 6
  Styled Tree ────────────> Layout Tree (each box has x, y, width, height)

                    Phase 7
  Layout Tree ────────────> Pixel Buffer (a PNG image)

                    Phase 8
  Pixel Buffer ───────────> OS Window on screen
```

Each subcommand is a window into one stage. When you run `crowser tokenize`,
you see the raw token stream. When you run `crowser parse`, you see the DOM
tree that was built from those tokens. When you run `crowser paint`, you get a
PNG. This makes debugging straightforward: if the final output is wrong, you
can inspect every intermediate step to find where things diverged.

## A Note on Scope

Real browser engines are millions of lines of code and handle decades of web
standards, quirks modes, and backward compatibility. crowser does not attempt
that. It handles a well-defined subset of HTML and CSS -- enough to render real
pages with styled, positioned elements and visible output. The specs for each
phase clearly state what is in scope and what is not.

This is still a serious engineering project. You will build a tokenizer, a
parser, a style resolver, a layout engine, and a renderer. You will wrestle
with state machines, tree construction, the CSS box model, and coordinate
geometry. You will come out the other side understanding how browsers actually
work, not as an abstraction, but because you built one.

Start with the introduction: [00-introduction.md](00-introduction.md).
