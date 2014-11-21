extern crate libc;
#[cfg(test)]extern crate test;

use std::string;
#[cfg(test)] use std::io::File;
#[cfg(test)] use test::Bencher;

mod ffi;

#[deriving(Show, PartialEq)]
pub enum Node {
    Document(Vec<Node>),
    BQuote,
    BulletList{tight: bool, items: Vec<Node>},
    OrderedList{tight: bool, start: uint, items: Vec<Node>},
    ListItem,
    FencedCode(string::String, string::String),
    IndentedCode(string::String),
    HTML(string::String),
    Paragraph(Vec<Node>),
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
    Link(string::String, Option<string::String>, Box<Node>),
    Image,
    FirstInline,
    LastInline,
    NotReallyANode,
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
        let level = unsafe { ffi::cmark_node_get_header_level(raw) as uint };
        let content = Node::string_content(child);
        match kind {
            ffi::CMARK_NODE_ATX_HEADER => Node::AtxHeader(level, content),
            ffi::CMARK_NODE_SETEXT_HEADER => Node::SetextHeader(level, content),
            _ => panic!(),
        }
    }

    fn string_content(raw: *mut ffi::cmark_node) -> String {
        unsafe {
            string::raw::from_buf(ffi::cmark_node_get_string_content(raw) as *const u8)
        }
    }

    fn from_raw(raw: *mut ffi::cmark_node) -> Node {
        let kind = unsafe { ffi::cmark_node_get_type(raw) };
        match kind {
            ffi::CMARK_NODE_DOCUMENT => Node::Document(Node::children(raw)),
            ffi::CMARK_NODE_ATX_HEADER | ffi::CMARK_NODE_SETEXT_HEADER => Node::new_header(raw, kind),
            ffi::CMARK_NODE_LIST => Node::new_list(raw),
            ffi::CMARK_NODE_LIST_ITEM => Node::from_raw(unsafe{ffi::cmark_node_first_child(raw)}),
            ffi::CMARK_NODE_PARAGRAPH => Node::Paragraph(Node::children(raw)),
            ffi::CMARK_NODE_STRING => Node::String(Node::string_content(raw)),
            ffi::CMARK_NODE_INDENTED_CODE => Node::IndentedCode(unsafe {
                string::raw::from_buf(ffi::cmark_node_get_string_content(raw) as *const u8)
            }),
            ffi::CMARK_NODE_FENCED_CODE => unsafe {
                Node::FencedCode(string::raw::from_buf(ffi::cmark_node_get_fence_info(raw) as *const u8),
                                 Node::string_content(raw))
            },
            ffi::CMARK_NODE_HTML => Node::HTML(Node::string_content(raw)),
            ffi::CMARK_NODE_LINK => {
                let url = unsafe {
                    string::raw::from_buf(ffi::cmark_node_get_url(raw) as *const u8)
                };
                let title = unsafe {
                    let ptr = ffi::cmark_node_get_title(raw) as *const u8;
                    if ptr.is_null() {
                        None
                    } else {
                        Some(string::raw::from_buf(ptr))
                    }
                };
                let child = unsafe { ffi::cmark_node_first_child(raw) };
                Node::Link(url, title, box Node::from_raw(child))
            },
            _ => Node::NotReallyANode,
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

    pub fn process_line(&mut self, data: &[u8]) {
        unsafe {
            ffi::cmark_process_line(self.raw, data.as_ptr(), data.len() as libc::size_t);
        }
    }

    pub fn finish(&mut self) -> Node {
        let root_raw = unsafe { ffi::cmark_finish(self.raw) };
        Node::from_raw(root_raw)
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

[link](url 'title')
"####;

#[test]
fn parse_document() {
    let doc = Parser::parse_document(DOCUMENT.as_bytes());
    let expected = Node::Document(vec![
        Node::AtxHeader(2, "Header".to_string()),
        Node::BulletList{tight: true,
                         items: vec![Node::Paragraph(vec![Node::String("Item 1".to_string())]),
                                     Node::Paragraph(vec![Node::String("Item 2".to_string())])]},
        Node::OrderedList{tight: false, start: 2,
                          items: vec![Node::Paragraph(vec![Node::String("Item 1".to_string())]),
                                      Node::Paragraph(vec![Node::String("Item 2".to_string())])]},
        Node::IndentedCode("code\n".to_string()),
        Node::FencedCode("lang".to_string(), "fenced\n".to_string()),
        Node::HTML("<div>html</div>\n".to_string()),
        Node::Paragraph(
            vec![Node::Link("url".to_string(), Some("title".to_string()),
                            box Node::String("link".to_string()))])
            ]);
    assert_eq!(expected, doc);
}

#[bench]
fn parse_progit(b: &mut Bencher) {
    let doc_contents = File::open(&Path::new("progit.md")).unwrap().read_to_end().unwrap();
    b.iter(|| {
        test::black_box(Parser::parse_document(doc_contents.as_slice()));
    });
}
