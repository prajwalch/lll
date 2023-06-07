use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::common::{FILE_SVG_ICON, FOLDER_SVG_ICON, PAGE_TEMPLATE};
use crate::normalize_url;

pub struct UrlsTable<'a> {
    root_path: &'a Path,
    table: HashMap<String, UrlEntry>,
}

impl<'a> UrlsTable<'a> {
    pub fn new(root_path: &'a Path) -> Self {
        Self {
            root_path,
            table: HashMap::new(),
        }
    }

    pub fn get_url_entry_mut(&mut self, requested_url: &str) -> Option<&mut UrlEntry> {
        self.update_table_if_needed(requested_url);
        self.table.get_mut(requested_url)
    }

    pub fn contains_url_entry(&self, requested_url: &str) -> bool {
        self.table.contains_key(requested_url)
    }

    fn map_urls_from(&mut self, path: &Path) -> Result<(), String> {
        let dir_entries = path
            .read_dir()
            .map_err(|err| format!("Unable to map urls from `{}`: {}", path.display(), err))?;

        for dir_entry in dir_entries {
            let dir_entry = dir_entry.unwrap();
            let entry_fs_path = dir_entry.path();
            let mapped_url = self.fs_path_to_url(&entry_fs_path);

            self.table
                .entry(mapped_url)
                .or_insert(UrlEntry::new(entry_fs_path, None));
        }
        let mapped_root_url = self.fs_path_to_url(path);

        if self.table.contains_key(&mapped_root_url) {
            return Ok(());
        }
        // If path not contains `index.html` file build a directory listing page for it
        let entry_cache = EntryCache::new(
            self.build_directory_listing_page(&mapped_root_url, path),
            String::from("Content-Type: text/html"),
        );
        self.table.insert(
            mapped_root_url,
            UrlEntry::new(path.to_path_buf(), Some(entry_cache)),
        );
        Ok(())
    }

    fn fs_path_to_url(&self, fs_path: &Path) -> String {
        if fs_path == self.root_path {
            return String::from("/");
        }
        let child_path = fs_path.strip_prefix(self.root_path).unwrap();
        let mut url = normalize_url(&child_path.to_string_lossy());
        // Add a trailing slash `/` at the end of url for directory
        if child_path.is_dir() {
            url.push('/');
        }
        url
    }

    fn build_directory_listing_page(&self, url: &str, dir_path: &Path) -> Vec<u8> {
        let mut matched_entries = self
            .table
            .iter()
            .filter(|(_, url_entry)| url_entry.fs_path.parent().map_or(false, |p| p == dir_path))
            .collect::<Vec<(&String, &UrlEntry)>>();

        // Sort the entries so that directories shows first and then files
        matched_entries.sort_by_cached_key(|(_, url_entry)| url_entry.fs_path.is_file());

        let entries_hyperlinks = matched_entries
            .iter()
            .map(|(mapped_url, url_entry)| {
                let icon = if url_entry.fs_path.is_dir() {
                    FOLDER_SVG_ICON
                } else {
                    FILE_SVG_ICON
                };

                let basename = url_entry
                    .fs_path
                    .file_name()
                    .unwrap_or(url_entry.fs_path.as_ref())
                    .to_string_lossy();

                format!(r#"<li><a href="{mapped_url}">{icon} {basename}</a></li>"#)
            })
            .collect::<String>();

        let mut content = format!("<h1>Directory Listing for {url}</h1><ul>");
        content.push_str(&entries_hyperlinks);
        content.push_str("</ul>");

        PAGE_TEMPLATE
            .replace("{title}", "Directory Listing")
            .replace("{content}", &content)
            .into_bytes()
    }

    fn update_table_if_needed(&mut self, requested_url: &str) {
        let mut fs_path: Option<PathBuf> = None;

        if let Some(url_entry) = self.table.get(requested_url) {
            if url_entry.fs_path.is_file()
                || url_entry.cache.as_ref().is_some_and(|c| !c.is_expired())
            {
                return;
            }
            let requested_path = url_entry.fs_path.clone();

            self.table.remove(requested_url);
            // To prevent displaying the deleted file or directory in listing page, remove all urls
            // previously mapped from `requested_path` by keeping only the urls whose equivalent
            // fs_path's parent is not `requested_path`.
            self.table
                .retain(|_, entry| entry.fs_path.parent().map_or(true, |p| p != requested_path));

            fs_path = Some(requested_path);
        }
        let fs_path = fs_path.unwrap_or_else(|| self.url_to_fs_path(requested_url));
        if !fs_path.exists() {
            return;
        }

        let fs_path = if fs_path.is_file() && fs_path.parent().is_some_and(|p| p != self.root_path)
        {
            fs_path.parent().unwrap().to_path_buf()
        } else {
            fs_path
        };
        if let Err(e) = self.map_urls_from(&fs_path) {
            eprintln!("{e}");
        }
    }

    fn url_to_fs_path(&self, requested_url: &str) -> PathBuf {
        if let Some(url_entry) = self.table.get(requested_url) {
            return url_entry.fs_path.clone();
        }
        let root_path = self.root_path.to_path_buf();
        root_path.join(requested_url.strip_prefix('/').unwrap())
    }
}

#[derive(Debug, Clone)]
pub struct UrlEntry {
    pub fs_path: PathBuf,
    pub cache: Option<EntryCache>,
}

impl UrlEntry {
    pub fn new(fs_path: PathBuf, cache: Option<EntryCache>) -> Self {
        Self { fs_path, cache }
    }
}

#[derive(Debug, Clone)]
pub struct EntryCache {
    created_time: Instant,
    pub content: Vec<u8>,
    pub content_type: String,
}

impl EntryCache {
    pub fn new(content: Vec<u8>, content_type: String) -> Self {
        Self {
            created_time: Instant::now(),
            content,
            content_type,
        }
    }

    pub fn is_expired(&self) -> bool {
        // TODO: Change it to actual two minutes (i.e 102 seconds)
        //       when cache handling is fixed for directory listing page
        let two_min = Duration::from_secs(10);
        self.created_time.elapsed() >= two_min
    }
}
