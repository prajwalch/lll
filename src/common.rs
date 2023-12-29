use std::fmt::Write;
use std::fs::DirEntry;
use std::io;
use std::path::Path;

pub const PAGE_TEMPLATE: &str = include_str!("static/template.html");
pub const FILE_ICON: &str = include_str!("static/file.svg");
pub const DIR_ICON: &str = include_str!("static/folder.svg");

pub fn build_not_found_page() -> String {
    PAGE_TEMPLATE.replace("{title}", "Error Response").replace(
        "{content}",
        "<h1>404 Not Found</h1><p>Nothing matches the given URI</p>",
    )
}

pub fn build_directory_listing_page(url: &str, root: &Path, path: &Path) -> io::Result<String> {
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
        // Convert path to a url path.
        let link = fs_path_to_url(root, &path);
        // Use icon to represent whether path is a dir or file.
        let icon = if path.is_dir() { DIR_ICON } else { FILE_ICON };
        // We can directly print it but it adds `"` as prefix and suffix.
        let text = entry.file_name();
        // Convert to a lossy string so that we can print.
        let text = text.to_string_lossy();

        // NOTE: Unwrapping is completely safe here.
        writeln!(output, "<li><a href=\"{link}\">{icon} {text}</a></li>",).unwrap();
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
    let mut url = crate::normalize_url(&relative_path.to_string_lossy());
    // 3) Add trailing `/` if path points to a dir on disk.
    if path.is_dir() {
        url.push('/');
    }

    url
}
