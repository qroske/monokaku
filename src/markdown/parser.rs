mod block;
mod inline;
mod line;
mod list;

use comrak::{Arena, Options, parse_document};

use super::model::Block;
use block::parse_block;

pub fn parse_markdown(content: &str) -> Vec<Block> {
    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.table = true;
    let root = parse_document(&arena, content, &options);

    let mut result = Vec::new();

    for node in root.children() {
        parse_block(node, 0, 0, &mut |b| result.push(b));
    }
    result
}
