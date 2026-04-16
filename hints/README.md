# Hints

These hints are **optional**. The [specs](../spec/) are the source of truth for what your engine should do. Hints only suggest how you might implement it in Rust.

## What hints are

Each phase directory contains a single `hints.md` file with:

- **Suggested types** -- Rust structs and enums that fit the problem well
- **Architecture notes** -- how you might structure modules and functions
- **Rust tips** -- language-specific advice on ownership, serialization, error handling
- **Common pitfalls** -- mistakes that Rust beginners (and non-beginners) tend to make

The hints describe one possible approach. Following them will give you a working engine, but there are many valid designs. A different set of types, a different module layout, a different parsing strategy -- all fine, as long as your output matches the spec.

## How to use them

1. Read the phase spec first.
2. Try to design your own types and architecture.
3. Write some code. Get stuck if you are going to get stuck.
4. **Then** look at the hints for that phase.

If you look at hints before trying on your own, you will learn less. The point of this project is to make design decisions, not to copy a reference implementation.

## What hints are not

- They are not a walkthrough. They do not hold your hand through every function.
- They are not the only way. If your approach differs, that is expected.
- They are not tested code. The types shown are suggestions, not copy-paste solutions. You will need to adapt them.
