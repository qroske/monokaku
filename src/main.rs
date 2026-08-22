use gpui::{
    App, Application, Bounds, Context, Window, WindowBounds, WindowOptions, div, prelude::*, px,
    size,
};

mod markdown;

use markdown::parser::parse_markdown;
use markdown::render::render_block;

struct MarkdownViewer {
    content: String,
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
