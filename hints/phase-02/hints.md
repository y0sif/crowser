# Phase 2 Hints: HTML Parser

## Suggested Types

```rust
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum Node {
    Element {
        tag: String,
        attributes: HashMap<String, String>,
        children: Vec<Node>,
    },
    Text {
        content: String,
    },
    Comment {
        content: String,
    },
    Document {
        children: Vec<Node>,
    },
}
```

Just like Phase 1, `#[serde(tag = "type")]` gives you the internally tagged JSON that the spec requires.

## Architecture

A `Parser` struct that walks through the token list:

```rust
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { ... }
    pub fn parse(&mut self) -> Node { ... }
}
```

The key technique is a **stack of child lists**. Each entry in the stack is a `Vec<Node>` that collects the children for the currently open element.

```
Algorithm:
1. Initialize the stack with one empty Vec<Node> (the root level).
2. For each token:
   - StartTag: push a new empty Vec<Node> onto the stack.
   - EndTag: pop the top Vec<Node>, wrap it in an Element node, and
     push that Element onto the new top of the stack.
   - SelfClosingTag: create an Element with children: [] and push it
     onto the current top of the stack.
   - Text: create a Text node and push it onto the current top.
   - Comment: create a Comment node and push it onto the current top.
3. After all tokens: the stack should have one entry (the root level).
4. If the root level has exactly one node, output that node.
   If it has multiple nodes, wrap them in a Document node.
```

You need to carry the tag name and attributes from the `StartTag` token through to the `EndTag`. One approach: also maintain a parallel stack of `(tag_name, attributes)` tuples. When you see an `EndTag`, pop both stacks and assemble the `Element` node.

## Rust Tips

**Ownership of tokens.** The parser consumes the token list. You can iterate with index-based access (`self.tokens[self.pos]`) and clone the data you need, or you can take ownership by converting the `Vec<Token>` into an iterator. Index-based access is simpler for this use case.

**Avoid deep recursion.** A stack-based approach (using a `Vec` as your explicit stack) avoids potential stack overflow on deeply nested HTML. It also maps cleanly to the iterative token processing loop.

**The Document wrapper.** The spec says: if there is exactly one root-level node, output it directly. If there are multiple root-level nodes, wrap them in a `Document` node. Check `root_nodes.len()` at the end. This matters for the JSON structure -- a single-root document should not have the extra `Document` wrapper.

**HashMap ordering in JSON.** The `serde_json` crate does not guarantee key order in `HashMap` serialization by default. If you need deterministic output (helpful for debugging), consider using `serde_json::Map` or `indexmap::IndexMap` with the `serde` feature. The test runner compares structurally, so key order does not matter for passing tests, but stable output is easier to read.

## Common Pitfalls

**The stack must start with one frame.** If you forget to push the initial `Vec<Node>` for the root level, the first `EndTag` will pop an empty stack and panic.

**Mismatched tag names.** The spec says input is well-formed, so you do not need to handle mismatches. But adding a debug assertion (`assert_eq!(end_tag_name, expected_name)`) will catch bugs in your tokenizer early.

**SelfClosingTag handling.** Self-closing tags do not push onto the stack. They create an Element with `children: []` and immediately add it as a child of the current frame. A common mistake is treating them like `StartTag` and pushing a new frame, which corrupts the stack.

**Empty children arrays.** An element with no children must serialize as `"children": []`, not have the field omitted. The `Vec<Node>` handles this naturally with serde.

**Text nodes are leaf nodes.** They do not have a `children` field in the output. The serde enum variant handles this since `Text` only has `content`. If you use a struct-based approach instead of an enum, make sure Text nodes do not include `children` in the JSON.
