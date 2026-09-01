use std::path::Path;
use std::sync::Arc;

use gpui::{
    AnyElement, ClickEvent, Context, Div, ElementId, FontWeight, Window, div, prelude::*, px,
};

use crate::files::FileEntry;
use crate::markdown::parser::parse_markdown;
use crate::markdown::render::render_block;

const SELECTED_BACKGROUND: u32 = 0xe0e0e0;
const INDENT_STEP: f32 = 16.0;

pub struct MarkdownViewer {
    pub content: String,
    pub tree: Vec<FileEntry>,
    pub current_path: Arc<Path>,
}

impl Render for MarkdownViewer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sidebar = render_sidebar(&self.tree, &self.current_path, cx);
        let content = render_content(&self.content, &self.current_path);

        div().size_full().flex().child(sidebar).child(content)
    }
}

fn render_sidebar(
    tree: &[FileEntry],
    current_path: &Arc<Path>,
    cx: &mut Context<MarkdownViewer>,
) -> impl IntoElement {
    let items = tree
        .iter()
        .map(|entry| render_tree_entry(entry, current_path, 0, cx));

    div()
        .w(px(200.0))
        .h_full()
        .flex()
        .flex_col()
        .bg(gpui::white())
        .text_color(gpui::black())
        .children(items)
}

fn render_tree_entry(
    entry: &FileEntry,
    current_path: &Arc<Path>,
    depth: usize,
    cx: &mut Context<MarkdownViewer>,
) -> AnyElement {
    match entry {
        FileEntry::Dir { name, children, .. } => {
            render_dir(name, children, current_path, depth, cx).into_any_element()
        }
        FileEntry::File { path, name } => {
            render_file(path, name, current_path, depth, cx).into_any_element()
        }
    }
}

fn render_dir(
    name: &str,
    children: &[FileEntry],
    current_path: &Arc<Path>,
    depth: usize,
    cx: &mut Context<MarkdownViewer>,
) -> Div {
    let rows = children
        .iter()
        .map(|child| render_tree_entry(child, current_path, depth + 1, cx));

    div()
        .flex()
        .flex_col()
        .child(
            div()
                .pl(px(INDENT_STEP * depth as f32))
                .child(name.to_string()),
        )
        .children(rows)
}

fn render_file(
    path: &Path,
    name: &str,
    current_path: &Arc<Path>,
    depth: usize,
    cx: &mut Context<MarkdownViewer>,
) -> impl IntoElement + use<> {
    let is_selected = path == current_path.as_ref();
    let path: Arc<Path> = Arc::from(path);

    let mut item = div()
        .id((ElementId::from(path.clone()), "sidebar-item"))
        .pl(px(INDENT_STEP * depth as f32))
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
