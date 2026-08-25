use std::fs;
use std::path::{Path, PathBuf};

pub fn list_markdown_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("monokaku_list_markdown_files_test_{name}"));
        fs::create_dir_all(&dir).expect("一時ディレクトリの作成に失敗しました");
        dir
    }

    #[test]
    fn returns_only_markdown_files() {
        let dir = temp_dir("returns_only_markdown_files");

        fs::write(dir.join("a.md"), "# a").expect("書き込みに失敗しました");
        fs::write(dir.join("b.md"), "# b").expect("書き込みに失敗しました");
        fs::write(dir.join("c.txt"), "not markdown").expect("書き込みに失敗しました");

        let mut files = list_markdown_files(&dir);
        files.sort();

        assert_eq!(files, vec![dir.join("a.md"), dir.join("b.md")]);

        fs::remove_dir_all(&dir).expect("後片付けに失敗しました");
    }

    #[test]
    fn returns_empty_vec_when_no_markdown_files_present() {
        let dir = temp_dir("returns_empty_vec_when_no_markdown_files_present");

        fs::write(dir.join("readme.txt"), "text").expect("書き込みに失敗しました");

        assert!(list_markdown_files(&dir).is_empty());

        fs::remove_dir_all(&dir).expect("後片付けに失敗しました");
    }

    #[test]
    fn excludes_subdirectories_even_if_name_ends_with_md() {
        let dir = temp_dir("excludes_subdirectories_even_if_name_ends_with_md");

        fs::create_dir_all(dir.join("sub.md")).expect("サブディレクトリの作成に失敗しました");
        fs::write(dir.join("real.md"), "# real").expect("書き込みに失敗しました");

        assert_eq!(list_markdown_files(&dir), vec![dir.join("real.md")]);

        fs::remove_dir_all(&dir).expect("後片付けに失敗しました");
    }

    #[test]
    fn includes_hidden_markdown_files() {
        let dir = temp_dir("includes_hidden_markdown_files");

        fs::write(dir.join("hidden.md"), "# hidden").expect("書き込みに失敗しました");

        assert_eq!(list_markdown_files(&dir), vec![dir.join("hidden.md")]);

        fs::remove_dir_all(&dir).expect("後片付けに失敗しました");
    }

    #[test]
    fn extension_match_is_case_sensitive() {
        let dir = temp_dir("extension_match_is_case_sensitive");

        fs::write(dir.join("upper.MD"), "# upper").expect("書き込みに失敗しました");

        assert!(list_markdown_files(&dir).is_empty());

        fs::remove_dir_all(&dir).expect("後片付けに失敗しました");
    }

    #[test]
    fn returns_empty_vec_for_noneexistent_directory() {
        let dir = temp_dir("monokaku_this_directory_should_not_exist");

        assert!(list_markdown_files(&dir).is_empty());
    }
}
