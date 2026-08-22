use gpui::{Context, Window, div, prelude::*};

use crate::markdown::parser::parse_markdown;
use crate::markdown::render::render_block;

pub struct MarkdownViewer {
    pub content: String,
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
