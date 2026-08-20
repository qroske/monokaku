use gpui::{
    App, Application, Bounds, Context, Div, FontWeight, StyledText, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, size,
};

mod markdown;

use markdown::model::{Block, Line};
use markdown::parser::parse_markdown;

struct MarkdownViewer {
    content: String,
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

#[expect(clippy::too_many_lines)]
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
