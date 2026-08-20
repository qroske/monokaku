use comrak::nodes::AstNode;
use gpui::HighlightStyle;

use super::inline::collect_inline;
use crate::markdown::model::{Block, Line};

pub(super) fn nesting_limit_block(indent_level: usize) -> Block {
    Block::Text(Line {
        text: "…（ネストが深すぎるため以下省略）".to_string(),
        highlights: Vec::new(),
        heading_level: None,
        list_marker: None,
        indent_level,
    })
}

pub(super) fn marker_only_line(marker: String, indent_level: usize) -> Line {
    Line {
        text: String::new(),
        highlights: Vec::new(),
        heading_level: None,
        list_marker: Some(marker),
        indent_level,
    }
}

pub(super) fn collect_text_line<'a>(
    node: &'a AstNode<'a>,
    heading_level: Option<u8>,
    list_marker: Option<String>,
    indent_level: usize,
) -> Line {
    let mut text = String::new();
    let mut highlights = Vec::new();
    collect_inline(
        node,
        &mut text,
        &mut highlights,
        HighlightStyle::default(),
        0,
    );
    Line {
        text,
        highlights,
        heading_level,
        list_marker,
        indent_level,
    }
}

pub(super) fn push_text_line<'a>(
    node: &'a AstNode<'a>,
    heading_level: Option<u8>,
    list_marker: Option<String>,
    indent_level: usize,
    push_block: &mut dyn FnMut(Block),
) {
    let line = collect_text_line(node, heading_level, list_marker, indent_level);
    if !line.text.is_empty() || line.list_marker.is_some() {
        push_block(Block::Text(line));
    }
}
