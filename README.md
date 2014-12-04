rust-cmark
==========

Rust bindings for the CommonMark
[reference C implementation](https://github.com/jgm/CommonMark). It
post-processes the output of the C implementation and returns an AST.

Installation
------------

Add

    [dependencies.cmark]
    git = "https://github.com/carlosmn/rust-cmark"

to your `Cargo.toml` file.

Usage
------

For now, only parsing a string and getting the AST is supported.

```rust
extern crate cmark;

fn main() {
    let doc_contents = "## Header\n* Item 1\n* Item 2\n";
    let ast = cmark::Parser::parse_document(doc_contents.as_bytes());
    println!("{}", ast);

}
```

will print out

    Document([AtxHeader(2, [String(Header)]), BulletList { tight: true, items: [ListItem([Paragraph([String(Item 1)])]), ListItem([Paragraph([String(Item 2)])])] }])

which is not the prettiest of outputs, but it's an AST which you can
use to generate whatever output you want out of your tool.

Tests
------

The tests rely on two files, `leakcheck.md` and `progit.md`. These are
not included in this repository but are part of the cmark tests, which
you can generate from its repository.
