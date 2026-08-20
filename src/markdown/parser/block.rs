use comrak::nodes::{AstNode, NodeCodeBlock, NodeValue};

use super::line::{collect_text_line, nesting_limit_block, push_text_line};
use super::list::parse_list;
use crate::markdown::model::{Block, MAX_NESTING_DEPTH};

pub(super) fn parse_block<'a>(
    node: &'a AstNode<'a>,
    indent_level: usize,
    depth: usize,
    push_block: &mut dyn FnMut(Block),
) {
    if depth > MAX_NESTING_DEPTH {
        push_block(nesting_limit_block(indent_level));
        return;
    }
    let value = node.data.borrow().value.clone();
    match value {
        NodeValue::Heading(heading) => {
            push_text_line(node, Some(heading.level), None, indent_level, push_block);
        }
        NodeValue::Paragraph => push_text_line(node, None, None, indent_level, push_block),
        NodeValue::List(list) => parse_list(node, list, indent_level, depth, push_block),
        NodeValue::CodeBlock(code_block) => push_code(&code_block, indent_level, push_block),
        NodeValue::Table(_) => push_table(node, indent_level, push_block),
        NodeValue::BlockQuote => push_quote(node, indent_level, depth, push_block),
        NodeValue::ThematicBreak => push_block(Block::Rule(indent_level)),
        _ => {}
    }
}

fn push_code(code_block: &NodeCodeBlock, indent_level: usize, push_block: &mut dyn FnMut(Block)) {
    let code = code_block
        .literal
        .strip_suffix('\n')
        .unwrap_or(&code_block.literal)
        .to_string();
    push_block(Block::Code(code, indent_level));
}

fn push_table<'a>(node: &'a AstNode<'a>, indent_level: usize, push_block: &mut dyn FnMut(Block)) {
    let mut rows = Vec::new();
    for row in node.children() {
        if let NodeValue::TableRow(is_header) = row.data.borrow().value {
            let cells = row
                .children()
                .filter(|c| matches!(c.data.borrow().value, NodeValue::TableCell))
                .map(|cell| collect_text_line(cell, None, None, 0))
                .collect();
            rows.push((is_header, cells));
        }
    }
    push_block(Block::Table(rows, indent_level));
}

fn push_quote<'a>(
    node: &'a AstNode<'a>,
    indent_level: usize,
    depth: usize,
    push_block: &mut dyn FnMut(Block),
) {
    let mut blocks = Vec::new();
    for child in node.children() {
        parse_block(child, 0, depth + 1, &mut |b| blocks.push(b));
    }
    push_block(Block::Quote(blocks, indent_level));
}
