use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

pub enum FileEntry {
    Dir {
        name: String,
        children: Vec<FileEntry>,
    },
    File {
        path: PathBuf,
        name: String,
    },
}

pub fn build_file_tree(dir: &Path) -> Vec<FileEntry> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut tree: Vec<FileEntry> = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| classify_entry(entry.path()))
        .collect();
    tree.sort_by(compare_entries);
    tree
}

pub fn first_markdown(tree: &[FileEntry]) -> Option<PathBuf> {
    let file_here = tree.iter().find_map(|entry| match entry {
        FileEntry::File { path, .. } => Some(path.clone()),
        FileEntry::Dir { .. } => None,
    });
    file_here.or_else(|| {
        tree.iter().find_map(|entry| match entry {
            FileEntry::Dir { children, .. } => first_markdown(children),
            FileEntry::File { .. } => None,
        })
    })
}

fn classify_entry(path: PathBuf) -> Option<FileEntry> {
    let name = path.file_name()?.to_str()?.to_string();

    if path.is_dir() {
        let children = build_file_tree(&path);
        return (!children.is_empty()).then_some(FileEntry::Dir { name, children });
    }

    let is_markdown = path.extension().is_some_and(|ext| ext == "md");
    is_markdown.then_some(FileEntry::File { path, name })
}

fn compare_entries(a: &FileEntry, b: &FileEntry) -> Ordering {
    match (a, b) {
        (FileEntry::Dir { .. }, FileEntry::File { .. }) => Ordering::Less,
        (FileEntry::File { .. }, FileEntry::Dir { .. }) => Ordering::Greater,
        (FileEntry::Dir { name: a, .. }, FileEntry::Dir { name: b, .. }) => a.cmp(b),
        (FileEntry::File { name: a, .. }, FileEntry::File { name: b, .. }) => a.cmp(b),
    }
}
