use gpui::{App, Application, Bounds, Context, Window, WindowBounds, WindowOptions, div, prelude::*, px, size};

struct EmptyWindow;

impl Render for EmptyWindow {
    // 再描画のたびに呼ばれる関数。window/cxは今回未使用なので_接頭辞。戻り値は描画可能な要素
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full()
    }
}

fn main() {
    Application::new().run( |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| EmptyWindow)
        )
        .unwrap();
        cx.activate(true);
    });
}
