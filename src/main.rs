use comrak::nodes::{AstNode, NodeValue};
use comrak::{Arena, Options, parse_document};
use gpui::{
    App, Application, Bounds, Context, FontStyle, FontWeight, HighlightStyle, StyledText, Window,
    WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use std::ops::Range;

struct MarkdownViewer {
    content: String,
}

struct Line {
    text: String,
    highlights: Vec<(Range<usize>, HighlightStyle)>,
    heading_level: Option<u8>,
    is_list_item: bool,
}

fn collect_inline<'a>(
    node: &'a AstNode<'a>,
    text: &mut String,
    highlights: &mut Vec<(Range<usize>, HighlightStyle)>,
) {
    for child in node.children() {
        let value = child.data.borrow().value.clone();
        match value {
            NodeValue::Text(t) => text.push_str(&t),
            NodeValue::Strong => {
                let start = text.len();
                collect_inline(child, text, highlights);
                highlights.push((
                    start..text.len(),
                    HighlightStyle {
                        font_weight: Some(FontWeight::BOLD),
                        ..Default::default()
                    },
                ));
            }
            NodeValue::Emph => {
                let start = text.len();
                collect_inline(child, text, highlights);
                highlights.push((
                    start..text.len(),
                    HighlightStyle {
                        font_style: Some(FontStyle::Italic),
                        ..Default::default()
                    },
                ));
            }
            NodeValue::SoftBreak | NodeValue::LineBreak => text.push(' '),
            NodeValue::Code(code) => text.push_str(&code.literal),
            _ => collect_inline(child, text, highlights),
        }
    }
}

fn parse_markdown(content: &str) -> Vec<Line> {
    let arena = Arena::new();
    let root = parse_document(&arena, content, &Options::default());

    let mut result = Vec::new();

    for node in root.children() {
        let value = node.data.borrow().value.clone();
        match value {
            NodeValue::Heading(heading) => {
                let mut text = String::new();
                let mut highlights = Vec::new();
                collect_inline(node, &mut text, &mut highlights);
                if !text.is_empty() {
                    result.push(Line {
                        text,
                        highlights,
                        heading_level: Some(heading.level),
                        is_list_item: false,
                    });
                }
            }
            NodeValue::Paragraph => {
                let mut text = String::new();
                let mut highlights = Vec::new();
                collect_inline(node, &mut text, &mut highlights);
                if !text.is_empty() {
                    result.push(Line {
                        text,
                        highlights,
                        heading_level: None,
                        is_list_item: false,
                    });
                }
            }
            NodeValue::List(_) => {
                for item in node.children() {
                    if let Some(block) = item.children().next() {
                        let mut text = String::new();
                        let mut highlights = Vec::new();
                        collect_inline(block, &mut text, &mut highlights);
                        if !text.is_empty() {
                            result.push(Line {
                                text,
                                highlights,
                                heading_level: None,
                                is_list_item: true,
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }
    result
}

impl Render for MarkdownViewer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let lines = parse_markdown(&self.content).into_iter().map(|line| {
            let mut row = div().flex();
            if line.is_list_item {
                row = row.child("• ");
            }
            row = row.child(StyledText::new(line.text).with_highlights(line.highlights));

            match line.heading_level {
                Some(1) | Some(2) => row.text_2xl().font_weight(FontWeight::BOLD),
                Some(3) | Some(4) => row.text_xl().font_weight(FontWeight::BOLD),
                Some(5) | Some(6) => row.text_lg().font_weight(FontWeight::BOLD),
                _ => row,
            }
        });
        div()
            .id("markdown-content")
            .size_full()
            .flex()
            .flex_col()
            .overflow_scroll()
            .bg(gpui::white())
            .text_color(gpui::black())
            .px_3()
            .children(lines)
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
