use comrak::nodes::{AstNode, NodeValue};
use gpui::{FontStyle, FontWeight, HighlightStyle, UnderlineStyle, px};
use std::ops::Range;

use crate::markdown::model::MAX_NESTING_DEPTH;

pub(super) fn collect_inline<'a>(
    node: &'a AstNode<'a>,
    text: &mut String,
    highlights: &mut Vec<(Range<usize>, HighlightStyle)>,
    style: HighlightStyle,
    depth: usize,
) {
    if depth > MAX_NESTING_DEPTH {
        push_highlighted(text, highlights, style, "…");
        return;
    }
    for child in node.children() {
        collect_child(child, text, highlights, style, depth);
    }
}

fn collect_child<'a>(
    child: &'a AstNode<'a>,
    text: &mut String,
    highlights: &mut Vec<(Range<usize>, HighlightStyle)>,
    style: HighlightStyle,
    depth: usize,
) {
    let value = child.data.borrow().value.clone();
    match value {
        NodeValue::Text(t) => push_highlighted(text, highlights, style, &t),
        NodeValue::Strong => collect_inline(child, text, highlights, bold_style(style), depth + 1),
        NodeValue::Emph => collect_inline(child, text, highlights, italic_style(style), depth + 1),
        NodeValue::Link(_) => collect_inline(child, text, highlights, link_style(style), depth + 1),
        NodeValue::Image(_) => push_image(child, text, highlights, style, depth + 1),
        NodeValue::SoftBreak | NodeValue::LineBreak => {
            push_highlighted(text, highlights, style, " ")
        }
        NodeValue::Code(code) => push_highlighted(text, highlights, style, &code.literal),
        _ => collect_inline(child, text, highlights, style, depth + 1),
    }
}

fn bold_style(style: HighlightStyle) -> HighlightStyle {
    HighlightStyle {
        font_weight: Some(FontWeight::BOLD),
        ..style
    }
}

fn italic_style(style: HighlightStyle) -> HighlightStyle {
    HighlightStyle {
        font_style: Some(FontStyle::Italic),
        ..style
    }
}

fn link_style(style: HighlightStyle) -> HighlightStyle {
    HighlightStyle {
        underline: Some(UnderlineStyle {
            thickness: px(1.0),
            ..Default::default()
        }),
        ..style
    }
}

fn push_image<'a>(
    child: &'a AstNode<'a>,
    text: &mut String,
    highlights: &mut Vec<(Range<usize>, HighlightStyle)>,
    style: HighlightStyle,
    depth: usize,
) {
    push_highlighted(text, highlights, style, "[image: ");
    collect_inline(child, text, highlights, style, depth);
    push_highlighted(text, highlights, style, "]");
}

fn push_highlighted(
    text: &mut String,
    highlights: &mut Vec<(Range<usize>, HighlightStyle)>,
    style: HighlightStyle,
    content: &str,
) {
    let start = text.len();
    text.push_str(content);
    if style != HighlightStyle::default() {
        highlights.push((start..text.len(), style));
    }
}
