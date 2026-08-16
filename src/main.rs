use comrak::nodes::{AstNode, ListDelimType, ListType, NodeValue};
use comrak::{Arena, Options, parse_document};
use gpui::{
    App, Application, Bounds, Context, Div, FontStyle, FontWeight, HighlightStyle, StyledText,
    UnderlineStyle, Window, WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use std::ops::Range;

mod markdown;

use markdown::model::{Block, Line, MAX_NESTING_DEPTH};

struct MarkdownViewer {
    content: String,
}

fn parse_block<'a>(
    node: &'a AstNode<'a>,
    indent_level: usize,
    depth: usize,
    push_block: &mut dyn FnMut(Block),
) {
    if depth > MAX_NESTING_DEPTH {
        push_block(Block::Text(Line {
            text: "…（ネストが深すぎるため以下省略）".to_string(),
            highlights: Vec::new(),
            heading_level: None,
            list_marker: None,
            indent_level,
        }));
        return;
    }
    let value = node.data.borrow().value.clone();
    match value {
        NodeValue::Heading(heading) => {
            let mut text = String::new();
            let mut highlights = Vec::new();
            collect_inline(node, &mut text, &mut highlights, HighlightStyle::default());
            if !text.is_empty() {
                push_block(Block::Text(Line {
                    text,
                    highlights,
                    heading_level: Some(heading.level),
                    list_marker: None,
                    indent_level,
                }));
            }
        }
        NodeValue::Paragraph => {
            let mut text = String::new();
            let mut highlights = Vec::new();
            collect_inline(node, &mut text, &mut highlights, HighlightStyle::default());
            if !text.is_empty() {
                push_block(Block::Text(Line {
                    text,
                    highlights,
                    heading_level: None,
                    list_marker: None,
                    indent_level,
                }));
            }
        }
        NodeValue::List(list) => {
            parse_list(node, list, indent_level, depth, push_block);
        }
        NodeValue::CodeBlock(code_block) => {
            let code = code_block
                .literal
                .strip_suffix('\n')
                .unwrap_or(&code_block.literal)
                .to_string();
            push_block(Block::Code(code, indent_level));
        }
        NodeValue::Table(_) => {
            let mut rows = Vec::new();
            for row in node.children() {
                if let NodeValue::TableRow(is_header) = row.data.borrow().value {
                    let mut cells = Vec::new();
                    for cell in row.children() {
                        if let NodeValue::TableCell = cell.data.borrow().value {
                            let mut text = String::new();
                            let mut highlights = Vec::new();
                            collect_inline(
                                cell,
                                &mut text,
                                &mut highlights,
                                HighlightStyle::default(),
                            );
                            cells.push(Line {
                                text,
                                highlights,
                                heading_level: None,
                                list_marker: None,
                                indent_level: 0,
                            });
                        }
                    }
                    rows.push((is_header, cells));
                }
            }
            push_block(Block::Table(rows, indent_level));
        }
        NodeValue::BlockQuote => {
            let mut blocks = Vec::new();
            for child in node.children() {
                parse_block(child, 0, depth + 1, &mut |b| blocks.push(b));
            }
            push_block(Block::Quote(blocks, indent_level));
        }
        NodeValue::ThematicBreak => {
            push_block(Block::Rule(indent_level));
        }
        _ => {}
    }
}

fn parse_list<'a>(
    node: &'a AstNode<'a>,
    list: comrak::nodes::NodeList,
    indent_level: usize,
    depth: usize,
    push_block: &mut dyn FnMut(Block),
) {
    if depth > MAX_NESTING_DEPTH {
        push_block(Block::Text(Line {
            text: "…（ネストが深すぎるため以下省略）".to_string(),
            highlights: Vec::new(),
            heading_level: None,
            list_marker: None,
            indent_level,
        }));
        return;
    }
    let mut ordinal = list.start;
    let delimiter = match list.delimiter {
        ListDelimType::Period => '.',
        ListDelimType::Paren => ')',
    };
    for item in node.children() {
        let marker_text = match list.list_type {
            ListType::Bullet => "• ".to_string(),
            ListType::Ordered => {
                let marker = format!("{}{} ", ordinal, delimiter);
                ordinal += 1;
                marker
            }
        };
        let children: Vec<_> = item.children().collect();
        let starts_with_text = matches!(
            children.first().map(|b| b.data.borrow().value.clone()),
            Some(NodeValue::Heading(_)) | Some(NodeValue::Paragraph)
        );
        let mut marker = if starts_with_text {
            Some(marker_text)
        } else {
            push_block(Block::Text(Line {
                text: String::new(),
                highlights: Vec::new(),
                heading_level: None,
                list_marker: Some(marker_text),
                indent_level,
            }));
            None
        };
        for block in children {
            let block_value = block.data.borrow().value.clone();
            match &block_value {
                NodeValue::Heading(heading) if marker.is_some() => {
                    let mut text = String::new();
                    let mut highlights = Vec::new();
                    collect_inline(block, &mut text, &mut highlights, HighlightStyle::default());
                    push_block(Block::Text(Line {
                        text,
                        highlights,
                        heading_level: Some(heading.level),
                        list_marker: marker.take(),
                        indent_level,
                    }));
                }
                NodeValue::Paragraph if marker.is_some() => {
                    let mut text = String::new();
                    let mut highlights = Vec::new();
                    collect_inline(block, &mut text, &mut highlights, HighlightStyle::default());
                    push_block(Block::Text(Line {
                        text,
                        highlights,
                        heading_level: None,
                        list_marker: marker.take(),
                        indent_level,
                    }));
                }
                NodeValue::List(_) => {
                    parse_block(block, indent_level + 1, depth + 1, push_block);
                }
                _ => {
                    parse_block(block, indent_level, depth + 1, push_block);
                }
            }
        }
    }
}

fn collect_inline<'a>(
    node: &'a AstNode<'a>,
    text: &mut String,
    highlights: &mut Vec<(Range<usize>, HighlightStyle)>,
    style: HighlightStyle,
) {
    for child in node.children() {
        let value = child.data.borrow().value.clone();
        match value {
            NodeValue::Text(t) => {
                let start = text.len();
                text.push_str(&t);
                if style != HighlightStyle::default() {
                    highlights.push((start..text.len(), style));
                }
            }
            NodeValue::Strong => {
                collect_inline(
                    child,
                    text,
                    highlights,
                    HighlightStyle {
                        font_weight: Some(FontWeight::BOLD),
                        ..style
                    },
                );
            }
            NodeValue::Emph => {
                collect_inline(
                    child,
                    text,
                    highlights,
                    HighlightStyle {
                        font_style: Some(FontStyle::Italic),
                        ..style
                    },
                );
            }
            NodeValue::Link(_) => {
                collect_inline(
                    child,
                    text,
                    highlights,
                    HighlightStyle {
                        underline: Some(UnderlineStyle {
                            thickness: px(1.0),
                            ..Default::default()
                        }),
                        ..style
                    },
                );
            }
            NodeValue::Image(_) => {
                let prefix_start = text.len();
                text.push_str("[image: ");
                if style != HighlightStyle::default() {
                    highlights.push((prefix_start..text.len(), style));
                }
                collect_inline(child, text, highlights, style);
                let suffix_start = text.len();
                text.push(']');
                if style != HighlightStyle::default() {
                    highlights.push((suffix_start..text.len(), style));
                }
            }
            NodeValue::SoftBreak | NodeValue::LineBreak => {
                let start = text.len();
                text.push(' ');
                if style != HighlightStyle::default() {
                    highlights.push((start..text.len(), style));
                }
            }
            NodeValue::Code(code) => {
                let start = text.len();
                text.push_str(&code.literal);
                if style != HighlightStyle::default() {
                    highlights.push((start..text.len(), style));
                }
            }
            _ => collect_inline(child, text, highlights, style),
        }
    }
}

fn parse_markdown(content: &str) -> Vec<Block> {
    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.table = true;
    let root = parse_document(&arena, content, &options);

    let mut result = Vec::new();

    for node in root.children() {
        parse_block(node, 0, 0, &mut |b| result.push(b));
    }
    result
}

fn render_line(line: Line) -> Div {
    let mut row = div().flex().pl(px(16.0 * line.indent_level as f32));
    if let Some(marker) = &line.list_marker {
        row = row.child(marker.clone());
    }
    row = row.child(StyledText::new(line.text).with_highlights(line.highlights));

    match line.heading_level {
        Some(1) | Some(2) => row.text_2xl().font_weight(FontWeight::BOLD),
        Some(3) | Some(4) => row.text_xl().font_weight(FontWeight::BOLD),
        Some(5) | Some(6) => row.text_lg().font_weight(FontWeight::BOLD),
        _ => row,
    }
}

fn render_block(block: Block) -> Div {
    match block {
        Block::Text(line) => render_line(line),
        Block::Code(code, indent) => div().pl(px(16.0 * indent as f32)).child(
            div()
                .w_full()
                .rounded_md()
                .bg(gpui::rgb(0xf0f0f0))
                .p_2()
                .font_family("monospace")
                .child(code),
        ),
        Block::Table(rows, indent) => {
            let mut table = div().flex().flex_col();
            for (is_header, row) in rows {
                let mut row_div = div().flex();
                for cell in row {
                    let mut cell_div = div()
                        .flex_1()
                        .pl_1()
                        .border_1()
                        .border_color(gpui::rgb(0xcccccc));
                    if is_header {
                        cell_div = cell_div
                            .font_weight(FontWeight::BOLD)
                            .bg(gpui::rgb(0xf0f0f0));
                    }
                    row_div = row_div.child(cell_div.child(render_line(cell)));
                }
                table = table.child(row_div);
            }
            div().pl(px(16.0 * indent as f32)).child(table)
        }
        Block::Quote(blocks, indent) => {
            let mut quote = div()
                .flex()
                .flex_col()
                .pl_3()
                .border_l_2()
                .border_color(gpui::rgb(0xcccccc));
            for block in blocks {
                quote = quote.child(render_block(block));
            }
            div().pl(px(16.0 * indent as f32)).child(quote)
        }
        Block::Rule(indent) => div()
            .pl(px(16.0 * indent as f32))
            .child(div().w_full().h(px(1.0)).bg(gpui::rgb(0xcccccc))),
    }
}

impl Render for MarkdownViewer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let blocks = parse_markdown(&self.content).into_iter().map(render_block);
        div()
            .id("markdown-content")
            .size_full()
            .flex()
            .flex_col()
            .overflow_scroll()
            .bg(gpui::white())
            .text_color(gpui::black())
            .px_3()
            .children(blocks)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("使い方: cargo run -- <path/to/file.md>")
    };
    let path = args[1].clone();
    let content = std::fs::read_to_string(&path).expect("ファイルの読み込みに失敗しました");
    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_, cx| cx.new(|_| MarkdownViewer { content }),
        )
        .unwrap();
        cx.activate(true);
    });
}
