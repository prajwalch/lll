use std::collections::HashMap;
use std::io;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::common::{FILE_SVG_ICON, FOLDER_SVG_ICON, PAGE_TEMPLATE};
use crate::normalize_url;

pub type Seconds = u64;

const DEFAULT_CACHE_EXPIRATION_TIME: Duration = Duration::from_secs(60);

pub struct UrlsTable {
    root_path: PathBuf,
    table: HashMap<String, UrlEntry>,
    cache_expiration_time: Duration,
}

impl UrlsTable {
    pub fn new(root_path: PathBuf, cache_expiration_time: Option<Seconds>) -> Self {
        let cache_expiration_time =
            cache_expiration_time.map_or(DEFAULT_CACHE_EXPIRATION_TIME, Duration::from_secs);

        Self {
            root_path,
            table: HashMap::new(),
            cache_expiration_time,
        }
    }

    pub fn get_url_entry(&self, url: &str) -> Option<&UrlEntry> {
        self.table.get(url)
    }

    pub fn contains_url_entry(&self, url: &str) -> bool {
        self.table.contains_key(url)
    }

    pub fn update_or_set_cache_of(&mut self, url: &str, content: Vec<u8>, content_type: String) {
        if let Some(entry) = self.table.get_mut(url) {
            entry.cache = Some(EntryCache::new(
                content,
                content_type,
                self.cache_expiration_time,
            ));
        }
    }

    pub fn update_if_needed(&mut self, url: &str) -> io::Result<()> {
        if let Some(url_entry) = self.table.get(url) {
            if url_entry.fs_path.is_file()
                || url_entry.cache.as_ref().is_some_and(|c| !c.is_expired())
            {
                return Ok(());
            }
            // FIXME: Clone is un-necessary here, but required to use inside `retain` function.
            //        Figure out solution to fix this.
            let requested_path = url_entry.fs_path.clone();
            self.table.remove(url);
            // To prevent displaying the deleted file or directory in listing page, remove all urls
            // previously mapped from `requested_path` by keeping only the urls whose equivalent
            // fs_path's parent is not `requested_path`.
            self.table
                .retain(|_, entry| entry.fs_path.parent().map_or(true, |p| p != requested_path));

            return self.map_urls_from(&requested_path);
        }
        let url_fs_path = self.url_to_fs_path(url);

        if !url_fs_path.exists() {
            return Err(Error::from(ErrorKind::NotFound));
        }
        if url_fs_path.is_file() && url_fs_path.parent().is_some_and(|p| p != self.root_path) {
            self.map_urls_from(url_fs_path.parent().unwrap())
        } else {
            self.map_urls_from(&url_fs_path)
        }
    }

    fn map_urls_from(&mut self, path: &Path) -> io::Result<()> {
        let mut found_index_html_file = false;

        for dir_entry in path.read_dir()? {
            let entry_fs_path = dir_entry?.path();

            if !found_index_html_file
                && entry_fs_path.file_name().is_some_and(|f| f == "index.html")
            {
                found_index_html_file = true;
            }
            let mapped_url = self.fs_path_to_url(&entry_fs_path);

            self.table
                .entry(mapped_url)
                .or_insert(UrlEntry::new(entry_fs_path, None));
        }

        if found_index_html_file {
            return Ok(());
        }
        // If path not contains `index.html` file build a directory listing page for it
        let mapped_root_url = self.fs_path_to_url(path);
        let entry_cache = EntryCache::new(
            self.build_directory_listing_page(&mapped_root_url, path),
            String::from("Content-Type: text/html"),
            self.cache_expiration_time,
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
        let child_path = fs_path.strip_prefix(&self.root_path).unwrap_or(fs_path);
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
            .filter(|(_, url_entry)| url_entry.fs_path.parent().is_some_and(|p| p == dir_path))
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

    fn url_to_fs_path(&self, url: &str) -> PathBuf {
        if let Some(url_entry) = self.table.get(url) {
            return url_entry.fs_path.clone();
        }
        self.root_path.join(url.trim_start_matches('/'))
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
    pub content: Vec<u8>,
    pub content_type: String,
    created_time: Instant,
    expiration_time: Duration,
}

impl EntryCache {
    pub fn new(content: Vec<u8>, content_type: String, expiration_time: Duration) -> Self {
        Self {
            content,
            content_type,
            created_time: Instant::now(),
            expiration_time,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created_time.elapsed() >= self.expiration_time
    }
}
