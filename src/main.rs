use std::env;
use std::fs::{DirEntry, File};
use std::path::PathBuf;

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

    start_server(&dirs);
}

fn start_server(dirs: &[DirEntry]) {
    let server = Server::http("127.0.0.1:8080").unwrap();
    println!("Listening at `http://{}`", server.server_addr());

    for request in server.incoming_requests() {
        handle_request(request, dirs);
    }
}

fn handle_request(request: Request, dirs: &[DirEntry]) {
    println!("{:?}: {}", request.method(), request.url());
    let index_file = get_index_file(dirs);

    if request.url() == "/" {
        if let Some(index_file) = index_file {
            let response = Response::from_file(index_file)
                .with_header(Header::from_bytes("Content-Type", "text/html").unwrap());
            request.respond(response).unwrap();
            return;
        }
    }

    let response = Response::from_string("Custom index.html file");
    request.respond(response).unwrap();
}

/// Returns a `index.html` file if given path contains
fn get_index_file(dirs: &[DirEntry]) -> Option<File> {
    dirs.iter().find_map(|e| {
        if e.file_name() == "index.html" {
            Some(File::open(e.path()).unwrap())
        } else {
            None
        }
    })
}
