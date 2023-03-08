use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

type UrlsMap = HashMap<String, UrlEntry>;
type MimeTypes = HashMap<&'static str, &'static str>;

pub const PAGE_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
        body {
            background-color: #0a0a0a;
            color: #f5f5f5;
        }

        a {
            color: #948bff;
        }
    </style>
</head>
<body>
    {content}
</body>
</html>
"#;

#[derive(Debug, Clone)]
pub struct UrlEntry {
    pub fs_path: PathBuf,
    pub cached_content: Option<Vec<u8>>,
    pub content_type: Option<String>,
}

impl UrlEntry {
    pub fn new(
        fs_path: PathBuf,
        cached_content: Option<Vec<u8>>,
        content_type: Option<String>,
    ) -> Self {
        Self {
            fs_path,
            cached_content,
            content_type,
        }
    }
}

pub struct Config<'a> {
    root_path: &'a Path,
    pub urls_map: UrlsMap,
    mime_types: MimeTypes,
}

impl<'a> Config<'a> {
    pub fn new(root_path: &'a Path) -> Self {
        let mut config = Self {
            root_path,
            urls_map: UrlsMap::new(),
            mime_types: build_mime_types(),
        };
        config.build_urls_map(root_path);
        config
    }

    pub fn get_content_type<E>(&self, file_extension: E) -> String
    where
        E: AsRef<OsStr>,
    {
        let file_extension = file_extension.as_ref();
        let default_mime_type =
            format!("Content-Type: {}", self.mime_types.get("default").unwrap());

        self.mime_types
            .get(file_extension.to_str().unwrap())
            .map_or(default_mime_type, |mime_type| {
                format!("Content-Type: {}", mime_type)
            })
    }

    pub fn get_url_entry(&mut self, requested_url: &str) -> Option<&UrlEntry> {
        self.rebuild_urls_map(requested_url);
        self.urls_map.get(requested_url)
    }
}

impl Config<'_> {
    fn fs_path_to_url(&self, fs_path: &Path) -> String {
        dbg!(self.root_path, fs_path);

        let parent = if let Some(parent) = fs_path.parent() {
            if parent == Path::new("") {
                Path::new(".")
            } else {
                parent
            }
        } else {
            return String::from("/");
        };

        let is_root_path = (parent == self.root_path) || (fs_path == self.root_path);
        let basename = if let Some(name) = fs_path.file_name() {
            name.to_str().expect("OsStr should convert into &str")
        } else {
            return String::from("/");
        };

        #[rustfmt::skip]
        let basename = if basename == "index.html" { "" } else { basename };

        if is_root_path {
            return format!("/{basename}");
        }
        dbg!(fs_path, self.root_path);
        let parent = fs_path
            .parent()
            .unwrap()
            .strip_prefix(self.root_path)
            .unwrap()
            .to_str()
            .expect("fs_path's parent should able to convert into &str");

        if basename.is_empty() {
            return format!("/{parent}");
        }
        format!("/{parent}/{basename}")
    }
}

// Methods for `url_maps`
impl Config<'_> {
    fn build_urls_map(&mut self, path: &Path) {
        path.read_dir().unwrap().into_iter().for_each(|dir_entry| {
            let dir_entry = dir_entry.unwrap();
            let entry_fs_path = dir_entry.path();
            let mapped_url = self.fs_path_to_url(&entry_fs_path);
            dbg!(&mapped_url);

            self.urls_map
                .entry(mapped_url)
                .or_insert_with(|| UrlEntry::new(entry_fs_path, None, None));
        });
        let mapped_root_url = self.fs_path_to_url(path);

        if self.urls_map.contains_key(&mapped_root_url) {
            return;
        }
        // If path not contains `index.html` file build a file listing page for it
        self.urls_map.insert(
            mapped_root_url,
            UrlEntry::new(
                PathBuf::new(),
                Some(self.generate_file_listing_page(path)),
                Some(self.get_content_type("html")),
            ),
        );
    }

    fn generate_file_listing_page(&self, path: &Path) -> Vec<u8> {
        let file_list_urls = self
            .urls_map
            .iter()
            .filter_map(|(mapped_url, url_entry)| {
                url_entry.fs_path.parent().and_then(|parent| {
                    if parent == path {
                        Some(mapped_url)
                    } else {
                        None
                    }
                })
            })
            .map(|url| format!(r#"<a href="{}">{}</a><br>"#, url, url))
            .collect::<String>();

        let mut content = String::from("<h1>File Listing</h1><br>");
        content.push_str(&file_list_urls);

        PAGE_TEMPLATE
            .replace("{title}", "File Listing")
            .replace("{content}", &content)
            .into_bytes()
    }

    fn rebuild_urls_map(&mut self, requested_url: &str) {
        let mut fs_path: Option<PathBuf> = None;

        if let Some(url_entry) = self.urls_map.get(requested_url) {
            if url_entry.cached_content.is_some() || url_entry.fs_path.is_file() {
                return;
            }
            fs_path = Some(url_entry.fs_path.clone());
            self.urls_map.remove(requested_url);
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
            if ancestor == self.root_path {
                break;
            }
            self.build_urls_map(ancestor);
        }
    }

    fn url_to_fs_path(&self, requested_url: &str) -> PathBuf {
        if let Some(url_entry) = self.urls_map.get(requested_url) {
            return url_entry.fs_path.clone();
        }
        let root_path = self.root_path.to_path_buf();
        root_path.join(requested_url.strip_prefix('/').unwrap())
    }
}

#[rustfmt::skip]
pub fn generate_not_found_page() -> String {
    PAGE_TEMPLATE
        .replace("{title}", "Error Response")
        .replace("{content}", "<h1>404 Not Found</h1><p>Nothing matches the given URI</p>")
}

fn build_mime_types() -> MimeTypes {
    let mut mime_types = MimeTypes::new();

    mime_types.insert("default", "application/octet-stream");
    mime_types.insert("aac", "audio/aac");
    mime_types.insert("abw", "application/x-abiword");
    mime_types.insert("arc", "application/x-freearc");
    mime_types.insert("avif", "image/avif");
    mime_types.insert("avi", "video/x-msvideo");
    mime_types.insert("azw", "application/vnd.amazon.ebook");
    mime_types.insert("bin", "application/octet-stream");
    mime_types.insert("bmp", "image/bmp");
    mime_types.insert("bz", "application/x-bzip");
    mime_types.insert("bz2", "application/x-bzip2");
    mime_types.insert("cda", "application/x-cdf");
    mime_types.insert("csh", "application/x-csh");
    mime_types.insert("css", "text/css");
    mime_types.insert("csv", "text/csv");
    mime_types.insert("doc", "application/msword");
    mime_types.insert(
        "docx",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    );
    mime_types.insert("eot", "application/vnd.ms-fontobject");
    mime_types.insert("epub", "application/epub+zip");
    mime_types.insert("gz", "application/gzip");
    mime_types.insert("gif", "image/gif");
    mime_types.insert("htm", "text/html");
    mime_types.insert("html", "text/html");
    mime_types.insert("ico", "image/vnd.microsoft.icon");
    mime_types.insert("ics", "text/calendar");
    mime_types.insert("jar", "application/java-archive");
    mime_types.insert("jpeg", "image/jpeg");
    mime_types.insert("jpg", "image/jpeg");
    mime_types.insert("js", "text/javascript");
    mime_types.insert("json", "application/json");
    mime_types.insert("jsonld", "application/ld+json");
    mime_types.insert("mid", "audio/midi");
    mime_types.insert("midi", "audio/x-midi");
    mime_types.insert("mjs", "text/javascript");
    mime_types.insert("mp3", "audio/mpeg");
    mime_types.insert("mp4", "video/mp4");
    mime_types.insert("mpeg", "video/mpeg");
    mime_types.insert("mpkg", "application/vnd.apple.installer+xml");
    mime_types.insert("odp", "application/vnd.oasis.opendocument.presentation");
    mime_types.insert("ods", "application/vnd.oasis.opendocument.spreadsheet");
    mime_types.insert("odt", "application/vnd.oasis.opendocument.text");
    mime_types.insert("oga", "audio/ogg");
    mime_types.insert("ogv", "video/ogg");
    mime_types.insert("ogx", "application/ogg");
    mime_types.insert("opus", "audio/opus");
    mime_types.insert("otf", "font/otf");
    mime_types.insert("png", "image/png");
    mime_types.insert("pdf", "application/pdf");
    mime_types.insert("php", "application/x-httpd-php");
    mime_types.insert("ppt", "application/vnd.ms-powerpoint");
    mime_types.insert(
        "pptx",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
    );
    mime_types.insert("rar", "application/vnd.rar");
    mime_types.insert("rtf", "application/rtf");
    mime_types.insert("sh", "application/x-sh");
    mime_types.insert("svg", "image/svg+xml");
    mime_types.insert("tar", "application/x-tar");
    mime_types.insert("tif", "image/tiff");
    mime_types.insert("tiff", "image/tiff");
    mime_types.insert("ts", "video/mp2t");
    mime_types.insert("ttf", "video/mp2t");
    mime_types.insert("txt", "text/plain");
    mime_types.insert("vsd", "application/vnd.visio");
    mime_types.insert("wav", "audio/wav");
    mime_types.insert("weba", "audio/webm");
    mime_types.insert("webm", "video/webm");
    mime_types.insert("webp", "image/webp");
    mime_types.insert("woff", "font/woff");
    mime_types.insert("woff2", "font/woff2");
    mime_types.insert("xhtml", "application/xhtml+xml");
    mime_types.insert("xls", "application/vnd.ms-excel");
    mime_types.insert(
        "xlsx",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    );
    mime_types.insert("xml", "application/xml");
    mime_types.insert("xul", "application/vnd.mozilla.xul+xml");
    mime_types.insert("zip", "application/zip");
    mime_types.insert("3gp", "video/3gpp");
    mime_types.insert("3g2", "video/3gpp2");
    mime_types.insert("7z", "application/x-7z-compressed");

    mime_types
}
