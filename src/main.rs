use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::{App, Application, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};

mod files;
mod markdown;
mod viewer;

use files::{FileEntry, build_file_tree, first_markdown};
use viewer::MarkdownViewer;

fn main() {
    let path = parse_args();
    let (tree, selected) = resolve_initial_state(&path);
    let content = std::fs::read_to_string(&selected).expect("ファイルの読み込みに失敗しました");
    let current_path: Arc<Path> = Arc::from(selected);

    run_app(content, tree, current_path)
}

fn parse_args() -> String {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("使い方: cargo run -- <path/to/file.md もしくはディレクトリ>")
    };
    args[1].clone()
}

fn resolve_initial_state(path: &str) -> (Vec<FileEntry>, PathBuf) {
    let path = Path::new(path);
    let is_dir = path.is_dir();
    let dir = listing_dir(path, is_dir);
    let tree = build_file_tree(dir);
    let selected = if is_dir {
        first_markdown(&tree).expect("ディレクトリ内に.mdファイルが見つかりませんでした")
    } else {
        match path.file_name() {
            Some(name) => dir.join(name),
            None => path.to_path_buf(),
        }
    };

    (tree, selected)
}

fn listing_dir(path: &Path, is_dir: bool) -> &Path {
    if is_dir {
        path
    } else {
        path.parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
    }
}

fn run_app(content: String, tree: Vec<FileEntry>, current_path: Arc<Path>) {
    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_, cx| {
                cx.new(|_| MarkdownViewer {
                    content,
                    tree,
                    current_path,
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
