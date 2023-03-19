use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::utils::{FILE_SVG_ICON, FOLDER_SVG_ICON, PAGE_TEMPLATE};

pub struct UrlsTable<'a> {
    root_path: &'a Path,
    table: HashMap<String, UrlEntry>,
}

impl<'a> UrlsTable<'a> {
    pub fn new(root_path: &'a Path) -> Self {
        let mut urls_table = Self {
            root_path,
            table: HashMap::new(),
        };
        if let Err(e) = urls_table.map_urls_from(root_path) {
            eprintln!("{e}");
        }
        urls_table
    }

    pub fn get_url_entry_mut(&mut self, requested_url: &str) -> Option<&mut UrlEntry> {
        self.update_table(requested_url);
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
            self.build_directory_listing_page(path),
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

        let parent = if let Some(parent) = fs_path.parent() {
            if parent == Path::new("") {
                Path::new(".")
            } else {
                parent
            }
        } else {
            return String::from("/");
        };

        let parent_is_root_path = parent == self.root_path;
        let mut basename = fs_path.file_name().map_or(String::new(), |file_name| {
            file_name.to_string_lossy().to_string()
        });

        let url_path = if fs_path.is_dir() {
            basename.push('/');
            basename
        } else if basename == "index.html" {
            String::new()
        } else {
            basename
        };

        if parent_is_root_path {
            return format!("/{url_path}");
        }
        let parent = parent
            .strip_prefix(self.root_path)
            .unwrap()
            .to_string_lossy();

        format!("/{parent}/{url_path}")
    }

    fn build_directory_listing_page(&self, path: &Path) -> Vec<u8> {
        let file_list_urls = self
            .table
            .iter()
            .filter_map(|(mapped_url, url_entry)| {
                if url_entry.fs_path.parent()? != path {
                    return None;
                }

                let icon = if url_entry.fs_path.is_dir() {
                    FOLDER_SVG_ICON
                } else {
                    FILE_SVG_ICON
                };
                let basename = url_entry.fs_path.file_name().unwrap().to_string_lossy();
                let inner_text = format!("{icon} {basename}");

                Some(format!(
                    r#"<li><a href="{mapped_url}">{inner_text}</a></li>"#
                ))
            })
            .collect::<String>();

        let mut content = String::from("<h1>Directory Listing</h1><ul>");
        content.push_str(&file_list_urls);
        content.push_str("</ul>");

        PAGE_TEMPLATE
            .replace("{title}", "Directory Listing")
            .replace("{content}", &content)
            .into_bytes()
    }

    fn update_table(&mut self, requested_url: &str) {
        let mut fs_path: Option<PathBuf> = None;

        if let Some(url_entry) = self.table.get(requested_url) {
            if url_entry.fs_path.is_file() {
                return;
            }

            if let Some(ref cache) = url_entry.cache {
                if !cache.is_expired() {
                    return;
                }
            }
            let parent = url_entry.fs_path.clone();

            self.table.remove(requested_url);
            // To prevent displaying the deleted file or directory in listing page, remove all urls
            // mapped from `parent` by keeping only the urls whose parent directory is not `parent`
            self.table
                .retain(|_, entry| entry.fs_path.parent().map_or(true, |p| p != parent));

            fs_path = Some(parent);
        }
        let fs_path = fs_path.unwrap_or(self.url_to_fs_path(requested_url));

        if !fs_path.exists() {
            return;
        }
        let parent = if fs_path.is_file() {
            fs_path.parent().unwrap().to_path_buf()
        } else {
            fs_path
        };

        for ancestor in parent.ancestors() {
            if let Some(root_path_parent) = self.root_path.parent() {
                if ancestor == root_path_parent {
                    break;
                }
            }
            if let Err(e) = self.map_urls_from(ancestor) {
                eprintln!("{e}");
            }
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
