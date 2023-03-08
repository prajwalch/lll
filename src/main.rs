mod config;

use std::env;
use std::fs;
use std::io::Error as IoError;
use std::path::PathBuf;
use std::str::FromStr;

use config::{Config, UrlEntry};
use tiny_http::{Header, Request, Response, Server};

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Path is not provided, serving current directory");
        String::from(".")
    });
    let path = PathBuf::from(path);

    if !path.is_dir() {
        eprintln!("Please provide a directory to serve");
        return;
    }

    let mut config = Config::new(path.as_path());
    if let Err(e) = start_server(&mut config) {
        eprintln!("Internal error: {e}");
    }
}

fn start_server(config: &mut Config) -> Result<(), IoError> {
    // TODO: Combine the error returned by this with IoError and return it too
    let server = Server::http("127.0.0.1:8080").unwrap();
    println!("Listening at `http://{}`", server.server_addr());

    for request in server.incoming_requests() {
        handle_request(request, config)?;
    }
    Ok(())
}

fn handle_request(request: Request, config: &mut Config) -> Result<(), IoError> {
    println!("{:?}: {}", request.method(), request.url());

    let requested_url = normalize_url(request.url());
    let old_url_entry = match config.urls_map.get(&requested_url) {
        Some(entry) => entry,
        None => {
            let response = Response::from_string(config::build_not_found_page())
                .with_header(Header::from_str(&config.get_content_type("html")).unwrap())
                .with_status_code(404);
            return request.respond(response);
        }
    };

    if let (Some(cached_content), Some(content_type)) =
        (&old_url_entry.cached_content, &old_url_entry.content_type)
    {
        let res = Response::from_string(cached_content)
            .with_header(Header::from_str(content_type).unwrap());
        return request.respond(res);
    }

    if old_url_entry.fs_path.is_dir() {
        todo!("Dir url handling")
    }

    let content = fs::read_to_string(old_url_entry.fs_path.as_path())?;
    let content_type = config.get_content_type(
        old_url_entry
            .fs_path
            .extension()
            .unwrap_or("default".as_ref()),
    );
    request.respond(
        Response::from_string(&content).with_header(Header::from_str(&content_type).unwrap()),
    )?;

    // Update the url entry with content and its type
    config.urls_map.insert(
        requested_url,
        UrlEntry::new(
            old_url_entry.fs_path.clone(), // FIXME: Find a way to avoid cloning here
            Some(content),
            Some(content_type),
        ),
    );
    Ok(())
}

fn normalize_url(requested_url: &str) -> String {
    if requested_url.len() > 1 {
        return requested_url
            .trim_end_matches('/')
            .trim_end_matches("index.html")
            .to_string();
    }
    requested_url.to_string()
}
