use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub frontmatter: HashMap<String, FrontmatterValue>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FrontmatterValue {
    Str(String),
    List(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading(Heading),
    Paragraph(Paragraph),
    List(List),
    CodeBlock(CodeBlock),
    QuoteBlock(QuoteBlock),
    HorizontalRule,
    BlankLine,
    HtmlBlock(String), // raw HTML/JSX block
}

#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    pub level: u8,
    pub content: Vec<Inline>,
    pub tags: Vec<String>,
    pub todo_keyword: Option<String>,
    pub priority: Option<char>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    pub content: Vec<Inline>,
    pub hard_line_break: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct List {
    pub kind: ListKind,
    pub items: Vec<ListItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListKind {
    Unordered,
    Ordered,
    Description,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub content: Vec<Block>,
    pub children: Vec<ListItem>,
    pub checkbox: Option<bool>, // Some(true) = checked, Some(false) = unchecked, None = no checkbox
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlock {
    pub language: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteBlock {
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text(String),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Underline(Vec<Inline>),
    StrikeThrough(Vec<Inline>),
    Code(String),     // inline code
    Verbatim(String), // =verbatim= (escaped)
    Link(Link),
    Image(Image),
    LineBreak,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub url: String,
    pub text: Vec<Inline>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub url: String,
    pub alt_text: Option<String>,
}
