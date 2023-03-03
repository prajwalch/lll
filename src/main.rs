use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use tiny_http::{Header, Request, Response, Server};

type UrlsMap = HashMap<String, UrlEntry>;

struct ServerConfig {
    urls_map: UrlsMap,
    mime_types: HashMap<&'static str, &'static str>,
}

impl ServerConfig {
    pub fn new(path_to_serve: &Path) -> Self {
        Self {
            urls_map: Self::build_urls_map(path_to_serve),
            mime_types: Self::build_mime_types(),
        }
    }

    fn build_urls_map(path: &Path) -> UrlsMap {
        let mut urls_map = HashMap::new();

        path.read_dir().unwrap().into_iter().for_each(|dir_entry| {
            let dir_entry = dir_entry.unwrap();

            if dir_entry.file_name() == "index.html" {
                urls_map.insert(
                    String::from("/"),
                    UrlEntry::new(dir_entry.path(), None, None),
                );
                return;
            }
            urls_map.insert(
                format!("/{}", dir_entry.file_name().to_str().unwrap()),
                UrlEntry::new(dir_entry.path(), None, None),
            );
        });

        urls_map
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

    pub fn get_mime_type<E>(&self, file_extension: E) -> String
    where
        E: AsRef<OsStr>,
    {
        let file_extension = file_extension.as_ref();
        let default_mime_type = self.mime_types.get("default").unwrap().to_string();

        self.mime_types
            .get(file_extension.to_str().unwrap())
            .map_or(default_mime_type, |mime_type| {
                format!("Content-Type: {}", mime_type)
            })
    }
}

struct UrlEntry {
    fs_path: PathBuf,
    pre_rendered_page: Option<String>,
    mime_type: Option<String>,
}

impl UrlEntry {
    #[rustfmt::skip]
    pub fn new(fs_path: PathBuf, pre_rendered_page: Option<String>, mime_type: Option<String>) -> Self {
        Self { fs_path, pre_rendered_page, mime_type }
    }
}

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Path is not provided, serving current directory");
        String::from(".")
    });
    let path = PathBuf::from(path);
    debug_assert!(path.is_dir());
    let mut server_config = ServerConfig::new(path.as_path());

    start_server(&mut server_config);
}

fn start_server(server_config: &mut ServerConfig) {
    let server = Server::http("127.0.0.1:8080").unwrap();
    println!("Listening at `http://{}`", server.server_addr());

    for request in server.incoming_requests() {
        handle_request(request, server_config);
    }
}

fn handle_request(request: Request, server_config: &mut ServerConfig) {
    println!("{:?}: {}", request.method(), request.url());

    let requested_url = request.url().to_string();
    let old_url_entry = match server_config.urls_map.get(&requested_url) {
        Some(entry) => entry,
        None => {
            let response = Response::from_string("<h1>404 Not Found</h1>")
                .with_header(Header::from_str(&server_config.get_mime_type("html")).unwrap())
                .with_status_code(404);
            request.respond(response).unwrap();
            return;
        }
    };

    // Respond the cached page
    if let (Some(pre_rendered_page), Some(mime_type)) =
        (&old_url_entry.pre_rendered_page, &old_url_entry.mime_type)
    {
        let res = Response::from_string(pre_rendered_page)
            .with_header(Header::from_str(mime_type).unwrap());
        request.respond(res).unwrap();
        return;
    }

    if old_url_entry.fs_path.is_dir() {
        todo!("Dir url handling")
    }

    let content = fs::read_to_string(old_url_entry.fs_path.as_path()).unwrap();
    let mime_type = server_config.get_mime_type(
        old_url_entry
            .fs_path
            .extension()
            .unwrap_or("default".as_ref()),
    );

    request
        .respond(Response::from_string(&content).with_header(Header::from_str(&mime_type).unwrap()))
        .unwrap();
    // Update the url entry with pre rendered page and its MIME type
    server_config.urls_map.insert(
        requested_url,
        UrlEntry::new(
            old_url_entry.fs_path.clone(), // FIXME: Find a way to avoid cloning here
            Some(content),
            Some(mime_type),
        ),
    );
}
