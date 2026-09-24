use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::{
    AnyElement, AsyncApp, ClickEvent, Context, Div, ElementId, FontWeight, PathPromptOptions,
    Window, div, prelude::*, px,
};

use crate::files::{FileEntry, build_file_tree, first_markdown_content};
use crate::markdown::parser::parse_markdown;
use crate::markdown::render::render_block;

const SELECTED_BACKGROUND: u32 = 0xe0e0e0;
const INDENT_STEP: f32 = 16.0;

pub struct MarkdownViewer {
    pub content: String,
    pub tree: Vec<FileEntry>,
    pub current_path: Arc<Path>,
    pub generation: u64,
}

type OpenedFolder = (Vec<FileEntry>, Option<(PathBuf, String)>);

impl MarkdownViewer {
    fn next_generation(&mut self) -> u64 {
        self.generation += 1;
        self.generation
    }

    fn open_folder(&mut self, dir: PathBuf, cx: &mut Context<Self>) {
        let generation = self.next_generation();

        cx.spawn(async move |this, cx| {
            let (tree, selected) = load_folder(dir, cx).await;
            this.update(cx, |this, cx| {
                this.apply_folder(generation, tree, selected, cx)
            })
        })
        .detach();
    }

    fn apply_folder(
        &mut self,
        generation: u64,
        tree: Vec<FileEntry>,
        selected: Option<(PathBuf, String)>,
        cx: &mut Context<Self>,
    ) {
        if generation != self.generation {
            return;
        }
        self.tree = tree;
        match selected {
            Some((path, content)) => {
                self.content = content;
                self.current_path = Arc::from(path);
            }
            None => {
                self.content = String::new();
                self.current_path = Arc::from(Path::new(""));
            }
        }
        cx.notify();
    }
}

async fn load_folder(dir: PathBuf, cx: &mut AsyncApp) -> OpenedFolder {
    cx.background_executor()
        .spawn(async move {
            let tree = build_file_tree(&dir);
            let selected = match first_markdown_content(&tree) {
                Ok(found) => found,
                Err(err) => Some((
                    PathBuf::new(),
                    format!("ファイルの読み込みに失敗しました: {err}"),
                )),
            };
            (tree, selected)
        })
        .await
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
    let open_folder_button = render_open_folder_button(cx);
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
        .child(open_folder_button)
        .children(items)
}

fn render_open_folder_button(cx: &mut Context<MarkdownViewer>) -> impl IntoElement + use<> {
    div()
        .id("open-folder-button")
        .px_2()
        .py_1()
        .child("フォルダを開く")
        .on_click(cx.listener(|_this, _event: &ClickEvent, _window, cx| {
            open_folder_dialog(cx);
        }))
}

fn open_folder_dialog(cx: &mut Context<MarkdownViewer>) {
    let options = PathPromptOptions {
        files: false,
        directories: true,
        multiple: false,
        prompt: None,
    };

    cx.spawn(async move |this, cx| {
        let Ok(receiver) = cx.update(|cx| cx.prompt_for_paths(options)) else {
            return;
        };
        let Ok(Ok(Some(mut paths))) = receiver.await else {
            return;
        };
        let Some(dir) = paths.pop() else {
            return;
        };

        this.update(cx, |this, cx| this.open_folder(dir, cx)).ok();
    })
    .detach();
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
        .on_click(cx.listener(move |this, _event: &ClickEvent, _window, cx| {
            load_file(this, path.clone(), cx);
        }));

    if is_selected {
        item = item
            .bg(gpui::rgb(SELECTED_BACKGROUND))
            .font_weight(FontWeight::BOLD);
    }

    item
}

fn load_file(this: &mut MarkdownViewer, path: Arc<Path>, cx: &mut Context<MarkdownViewer>) {
    let generation = this.next_generation();

    cx.spawn(async move |weak, cx| {
        let read_path = path.clone();
        let content = cx
            .background_executor()
            .spawn(async move { std::fs::read_to_string(&*read_path) })
            .await;
        weak.update(cx, |this, cx| {
            if generation == this.generation {
                this.content = content.expect("ファイルの読み込みに失敗しました");
                this.current_path = path;
                cx.notify();
            }
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
