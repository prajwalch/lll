mod config;

use std::env;
use std::fs;
use std::io::Error as IoError;
use std::path::PathBuf;
use std::str::FromStr;

use crate::config::Config;
use tiny_http::{Header, Request, Response, Server};

fn main() {
    let path = env::args().nth(1).map_or_else(
        || {
            eprintln!("Path is not provided, serving current directory");
            env::current_dir().unwrap_or(PathBuf::from("."))
        },
        PathBuf::from,
    );

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
    dbg!(&requested_url);

    let url_entry = match config.urls_table.get_url_entry(&requested_url) {
        Some(entry) => entry,
        None => {
            let response = Response::from_string(config::generate_not_found_page())
                .with_header(Header::from_str(&config.mime_types.get_content_type("html")).unwrap())
                .with_status_code(404);
            return request.respond(response);
        }
    };

    if let (Some(cached_content), Some(content_type)) =
        (&url_entry.cached_content, &url_entry.content_type)
    {
        // FIXME: Avoid cloning cached content
        let res = Response::from_data(cached_content.clone())
            .with_header(Header::from_str(content_type).unwrap());
        return request.respond(res);
    }

    let content = fs::read(&url_entry.fs_path)?;
    let content_type = config
        .mime_types
        .get_content_type(url_entry.fs_path.extension().unwrap_or("default".as_ref()));

    // FIXME: Avoid cloning content
    request.respond(
        Response::from_data(content.clone()).with_header(Header::from_str(&content_type).unwrap()),
    )?;

    // Update the url entry
    url_entry.cached_content = Some(content);
    url_entry.content_type = Some(content_type);
    Ok(())
}

fn normalize_url(requested_url: &str) -> String {
    if requested_url == "/" {
        return requested_url.to_string();
    }

    requested_url
        .trim_end_matches('/')
        .trim_end_matches("/index.html")
        .to_string()
}
