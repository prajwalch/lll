use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};

use tiny_http::{Header, Request, Response, Server};

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Path is not provided, serving current directory");
        String::from(".")
    });
    let path = PathBuf::from(path);
    debug_assert!(path.is_dir());
    let urls_map = build_urls_map(path.as_path());

    start_server(&urls_map);
}

fn build_urls_map(path: &Path) -> HashMap<String, PathBuf> {
    let mut urls_map = HashMap::new();

    path.read_dir()
        .unwrap()
        .into_iter()
        .map(|entry| entry.unwrap())
        .for_each(|dir_entry| {
            if dir_entry.file_name() == "index.html" {
                urls_map.insert(String::from("/"), dir_entry.path());
                return;
            }
            urls_map.insert(
                format!("/{}", dir_entry.file_name().to_str().unwrap()),
                dir_entry.path(),
            );
        });

    urls_map
}

fn start_server(urls_map: &HashMap<String, PathBuf>) {
    let server = Server::http("127.0.0.1:8080").unwrap();
    println!("Listening at `http://{}`", server.server_addr());

    for request in server.incoming_requests() {
        handle_request(request, urls_map);
    }
}

fn handle_request(request: Request, urls_map: &HashMap<String, PathBuf>) {
    println!("{:?}: {}", request.method(), request.url());

    if let Some(url_path) = urls_map.get(request.url()) {
        let response = Response::from_file(File::open(url_path.as_path()).unwrap());
        request.respond(response).unwrap();
        return;
    }

    let response = Response::from_string("<h1>404 Not Found</h1>")
        .with_header(Header::from_bytes("Content-Type", "text/html").unwrap())
        .with_status_code(404);
    request.respond(response).unwrap();
}
