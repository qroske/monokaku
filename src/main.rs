use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::{App, Application, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};

mod files;
mod markdown;
mod viewer;

use files::{FileEntry, build_file_tree, first_markdown_content};
use viewer::MarkdownViewer;

fn main() {
    let path = parse_args();
    let (tree, selected, content) = resolve_initial_state(&path);
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

fn resolve_initial_state(path: &str) -> (Vec<FileEntry>, PathBuf, String) {
    let path = Path::new(path);
    let is_dir = path.is_dir();
    let dir = listing_dir(path, is_dir);
    let tree = build_file_tree(dir);
    if is_dir {
        let (selected, content) = match first_markdown_content(&tree) {
            Ok(Some(found)) => found,
            Ok(None) => panic!("ディレクトリ内に.mdファイルが見つかりませんでした"),
            Err(err) => panic!("ファイルの読み込みに失敗しました: {err}"),
        };
        return (tree, selected, content);
    }

    let selected = match path.file_name() {
        Some(name) => dir.join(name),
        None => path.to_path_buf(),
    };
    let content = std::fs::read_to_string(&selected).expect("ファイルの読み込みに失敗しました");
    (tree, selected, content)
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
                    generation: 0,
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
