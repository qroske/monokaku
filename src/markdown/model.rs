use gpui::HighlightStyle;
use std::ops::Range;

pub struct Line {
    pub text: String,
    pub highlights: Vec<(Range<usize>, HighlightStyle)>,
    pub heading_level: Option<u8>,
    pub list_marker: Option<String>,
    pub indent_level: usize,
}

pub enum Block {
    Text(Line),
    Code(String, usize),
    Table(Vec<(bool, Vec<Line>)>, usize),
    Quote(Vec<Block>, usize),
    Rule(usize),
}

pub const MAX_NESTING_DEPTH: usize = 64;
