mod config;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use config::{ServerConfig, UrlEntry};
use tiny_http::{Header, Request, Response, Server};

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
                .with_header(Header::from_str(&server_config.get_content_type("html")).unwrap())
                .with_status_code(404);
            request.respond(response).unwrap();
            return;
        }
    };

    if let (Some(cached_content), Some(content_type)) =
        (&old_url_entry.cached_content, &old_url_entry.content_type)
    {
        let res = Response::from_string(cached_content)
            .with_header(Header::from_str(content_type).unwrap());
        request.respond(res).unwrap();
        return;
    }

    if old_url_entry.fs_path.is_dir() {
        todo!("Dir url handling")
    }

    let content = fs::read_to_string(old_url_entry.fs_path.as_path()).unwrap();
    let content_type = server_config.get_content_type(
        old_url_entry
            .fs_path
            .extension()
            .unwrap_or("default".as_ref()),
    );
    request
        .respond(
            Response::from_string(&content).with_header(Header::from_str(&content_type).unwrap()),
        )
        .unwrap();

    // Update the url entry with content and its type
    server_config.urls_map.insert(
        requested_url,
        UrlEntry::new(
            old_url_entry.fs_path.clone(), // FIXME: Find a way to avoid cloning here
            Some(content),
            Some(content_type),
        ),
    );
}
