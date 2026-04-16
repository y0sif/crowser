# Phase 2: HTML Parser

## What You'll Learn

- What parsing is and how it differs from tokenization
- The DOM (Document Object Model) as a tree data structure
- How to use a stack to build a tree from a flat token stream
- How to handle self-closing tags, comments, text nodes, and multiple roots
- How to serialize a tree as JSON

## From Tokens to Trees

In Phase 1 you built a tokenizer that breaks HTML into a flat list of tokens.
But HTML is not flat -- it is hierarchical. Tags nest inside other tags.
Elements have children. The structure is a tree, and a flat list does not
capture that.

Consider these tokens:

```
StartTag("div") -> StartTag("p") -> Text("hello") -> EndTag("p") -> EndTag("div")
```

The token list tells you "a div opens, then a p opens, then there is text, then
p closes, then div closes." But the *structure* is:

```
div
 └── p
      └── "hello"
```

The `p` element is a *child* of `div`. The text node is a child of `p`. The
parser's job is to reconstruct this parent-child relationship from the flat
token stream.

## The DOM

The DOM is the tree data structure that represents an HTML document. Each node
in the tree is one of:

- **Element** -- has a tag name, attributes, and an ordered list of children
- **Text** -- contains a string of text content, no children
- **Comment** -- contains the comment text, no children
- **Document** -- a synthetic root node, used only when the HTML has multiple
  root-level nodes

In Rust, this maps naturally to an enum:

```rust
enum Node {
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

## The Stack Algorithm

The key insight is that you maintain a *stack* of open elements. The element on
top of the stack is the "current parent" -- any new node you create becomes a
child of this element.

Here is the algorithm:

1. Start with an empty stack and an empty list of root nodes
2. For each token in the stream:
   - **StartTag**: create a new Element node (no children yet). Push it onto
     the stack. It is now the current parent.
   - **EndTag**: pop the top element from the stack. It is now complete. Add it
     as a child of the *new* top of the stack. If the stack is empty, add it to
     the root nodes list.
   - **SelfClosingTag**: create an Element node with an empty children list.
     Add it as a child of the current parent (top of stack). Do *not* push it
     onto the stack -- it has no children.
   - **Text**: create a Text node. Add it as a child of the current parent. If
     the stack is empty, add it to the root nodes list.
   - **Comment**: create a Comment node. Same rules as Text.
3. After all tokens are processed, if there is exactly one root node, output
   that node. If there are multiple root nodes, wrap them in a Document node.

## Walkthrough: Building a Tree

Let's trace through `<div><p>hello</p><br/></div>` step by step.

**Tokens produced by Phase 1:**
```
[StartTag("div"), StartTag("p"), Text("hello"), EndTag("p"),
 SelfClosingTag("br"), EndTag("div")]
```

**Processing:**

| Token               | Stack (top = right) | Action                              | Root Nodes |
|----------------------|---------------------|-------------------------------------|------------|
| StartTag("div")     | [div]               | Create div, push onto stack         | []         |
| StartTag("p")       | [div, p]            | Create p, push onto stack           | []         |
| Text("hello")       | [div, p]            | Create text, add as child of p      | []         |
| EndTag("p")         | [div]               | Pop p, add as child of div          | []         |
| SelfClosingTag("br")| [div]               | Create br, add as child of div      | []         |
| EndTag("div")       | []                  | Pop div, stack empty -> add to roots| [div]      |

**Resulting tree:**
```
div
 ├── p
 │    └── "hello"
 └── br
```

Since there is exactly one root node, the output is the `div` element directly
(no Document wrapper).

## The Document Wrapper

If the HTML has multiple root-level nodes, you wrap them in a Document node.
This happens when the input looks like:

```html
<h1>Title</h1>
<p>Body</p>
```

These produce two root elements. The output becomes:

```json
{
  "type": "Document",
  "children": [
    { "type": "Element", "tag": "h1", "attributes": {}, "children": [
        { "type": "Text", "content": "Title" }
    ]},
    { "type": "Element", "tag": "p", "attributes": {}, "children": [
        { "type": "Text", "content": "Body" }
    ]}
  ]
}
```

Root-level text nodes and comment nodes also count. If the input is just
`hello`, with no tags, the output is a single Text node (no Document wrapper
needed because there is only one root node).

## Implementation Considerations

### Popping and Attaching

The trickiest part of the algorithm is the relationship between popping an
element and attaching it to its parent. When you pop an element from the stack,
it needs to become a child of whatever is now on top of the stack. If the stack
is empty after the pop, the element is a root node.

There are several ways to model this in Rust. One approach: the stack holds
partially-constructed Element nodes. When you pop, you move the popped node
into the `children` Vec of the new stack top. Another approach: build the tree
using indices or `Rc<RefCell<...>>`. Choose what feels natural to you -- the
spec does not care about your internal representation, only the JSON output.

### A simpler approach

You might find it easier to think of it differently. Instead of pushing
partially-constructed nodes, you can push indices or use a different data
structure. Here is one concrete pattern:

- Keep a `Vec<Node>` as your node storage
- The stack holds indices into this Vec
- When you pop, you know the parent's index (new top of stack) and can push the
  completed child into the parent's children

The right approach depends on your comfort level with Rust's ownership model.
If you find yourself fighting the borrow checker, step back and think about
whether you can restructure to avoid shared mutable references.

### Whitespace Text Nodes

Just like in Phase 1, whitespace text nodes are preserved. If the input has
newlines and spaces between tags, those become Text nodes in the tree. Do not
filter them out.

```html
<div>
  <p>hello</p>
</div>
```

The tree for this has whitespace text nodes:

```
div
 ├── "\n  "       (text node)
 ├── p
 │    └── "hello"
 └── "\n"          (text node)
```

### Single Root Detection

After processing all tokens, look at your root nodes list:

- **One node:** output it directly
- **Zero nodes:** output `{ "type": "Document", "children": [] }`
- **Multiple nodes:** wrap in a Document node

This is a simple check at the very end of parsing.

## JSON Serialization

The output format uses `"type"` as a discriminator:

```json
{
  "type": "Element",
  "tag": "div",
  "attributes": { "class": "main" },
  "children": [
    { "type": "Text", "content": "hello" }
  ]
}
```

If you are using `serde`, you can derive `Serialize` with a tag attribute to
get this format. The exact approach depends on how you model your Node enum.
The spec at `spec/phase-02-html-parser/SPEC.md` shows the complete format for
every node type.

Note that Element nodes use `"tag"` for the tag name (not `"name"` as in the
tokenizer). Comment and Text nodes have `"content"`. Document nodes have only
`"type"` and `"children"`. Get these field names right -- the test runner
compares them exactly.

## Edge Cases

**Self-closing tags in nested context.** A self-closing tag like `<br/>` inside
a `<div>` becomes a child of the div but does not push onto the stack. Make
sure you do not accidentally try to pop it later.

**Adjacent text nodes.** Can the tokenizer produce two Text tokens in a row?
In your Phase 1 implementation, probably not -- you accumulate text until you
hit a `<`. But if it did happen, the parser should add them as separate Text
children. Do not worry about merging.

**Empty elements.** `<div></div>` produces an Element with an empty children
array `[]`. Make sure you do not omit the children field.

**Comments at root level.** A comment outside any element goes into the root
nodes list, just like a root-level element or text node.

**Mismatched tags.** The spec says input is assumed to be well-formed: every
StartTag has a matching EndTag, and nesting is correct. You do not need to
implement error recovery for this phase. If you want to be defensive, you can
check that the EndTag name matches the top of the stack and handle mismatches
gracefully, but it is not required.

## Testing

The spec at `spec/phase-02-html-parser/SPEC.md` defines the exact output
format and rules. The test suite covers simple trees, nested elements,
attributes, self-closing tags, whitespace preservation, comments, mixed
content, and deeply nested structures.

```
cargo build
./tools/test-runner.sh 2 ./target/debug/crowser
```

Stuck? Check `hints/phase-02/` for implementation guidance.

## The CLI Subcommand

```
crowser parse <file.html>
```

Reads the file, tokenizes it (reusing your Phase 1 tokenizer), parses the
tokens into a DOM tree, serializes the tree as JSON, and prints it to stdout.
Same exit code conventions as Phase 1.

## What You'll Have at the End

A parser that takes a token stream and produces a DOM tree. A CLI subcommand
that pipes an HTML file through your tokenizer and parser and outputs the tree
as JSON. Two of the eight pipeline stages are now complete -- you can go from
raw HTML text to a structured document tree.

## What's Next

Phase 3: the CSS tokenizer. CSS has a completely different grammar from HTML --
different token types, different rules, different edge cases. But the technique
is the same: a state machine that reads characters and emits tokens. Having
built one tokenizer already, the second one will go faster.

Continue to [03-css-tokenizer.md](03-css-tokenizer.md).
