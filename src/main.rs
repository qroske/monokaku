use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::{App, Application, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};

mod files;
mod markdown;
mod viewer;

use files::list_markdown_files;
use viewer::MarkdownViewer;

fn main() {
    let path = parse_args();
    let (files, selected) = resolve_initial_state(&path);
    let content = std::fs::read_to_string(&selected).expect("ファイルの読み込みに失敗しました");
    let current_path: Arc<Path> = Arc::from(selected);

    run_app(content, files, current_path)
}

fn parse_args() -> String {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("使い方: cargo run -- <path/to/file.md もしくはディレクトリ>")
    };
    args[1].clone()
}

fn resolve_initial_state(path: &str) -> (Vec<PathBuf>, PathBuf) {
    let path = Path::new(path);
    let is_dir = path.is_dir();
    let files = list_markdown_files(listing_dir(path, is_dir));
    let selected = if is_dir {
        files
            .first()
            .cloned()
            .expect("ディレクトリ内に.mdファイルが見つかりませんでした")
    } else {
        let name = path.file_name();
        files
            .iter()
            .find(|entry| entry.file_name() == name)
            .cloned()
            .unwrap_or_else(|| path.to_path_buf())
    };

    (files, selected)
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

fn run_app(content: String, files: Vec<std::path::PathBuf>, current_path: Arc<Path>) {
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
                    files,
                    current_path,
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
