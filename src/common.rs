use std::fmt::Write;
use std::io;
use std::path::Path;

use crate::normalize_url;

pub const PAGE_TEMPLATE: &str = include_str!("static/template.html");
pub const FILE_SVG_ICON: &str = include_str!("static/file.svg");
pub const FOLDER_SVG_ICON: &str = include_str!("static/folder.svg");

pub fn build_not_found_page() -> String {
    PAGE_TEMPLATE.replace("{title}", "Error Response").replace(
        "{content}",
        "<h1>404 Not Found</h1><p>Nothing matches the given URI</p>",
    )
}

pub fn build_directory_listing_page(url: &str, root: &Path, path: &Path) -> io::Result<String> {
    // Sort the entries so that the directories shows up first and then files.
    // entries.sort_by_key(|(_, url_entry)| url_entry.fs_path.is_file());

    let links = create_entry_hyperlinks(root, path)?;
    let content = format!("<h1>Directory Listing for {url}</h1>\n<ul>{links}</ul>");

    Ok(PAGE_TEMPLATE
        .replace("{title}", "Directory Listing")
        .replace("{content}", &content))
}

fn create_entry_hyperlinks(root: &Path, path: &Path) -> io::Result<String> {
    let mut entries: Vec<DirEntry> = path.read_dir()?.flatten().collect();
    // Sort the entries so that the directories shows up first and then files.
    entries.sort_by_cached_key(|entry| entry.path().is_file());

    entries.iter().try_fold(String::new(), |mut output, entry| {
        let path = entry.path();

        let icon = if path.is_dir() {
            FOLDER_SVG_ICON
        } else {
            FILE_SVG_ICON
        };

        // NOTE: Unwrapping is completely safe here.
        writeln!(
            output,
            "<li><a href=\"{}\">{icon} {}</a></li>",
            fs_path_to_url(root, &path),
            entry.file_name().to_string_lossy()
        )
        .unwrap();
        Ok(output)
    })
}

fn fs_path_to_url(root: &Path, path: &Path) -> String {
    if path == root {
        // If path is a root then return early.
        return String::from("/");
    }

    // 1) Remove root from the path.
    //
    // Eg: root = /home/x/one
    //     /home/x/one/main.rs -> /main.rs
    let relative_path = path.strip_prefix(root).unwrap_or(path);
    // 2) Convert it to string and normalize it.
    let mut url = normalize_url(&relative_path.to_string_lossy());
    // 3) Add trailing `/` if path points to a dir on disk.
    if path.is_dir() {
        url.push('/');
    }

    url
}
