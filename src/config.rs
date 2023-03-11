use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

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
            background-color: #000000;
            color: #f5f5f5;
            padding: 0px 50px;
        }

        a {
            display: block;
            color: #948bff;
            padding: 2px 10px;
            border-radius: 5px;
            text-decoration: none;
        }
        
        a:hover {
            background-color: #090909;
        }

        ul {
            padding: 10px 0px;
            border-top: 1px solid #1a1a1a;
            border-bottom: 1px solid #1a1a1a;
        }
        
        li {
            list-style-type: none;
        }
        
        li:nth-child(n + 2) {            
            margin-top: 5px;
        }

        svg {
            vertical-align: middle;
        }
    </style>
</head>
<body>
    {content}
</body>
</html>
"#;

const FILE_SVG_ICON: &str = r##"
<svg width="40px" height="40px" viewBox="0 0 1024 1024" class="icon" version="1.1"
    xmlns="http://www.w3.org/2000/svg">
    <path
        d="M576 102.4H268.8c-14.08 0-25.6 11.52-25.6 25.6v742.4c0 14.08 11.52 25.6 25.6 25.6h512c14.08 0 25.6-11.52 25.6-25.6V332.8L576 102.4z"
        fill="#00B2AE" />
    <path
        d="M780.8 908.8H268.8c-21.76 0-38.4-16.64-38.4-38.4V128c0-21.76 16.64-38.4 38.4-38.4h312.32L819.2 327.68V870.4c0 21.76-16.64 38.4-38.4 38.4zM268.8 115.2c-7.68 0-12.8 5.12-12.8 12.8v742.4c0 7.68 5.12 12.8 12.8 12.8h512c7.68 0 12.8-5.12 12.8-12.8V337.92L570.88 115.2H268.8z"
        fill="#231C1C" />
    <path d="M576 307.2c0 14.08 11.52 25.6 25.6 25.6h204.8L576 102.4v204.8z" fill="#008181" />
    <path
        d="M806.4 345.6H601.6c-21.76 0-38.4-16.64-38.4-38.4V102.4c0-5.12 2.56-10.24 7.68-11.52 5.12-2.56 10.24-1.28 14.08 2.56l230.4 230.4c3.84 3.84 5.12 8.96 2.56 14.08-1.28 5.12-6.4 7.68-11.52 7.68zM588.8 133.12V307.2c0 7.68 5.12 12.8 12.8 12.8h174.08L588.8 133.12zM332.8 435.2h371.2v25.6H332.8zM332.8 524.8h371.2v25.6H332.8z"
        fill="#231C1C" />
    <path d="M332.8 614.4h371.2v25.6H332.8z" fill="#231C1C" />
    <path d="M332.8 716.8h371.2v25.6H332.8z" fill="#231C1C" />
</svg>
"##;

const FOLDER_SVG_ICON: &str = r##"
<svg width="40px" height="40px" viewBox="0 0 1024 1024" class="icon" version="1.1"
        xmlns="http://www.w3.org/2000/svg" fill="#000000">
        <g id="SVGRepo_bgCarrier" stroke-width="0"></g>
        <g id="SVGRepo_tracerCarrier" stroke-linecap="round" stroke-linejoin="round"></g>
        <g id="SVGRepo_iconCarrier">
            <path
                d="M563.2 358.4c0 14.08-11.52 25.6-25.6 25.6H153.6c-14.08 0-25.6-11.52-25.6-25.6V166.4c0-14.08 11.52-25.6 25.6-25.6h230.4c14.08 0 25.6 11.52 25.6 25.6l153.6 192z"
                fill="#D3AC51"></path>
            <path
                d="M537.6 396.8H153.6c-21.76 0-38.4-16.64-38.4-38.4V166.4c0-21.76 16.64-38.4 38.4-38.4h230.4c19.2 0 35.84 14.08 38.4 33.28l153.6 192v5.12c0 21.76-16.64 38.4-38.4 38.4zM153.6 153.6c-7.68 0-12.8 5.12-12.8 12.8v192c0 7.68 5.12 12.8 12.8 12.8h384c5.12 0 10.24-3.84 12.8-8.96L396.8 171.52V166.4c0-7.68-5.12-12.8-12.8-12.8H153.6z"
                fill="#231C1C"></path>
            <path
                d="M921.6 768c0 14.08-11.52 25.6-25.6 25.6H153.6c-14.08 0-25.6-11.52-25.6-25.6V256c0-14.08 11.52-25.6 25.6-25.6h742.4c14.08 0 25.6 11.52 25.6 25.6v512z"
                fill="#FAC546"></path>
            <path
                d="M896 806.4H153.6c-21.76 0-38.4-16.64-38.4-38.4V256c0-21.76 16.64-38.4 38.4-38.4h742.4c21.76 0 38.4 16.64 38.4 38.4v512c0 21.76-16.64 38.4-38.4 38.4zM153.6 243.2c-7.68 0-12.8 5.12-12.8 12.8v512c0 7.68 5.12 12.8 12.8 12.8h742.4c7.68 0 12.8-5.12 12.8-12.8V256c0-7.68-5.12-12.8-12.8-12.8H153.6z"
                fill="#231C1C"></path>
        </g>
    </svg>
"##;

pub struct Config<'a> {
    pub urls_table: UrlsTable<'a>,
    pub mime_types: MimeTypes,
}

impl<'a> Config<'a> {
    pub fn new(root_path: &'a Path) -> Self {
        Self {
            urls_table: UrlsTable::new(root_path),
            mime_types: MimeTypes::new(),
        }
    }
}

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

    fn map_urls_from(&mut self, path: &Path) -> Result<(), String> {
        path.read_dir()
            .map_err(|err| format!("Unable to map urls from `{}`: {}", path.display(), err))?
            .for_each(|dir_entry| {
                let dir_entry = dir_entry.unwrap();
                let entry_fs_path = dir_entry.path();
                let mapped_url = self.fs_path_to_url(&entry_fs_path);
                dbg!(&mapped_url);

                self.table
                    .entry(mapped_url)
                    .or_insert(UrlEntry::new(entry_fs_path, None, None));
            });
        let mapped_root_url = self.fs_path_to_url(path);

        if self.table.contains_key(&mapped_root_url) {
            return Ok(());
        }
        // If path not contains `index.html` file build a file listing page for it
        self.table.insert(
            mapped_root_url,
            UrlEntry::new(
                PathBuf::new(),
                Some(self.generate_file_listing_page(path)),
                Some(String::from("Content-Type: text/html")),
            ),
        );
        Ok(())
    }

    fn fs_path_to_url(&self, fs_path: &Path) -> String {
        dbg!(self.root_path, fs_path);

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
        let basename = fs_path.file_name().map_or(String::new(), |file_name| {
            file_name.to_string_lossy().replace("index.html", "")
        });

        if parent_is_root_path {
            return format!("/{basename}");
        }
        dbg!(fs_path, self.root_path);

        let parent = fs_path
            .parent()
            .unwrap()
            .strip_prefix(self.root_path)
            .unwrap()
            .to_string_lossy();

        if basename.is_empty() {
            return format!("/{parent}");
        }
        format!("/{parent}/{basename}")
    }

    fn generate_file_listing_page(&self, path: &Path) -> Vec<u8> {
        let file_list_urls = self
            .table
            .iter()
            .filter_map(|(mapped_url, url_entry)| {
                url_entry.fs_path.parent().and_then(|parent| {
                    if parent != path {
                        return None;
                    }

                    let inner_text = if url_entry.fs_path.is_dir() {
                        format!("{FOLDER_SVG_ICON} {mapped_url}")
                    } else {
                        format!("{FILE_SVG_ICON} {}", mapped_url.strip_prefix('/').unwrap())
                    };

                    Some((mapped_url, inner_text))
                })
            })
            .map(|(href, inner_text)| format!(r#"<li><a href="{href}">{inner_text}</a></li>"#))
            .collect::<String>();

        let mut content = String::from("<h1>File Listing</h1><br><ul>");
        content.push_str(&file_list_urls);
        content.push_str("</ul>");

        PAGE_TEMPLATE
            .replace("{title}", "File Listing")
            .replace("{content}", &content)
            .into_bytes()
    }

    pub fn update_table(&mut self, requested_url: &str) {
        let mut fs_path: Option<PathBuf> = None;

        if let Some(url_entry) = self.table.get(requested_url) {
            if url_entry.cached_content.is_some() || url_entry.fs_path.is_file() {
                return;
            }
            fs_path = Some(url_entry.fs_path.clone());
            self.table.remove(requested_url);
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

pub struct MimeTypes(HashMap<&'static str, &'static str>);

impl MimeTypes {
    pub fn new() -> Self {
        Self(build_mime_types())
    }

    pub fn get_content_type<E: AsRef<OsStr>>(&self, file_extension: E) -> String {
        let file_extension = file_extension.as_ref();
        let default_mime_type = format!("Content-Type: {}", self.0.get("default").unwrap());

        self.0
            .get(file_extension.to_str().unwrap())
            .map_or(default_mime_type, |mime_type| {
                format!("Content-Type: {mime_type}")
            })
    }
}

#[rustfmt::skip]
pub fn generate_not_found_page() -> String {
    PAGE_TEMPLATE
        .replace("{title}", "Error Response")
        .replace("{content}", "<h1>404 Not Found</h1><p>Nothing matches the given URI</p>")
}

fn build_mime_types() -> HashMap<&'static str, &'static str> {
    let mut mime_types = HashMap::new();

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
