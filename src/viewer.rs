use std::path::PathBuf;

use gpui::{Context, Window, div, prelude::*, px};

use crate::markdown::parser::parse_markdown;
use crate::markdown::render::render_block;

pub struct MarkdownViewer {
    pub content: String,
    pub files: Vec<PathBuf>,
}

impl Render for MarkdownViewer {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let sidebar = render_sidebar(&self.files);
        let content = render_content(&self.content);

        div().size_full().flex().child(sidebar).child(content)
    }
}

fn render_sidebar(files: &[PathBuf]) -> impl IntoElement {
    let items = files.iter().map(|path| {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        div().child(name.to_string())
    });

    div()
        .w(px(200.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(gpui::white())
        .text_color(gpui::black())
        .children(items)
}

fn render_content(content: &str) -> impl IntoElement {
    let blocks = parse_markdown(content).into_iter().map(render_block);

    div()
        .id("markdown-content")
        .flex_1()
        .flex()
        .flex_col()
        .overflow_scroll()
        .bg(gpui::white())
        .text_color(gpui::black())
        .px_3()
        .children(blocks)
}
