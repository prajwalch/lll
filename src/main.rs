use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use tiny_http::{Header, Request, Response, Server};

type UrlsMap = HashMap<String, UrlData>;

struct UrlData {
    fs_path: PathBuf,
    pre_rendered_page: Option<String>,
}

impl UrlData {
    #[rustfmt::skip]
    pub fn new(fs_path: PathBuf, pre_rendered_page: Option<String>) -> Self {
        Self { fs_path, pre_rendered_page }
    }
}

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Path is not provided, serving current directory");
        String::from(".")
    });
    let path = PathBuf::from(path);
    debug_assert!(path.is_dir());
    let mut urls_map = build_urls_map(path.as_path());

    start_server(&mut urls_map);
}

fn build_urls_map(path: &Path) -> UrlsMap {
    let mut urls_map = HashMap::new();

    path.read_dir().unwrap().into_iter().for_each(|dir_entry| {
        let dir_entry = dir_entry.unwrap();

        if dir_entry.file_name() == "index.html" {
            urls_map.insert(String::from("/"), UrlData::new(dir_entry.path(), None));
            return;
        }
        urls_map.insert(
            format!("/{}", dir_entry.file_name().to_str().unwrap()),
            UrlData::new(dir_entry.path(), None),
        );
    });

    urls_map
}

fn start_server(urls_map: &mut UrlsMap) {
    let server = Server::http("127.0.0.1:8080").unwrap();
    println!("Listening at `http://{}`", server.server_addr());

    for request in server.incoming_requests() {
        handle_request(request, urls_map);
    }
}

fn handle_request(request: Request, urls_map: &mut UrlsMap) {
    println!("{:?}: {}", request.method(), request.url());

    let url_data = match urls_map.get_mut(request.url()) {
        Some(data) => data,
        None => {
            let response = Response::from_string("<h1>404 Not Found</h1>")
                .with_header(Header::from_str("Content-Type: text/html").unwrap())
                .with_status_code(404);
            request.respond(response).unwrap();
            return;
        }
    };

    if let Some(ref pre_rendered_page) = url_data.pre_rendered_page {
        let res = Response::from_string(pre_rendered_page)
            .with_header(Header::from_str("Content-Type: text/html").unwrap());
        request.respond(res).unwrap();
        return;
    }

    if url_data.fs_path.is_dir() {
        todo!("Dir url handling")
    }
    let content = fs::read_to_string(url_data.fs_path.as_path()).unwrap();
    request.respond(Response::from_string(&content)).unwrap();
    url_data.pre_rendered_page = Some(content);
}
