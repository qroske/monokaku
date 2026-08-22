use super::model::{Block, Line};
use gpui::{Div, FontWeight, Pixels, StyledText, div, prelude::*, px};

const MUTED_BACKGROUND: u32 = 0xf0f0f0;
const MUTED_BORDER: u32 = 0xcccccc;

pub fn render_block(block: Block) -> Div {
    match block {
        Block::Text(line) => render_line(line),
        Block::Code(code, indent) => render_code(code, indent),
        Block::Table(rows, indent) => render_table(rows, indent),
        Block::Quote(blocks, indent) => render_quote(blocks, indent),
        Block::Rule(indent) => render_rule(indent),
    }
}

fn indent_padding(level: usize) -> Pixels {
    px(16.0 * level as f32)
}

fn render_line(line: Line) -> Div {
    let mut row = div().flex().pl(indent_padding(line.indent_level));
    if let Some(marker) = &line.list_marker {
        row = row.child(marker.clone());
    }
    row = row.child(StyledText::new(line.text).with_highlights(line.highlights));

    match line.heading_level {
        Some(1) | Some(2) => row.text_2xl().font_weight(FontWeight::BOLD),
        Some(3) | Some(4) => row.text_xl().font_weight(FontWeight::BOLD),
        Some(5) | Some(6) => row.text_lg().font_weight(FontWeight::BOLD),
        _ => row,
    }
}

fn render_code(code: String, indent: usize) -> Div {
    div().pl(indent_padding(indent)).child(
        div()
            .w_full()
            .rounded_md()
            .bg(gpui::rgb(MUTED_BACKGROUND))
            .p_2()
            .font_family("monospace")
            .child(code),
    )
}

fn render_table_row(is_header: bool, row: Vec<Line>) -> Div {
    let mut row_div = div().flex();
    for cell in row {
        let mut cell_div = div()
            .flex_1()
            .pl_1()
            .border_1()
            .border_color(gpui::rgb(MUTED_BORDER));
        if is_header {
            cell_div = cell_div
                .font_weight(FontWeight::BOLD)
                .bg(gpui::rgb(MUTED_BACKGROUND));
        }
        row_div = row_div.child(cell_div.child(render_line(cell)));
    }
    row_div
}

fn render_table(rows: Vec<(bool, Vec<Line>)>, indent: usize) -> Div {
    let mut table = div().flex().flex_col();
    for (is_header, row) in rows {
        table = table.child(render_table_row(is_header, row));
    }
    div().pl(indent_padding(indent)).child(table)
}

fn render_quote(blocks: Vec<Block>, indent: usize) -> Div {
    let mut quote = div()
        .flex()
        .flex_col()
        .pl_3()
        .border_l_2()
        .border_color(gpui::rgb(MUTED_BORDER));
    for block in blocks {
        quote = quote.child(render_block(block));
    }
    div().pl(indent_padding(indent)).child(quote)
}

fn render_rule(indent: usize) -> Div {
    div()
        .pl(indent_padding(indent))
        .child(div().w_full().h(px(1.0)).bg(gpui::rgb(MUTED_BORDER)))
}
