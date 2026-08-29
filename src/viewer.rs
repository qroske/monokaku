use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::{ClickEvent, Context, ElementId, FontWeight, Window, div, prelude::*, px};

use crate::markdown::parser::parse_markdown;
use crate::markdown::render::render_block;

const SELECTED_BACKGROUND: u32 = 0xe0e0e0;

pub struct MarkdownViewer {
    pub content: String,
    pub files: Vec<PathBuf>,
    pub current_path: Arc<Path>,
}

impl Render for MarkdownViewer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sidebar = render_sidebar(&self.files, &self.current_path, cx);
        let content = render_content(&self.content, &self.current_path);

        div().size_full().flex().child(sidebar).child(content)
    }
}

fn render_sidebar(
    files: &[PathBuf],
    current_path: &Arc<Path>,
    cx: &mut Context<MarkdownViewer>,
) -> impl IntoElement {
    let items = files
        .iter()
        .map(|path| render_sidebar_item(path, current_path, cx));

    div()
        .w(px(200.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(gpui::white())
        .text_color(gpui::black())
        .children(items)
}

fn render_sidebar_item(
    path: &Path,
    current_path: &Arc<Path>,
    cx: &mut Context<MarkdownViewer>,
) -> impl IntoElement + use<> {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let is_selected = path == current_path.as_ref();
    let path: Arc<Path> = Arc::from(path);

    let mut item = div()
        .id((ElementId::from(path.clone()), "sidebar-item"))
        .child(name.to_string())
        .on_click(cx.listener(move |_this, _event: &ClickEvent, _window, cx| {
            load_file(path.clone(), cx);
        }));

    if is_selected {
        item = item
            .bg(gpui::rgb(SELECTED_BACKGROUND))
            .font_weight(FontWeight::BOLD);
    }

    item
}

fn load_file(path: Arc<Path>, cx: &mut Context<MarkdownViewer>) {
    cx.spawn(async move |this, cx| {
        let read_path = path.clone();
        let content = cx
            .background_executor()
            .spawn(async move { std::fs::read_to_string(&*read_path) })
            .await;
        this.update(cx, |this, cx| {
            this.content = content.expect("ファイルの読み込みに失敗しました");
            this.current_path = path;
            cx.notify();
        })
    })
    .detach();
}

fn render_content(content: &str, current_path: &Arc<Path>) -> impl IntoElement {
    let blocks = parse_markdown(content).into_iter().map(render_block);

    div()
        .id((ElementId::from(current_path.clone()), "content"))
        .flex_1()
        .flex()
        .flex_col()
        .overflow_scroll()
        .bg(gpui::white())
        .text_color(gpui::black())
        .px_3()
        .children(blocks)
}
