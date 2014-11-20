extern crate libc;

use std::string;

mod ffi;

#[deriving(Show, PartialEq)]
pub enum Node {
    Document(Vec<Node>),
    BQuote,
    BulletList{tight: bool, items: Vec<Node>},
    OrderedList{tight: bool, start: uint, items: Vec<Node>},
    ListItem,
    FencedCode,
    IndentedCode,
    HTML,
    Paragraph,
    AtxHeader(uint, string::String),
    SetextHeader(uint, string::String),
    HRule,
    ReferenceDef,
    FirstBlock,
    LastBlock,
    String(string::String),
    Softbreak,
    Linebreak,
    InlineCode,
    InlineHTML,
    Emph,
    Strong,
    Link{url: String, title: String},
    Image,
    FirstInline,
    LastInline,
}

impl Node {
    fn children(raw: *mut ffi::cmark_node) -> Vec<Node> {
        let mut vec = Vec::new();
        unsafe {
            let mut ptr = ffi::cmark_node_first_child(raw);
            while !ptr.is_null() {
                vec.push(Node::from_raw(ptr));
                ptr = ffi::cmark_node_next(ptr);
            }
        }
        vec
    }

    fn new_document(raw: *mut ffi::cmark_node) -> Node {
        Node::Document(Node::children(raw))
    }

    fn new_list(raw: *mut ffi::cmark_node) -> Node {
        let list_type = unsafe { ffi::cmark_node_get_list_type(raw) };
        let tight = unsafe { ffi::cmark_node_get_list_tight(raw) } != 0;
        match list_type {
            ffi::CMARK_ORDERED_LIST => {
                let start = unsafe { ffi::cmark_node_get_list_start(raw) } as uint;
                Node::OrderedList{tight: tight, start: start, items: Node::children(raw)}
            },
            ffi::CMARK_BULLET_LIST => {
                Node::BulletList{tight: tight, items: Node::children(raw)}
            },
            _ => panic!(),
        }
    }

    fn new_header(raw: *mut ffi::cmark_node, kind: u32) -> Node {
        let child = unsafe { ffi::cmark_node_first_child(raw)};
        let (level, content) = unsafe {
            (ffi::cmark_node_get_header_level(raw) as uint,
             string::raw::from_buf(ffi::cmark_node_get_string_content(child) as *const u8))
        };
        match kind {
            ffi::CMARK_NODE_ATX_HEADER => Node::AtxHeader(level, content),
            ffi::CMARK_NODE_SETEXT_HEADER => Node::SetextHeader(level, content),
            _ => panic!(),
        }
    }

    fn from_raw(raw: *mut ffi::cmark_node) -> Node {
        let kind = unsafe { ffi::cmark_node_get_type(raw) };
        match kind {
            ffi::CMARK_NODE_DOCUMENT => Node::new_document(raw),
            ffi::CMARK_NODE_ATX_HEADER | ffi::CMARK_NODE_SETEXT_HEADER => Node::new_header(raw, kind),
            ffi::CMARK_NODE_LIST => Node::new_list(raw),
            _ => Node::LastBlock,
            //_ => panic!(),
        }
    }
}

pub struct Parser {
    raw: *mut ffi::cmark_doc_parser,
}

impl Parser {
    pub fn new() -> Parser {
        Parser { raw: unsafe { ffi::cmark_new_doc_parser() } }
    }

    pub fn parse_document(data: &[u8]) -> Node {
        unsafe {
            let raw = ffi::cmark_parse_document(data.as_ptr(), data.len() as libc::size_t);
            let node = Node::from_raw(raw);
            ffi::cmark_free_nodes(raw);
            node
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
    let children = match doc {
        Node::Document(ref children) => children,
        _ => panic!(),
    };
    assert_eq!(Node::AtxHeader(2, "Header".to_string()), children[0]);
}
