extern crate libc;
#[cfg(test)]extern crate test;

use std::string;
#[cfg(test)] use std::io::File;
#[cfg(test)] use test::Bencher;

mod ffi;

#[deriving(Show, PartialEq)]
pub enum Node {
    Document(Vec<Node>),
    BQuote(Vec<Node>),
    BulletList{tight: bool, items: Vec<Node>},
    OrderedList{tight: bool, start: uint, items: Vec<Node>},
    ListItem(Vec<Node>),
    FencedCode(string::String, string::String),
    IndentedCode(string::String),
    HTML(string::String),
    Paragraph(Vec<Node>),
    AtxHeader(uint, Vec<Node>),
    SetextHeader(uint, Vec<Node>),
    HRule,
    ReferenceDef,
    String(string::String),
    Softbreak,
    Linebreak,
    InlineCode(string::String),
    InlineHTML(string::String),
    Emph(Vec<Node>),
    Strong(Vec<Node>),
    Link(Option<string::String>, Option<string::String>, Vec<Node>),
    Image(Option<string::String>, Option<string::String>, Vec<Node>),
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
        let level = unsafe { ffi::cmark_node_get_header_level(raw) as uint };
        let content = Node::children(raw);
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
            ffi::CMARK_NODE_LIST_ITEM => Node::ListItem(Node::children(raw)),
            ffi::CMARK_NODE_PARAGRAPH => Node::Paragraph(Node::children(raw)),
            ffi::CMARK_NODE_STRING => Node::String(Node::string_content(raw)),
            ffi::CMARK_NODE_INLINE_CODE => Node::InlineCode(Node::string_content(raw)),
            ffi::CMARK_NODE_INLINE_HTML => Node::InlineHTML(Node::string_content(raw)),
            ffi::CMARK_NODE_EMPH => Node::Emph(Node::children(raw)),
            ffi::CMARK_NODE_STRONG => Node::Strong(Node::children(raw)),
            ffi::CMARK_NODE_BQUOTE => Node::BQuote(Node::children(raw)),
            ffi::CMARK_NODE_SOFTBREAK => Node::Softbreak,
            ffi::CMARK_NODE_LINEBREAK => Node::Linebreak,
            ffi::CMARK_NODE_HRULE => Node::HRule,
            ffi::CMARK_NODE_REFERENCE_DEF => Node::ReferenceDef,
            ffi::CMARK_NODE_INDENTED_CODE => Node::IndentedCode(unsafe {
                string::raw::from_buf(ffi::cmark_node_get_string_content(raw) as *const u8)
            }),
            ffi::CMARK_NODE_FENCED_CODE => unsafe {
                Node::FencedCode(string::raw::from_buf(ffi::cmark_node_get_fence_info(raw) as *const u8),
                                 Node::string_content(raw))
            },
            ffi::CMARK_NODE_HTML => Node::HTML(Node::string_content(raw)),
            ffi::CMARK_NODE_LINK | ffi::CMARK_NODE_IMAGE => {
                let url = unsafe {
                    let ptr = ffi::cmark_node_get_url(raw) as *const u8;
                    if ptr.is_null() {
                        None
                    } else {
                        Some(string::raw::from_buf(ptr))
                    }
                };
                let title = unsafe {
                    let ptr = ffi::cmark_node_get_title(raw) as *const u8;
                    if ptr.is_null() {
                        None
                    } else {
                        Some(string::raw::from_buf(ptr))
                    }
                };
                if kind == ffi::CMARK_NODE_LINK {
                    Node::Link(url, title, Node::children(raw))
                } else {
                    Node::Image(url, title, Node::children(raw))
                }
            },
            _ => panic!("unimplemented {}", kind),
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
        Node::AtxHeader(2, vec![Node::String("Header".to_string())]),
        Node::BulletList{tight: true,
                         items: vec![Node::ListItem(vec![Node::Paragraph(vec![Node::String("Item 1".to_string())])]),
                                     Node::ListItem(vec![Node::Paragraph(vec![Node::String("Item 2".to_string())])])]},
        Node::OrderedList{tight: false, start: 2,
                          items: vec![Node::ListItem(vec![Node::Paragraph(vec![Node::String("Item 1".to_string())])]),
                                      Node::ListItem(vec![Node::Paragraph(vec![Node::String("Item 2".to_string())])])]},
        Node::IndentedCode("code\n".to_string()),
        Node::FencedCode("lang".to_string(), "fenced\n".to_string()),
        Node::HTML("<div>html</div>\n".to_string()),
        Node::Paragraph(
            vec![Node::Link(Some("url".to_string()), Some("title".to_string()),
                            vec![Node::String("link".to_string())])])
            ]);
    assert_eq!(expected, doc);
}

#[test]
fn leak_check() {
    let doc_contents = File::open(&Path::new("leakcheck.md")).unwrap().read_to_end().unwrap();
    test::black_box(Parser::parse_document(doc_contents.as_slice()));
}

#[bench]
fn parse_progit(b: &mut Bencher) {
    let doc_contents = File::open(&Path::new("progit.md")).unwrap().read_to_end().unwrap();
    b.iter(|| {
        let doc = Parser::parse_document(doc_contents.as_slice());
        test::black_box(doc);
    });
}

#[bench]
fn bench_leakcheck(b: &mut Bencher) {
    let doc_contents = File::open(&Path::new("leakcheck.md")).unwrap().read_to_end().unwrap();
    b.iter(|| {
        let doc = Parser::parse_document(doc_contents.as_slice());
        test::black_box(doc);
    });
}
