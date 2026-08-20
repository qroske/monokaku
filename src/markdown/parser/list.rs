use comrak::nodes::{AstNode, ListDelimType, ListType, NodeList, NodeValue};

use super::block::parse_block;
use super::line::{marker_only_line, nesting_limit_block, push_text_line};
use crate::markdown::model::{Block, MAX_NESTING_DEPTH};

pub(super) fn parse_list<'a>(
    node: &'a AstNode<'a>,
    list: NodeList,
    indent_level: usize,
    depth: usize,
    push_block: &mut dyn FnMut(Block),
) {
    if depth > MAX_NESTING_DEPTH {
        push_block(nesting_limit_block(indent_level));
        return;
    }
    let mut ordinal = list.start;
    for item in node.children() {
        let marker_text = next_marker(&list, &mut ordinal);
        process_item(item, marker_text, indent_level, depth, push_block);
    }
}

fn next_marker(list: &NodeList, ordinal: &mut usize) -> String {
    match list.list_type {
        ListType::Bullet => "• ".to_string(),
        ListType::Ordered => {
            let delimiter = match list.delimiter {
                ListDelimType::Period => '.',
                ListDelimType::Paren => ')',
            };
            let marker = format!("{}{} ", ordinal, delimiter);
            *ordinal += 1;
            marker
        }
    }
}

fn process_item<'a>(
    item: &'a AstNode<'a>,
    marker_text: String,
    indent_level: usize,
    depth: usize,
    push_block: &mut dyn FnMut(Block),
) {
    let children: Vec<_> = item.children().collect();
    let starts_with_text = matches!(
        children.first().map(|b| b.data.borrow().value.clone()),
        Some(NodeValue::Heading(_)) | Some(NodeValue::Paragraph)
    );
    let mut marker = if starts_with_text {
        Some(marker_text)
    } else {
        push_block(Block::Text(marker_only_line(marker_text, indent_level)));
        None
    };
    for block in children {
        process_item_block(block, &mut marker, indent_level, depth, push_block);
    }
}

fn process_item_block<'a>(
    block: &'a AstNode<'a>,
    marker: &mut Option<String>,
    indent_level: usize,
    depth: usize,
    push_block: &mut dyn FnMut(Block),
) {
    let value = block.data.borrow().value.clone();
    match value {
        NodeValue::Heading(heading) if marker.is_some() => {
            push_text_line(
                block,
                Some(heading.level),
                marker.take(),
                indent_level,
                push_block,
            );
        }
        NodeValue::Paragraph if marker.is_some() => {
            push_text_line(block, None, marker.take(), indent_level, push_block);
        }
        NodeValue::List(_) => parse_block(block, indent_level + 1, depth + 1, push_block),
        _ => parse_block(block, indent_level, depth + 1, push_block),
    }
}
