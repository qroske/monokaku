use std::path::Path;
use std::sync::Arc;

use gpui::{App, Application, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};

mod files;
mod markdown;
mod viewer;

use files::list_markdown_files;
use viewer::MarkdownViewer;

fn main() {
    let path = parse_args();
    let content = std::fs::read_to_string(&path).expect("ファイルの読み込みに失敗しました");
    let dir = Path::new(&path)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let files = list_markdown_files(dir);
    let current_path: Arc<Path> = Arc::from(Path::new(&path));

    run_app(content, files, current_path)
}

fn parse_args() -> String {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("使い方: cargo run -- <path/to/file.md>")
    };
    args[1].clone()
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
