extern crate libc;

mod ffi;

#[deriving(Show, PartialEq)]
pub enum Node {
    Document(DocumentNode),
    BQuote,
    List,
    ListItem,
    FencedCode,
    IndentedCode,
    HTML,
    Paragraph,
    AtxHeader{level: uint},
    SetextHeader{level: uint},
    HRule,
    ReferenceDef,
    FirstBlock,
    LastBlock,
    String(String),
    Softbreak,
    Linebreak,
    InlineCode,
    InlineHTML,
    Emph,
    Strong,
    Link,
    Image,
    FirstInline,
    LastInline,
}

impl Node {
    fn level(raw: *mut ffi::cmark_node) -> uint {
        unsafe {
            ffi::cmark_node_get_header_level(raw) as uint
        }
    }

    fn from_raw(raw: *mut ffi::cmark_node) -> Node {
        match unsafe { ffi::cmark_node_get_type(raw) } {
            ffi::CMARK_NODE_DOCUMENT => Node::Document(DocumentNode::from_raw(raw)),
            ffi::CMARK_NODE_ATX_HEADER => Node::AtxHeader{level: Node::level(raw)},
            ffi::CMARK_NODE_SETEXT_HEADER => Node::SetextHeader{level: Node::level(raw)},
            _ => panic!(),
        }
    }
}

#[deriving(Show)]
pub struct DocumentNode {
    raw: *mut ffi::cmark_node
}

impl DocumentNode {
    pub fn from_raw(raw: *mut ffi::cmark_node) -> DocumentNode {
        assert!(unsafe { ffi::cmark_node_get_type(raw) } == ffi::CMARK_NODE_DOCUMENT);
        DocumentNode { raw: raw }
    }

    pub fn first_child(&self) -> Node {
        Node::from_raw(unsafe { ffi::cmark_node_first_child(self.raw ) })
    }
}

impl PartialEq for DocumentNode {
    fn eq(&self, other: &DocumentNode) -> bool {
        self.raw == other.raw
    }
}

pub struct Parser {
    raw: *mut ffi::cmark_doc_parser,
}

impl Parser {
    pub fn new() -> Parser {
        Parser { raw: unsafe { ffi::cmark_new_doc_parser() } }
    }

    pub fn parse_document(data: &[u8]) -> DocumentNode {
        unsafe {
            DocumentNode {raw: ffi::cmark_parse_document(data.as_ptr(), data.len() as libc::size_t) }
        }
    }
}

impl Drop for Parser {
    fn drop(&mut self) {
        unsafe { ffi::cmark_free_doc_parser(self.raw) }
    }
}

#[cfg(test)]
const DOCUMENT: &'static str =
r####"## Header
* Item 1
* Item 2

2. Item 1

3. Item 2

    code

``` lang
fenced
```

    <div>html</div>

[link url](url 'title')
"####;

#[test]
fn parse_document() {
    let doc = Parser::parse_document(DOCUMENT.as_bytes());
    let child = doc.first_child();
    assert_eq!(Node::AtxHeader{ level: 2 }, child);
}
