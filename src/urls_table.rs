use std::collections::HashMap;
use std::fmt::Write;
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
            if url_entry.fs_path.is_file() {
                // Url points to a existing file on disk. Nothing to do.
                return Ok(());
            }
            // Url points to a directory on disk.
            let dir = url_entry.fs_path.clone();

            // 1) Remove this url.
            self.table.remove(url);
            // 2) Remove all sub url as well.
            //
            // Eg: url -> /one (remove this too)
            //     remove all `/one/*`
            //
            // Note: This is important to prevent from keeping url that points
            //       to a file or directory that no longer exists on disk.
            self.table
                .retain(|_, entry| entry.fs_path.parent().is_some_and(|p| p != dir));
            // 3) Remap everything.
            return self.map_urls_from(&dir);
        }
        // Url doesn't have any entry in the table. Construct disk path first.
        let fs_path = self.root_path.join(url.trim_start_matches('/'));

        if fs_path.is_file() {
            // If path points to a file then map urls from its parent (dir).
            self.map_urls_from(fs_path.parent().unwrap())
        } else if fs_path.is_dir() {
            // If path points to a dir then map urls from it.
            self.map_urls_from(&fs_path)
        } else {
            // Path doesn't exist on disk or its broken.
            Err(Error::from(ErrorKind::NotFound))
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
            // If path is a root then return early.
            return String::from("/");
        }
        // 1) Remove root from the path.
        //
        // Eg: root = /home/x/one
        //     /home/x/one/main.rs -> /main.rs
        let relative_path = fs_path.strip_prefix(&self.root_path).unwrap_or(fs_path);
        // 2) Convert it to string and normalize it.
        let mut url = normalize_url(&relative_path.to_string_lossy());
        // 3) Add trailing `/` if path points to a dir on disk.
        if fs_path.is_dir() {
            url.push('/');
        }
        url
    }

    fn build_directory_listing_page(&self, url: &str, fs_path: &Path) -> Vec<u8> {
        let mut entries = self
            .table
            .iter()
            .filter(|(_, entry)| entry.fs_path.parent().is_some_and(|p| p == fs_path))
            .collect::<Vec<(&String, &UrlEntry)>>();

        // Sort the entries so that the directories shows up first and then files.
        entries.sort_by_key(|(_, url_entry)| url_entry.fs_path.is_file());

        let links = entries
            .iter()
            .fold(String::new(), |mut output, (url, entry)| {
                let icon = if entry.fs_path.is_dir() {
                    FOLDER_SVG_ICON
                } else {
                    FILE_SVG_ICON
                };

                let basename = entry
                    .fs_path
                    .file_name()
                    .unwrap_or(entry.fs_path.as_ref())
                    .to_string_lossy();

                // NOTE: Unwrapping is completely safe here.
                write!(output, r#"<li><a href="{url}">{icon} {basename}</a></li>"#).unwrap();
                output
            });
        let content = format!("<h1>Directory Listing for {url}</h1>\n<ul>{links}</ul>");

        PAGE_TEMPLATE
            .replace("{title}", "Directory Listing")
            .replace("{content}", &content)
            .into_bytes()
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
