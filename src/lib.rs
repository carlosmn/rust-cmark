#[crate_name = "cmark"]
#[crate_type = "dylib"]
extern crate libc;

use std::mem;

mod ffi;

#[repr(u32)]
pub enum NodeType {
    Document,
    BQuote,
    List,
    ListItem,
    FencedCode,
    IndentedCode,
    HTML,
    Paragraph,
    AtxHeader,
    SettextHeader,
    HRule,
    ReferenceDef,
    FirstBlock,
    LastBlock,
    String,
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

pub struct Node {
    raw: *mut ffi::cmark_node
}

impl Node {
    pub fn kind(&self) -> NodeType {
        unsafe { mem::transmute(ffi::cmark_node_get_type(self.raw)) }
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
            Node {raw: ffi::cmark_parse_document(data.as_ptr(), data.len() as libc::size_t) }
        }
    }
}

impl Drop for Parser {
    fn drop(&mut self) {
        unsafe { ffi::cmark_free_doc_parser(self.raw) }
    }
}

#[test]
fn it_works() {
}
