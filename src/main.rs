use std::fs::DirEntry;
use std::path::PathBuf;
use std::{env, fs};

use tiny_http::{Header, Request, Response, Server};

fn main() {
    let path = env::args().nth(1).unwrap_or({
        eprintln!("Path is not provided, serving current directory");
        String::from(".")
    });
    let path = PathBuf::from(path);
    debug_assert!(path.is_dir());
    let dirs = path
        .read_dir()
        .unwrap()
        .into_iter()
        .map(|entry| entry.unwrap())
        .collect::<Vec<DirEntry>>();
    let index_page = get_index_page(&dirs);

    start_server(index_page);
}

/// Returns a content of `index.html` file if given path contains it
fn get_index_page(dirs: &[DirEntry]) -> Option<String> {
    dirs.iter().find_map(|e| {
        if e.file_name() == "index.html" {
            Some(fs::read_to_string(e.path()).unwrap())
        } else {
            None
        }
    })
}

fn start_server(index_page: Option<String>) {
    let server = Server::http("127.0.0.1:8080").unwrap();
    println!("Listening at `http://{}`", server.server_addr());

    for request in server.incoming_requests() {
        handle_request(request, index_page.clone());
    }
}

fn handle_request(request: Request, index_page: Option<String>) {
    println!("{:?}: {}", request.method(), request.url());

    if request.url() == "/" {
        if let Some(index_page) = index_page {
            let response = Response::from_string(index_page)
                .with_header(Header::from_bytes("Content-Type", "text/html").unwrap());
            request.respond(response).unwrap();
            return;
        }
    }

    let response = Response::from_string("Custom index.html file");
    request.respond(response).unwrap();
}
