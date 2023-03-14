mod mime_types;
mod urls_table;
mod utils;

use std::env;
use std::error::Error;
use std::fs;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::mime_types::MimeTypes;
use crate::urls_table::UrlsTable;

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

    if let Err(e) = start_server(&path) {
        eprintln!("Internal error: {e}");

        #[cfg(debug_assertions)]
        if let Some(source) = e.source() {
            dbg!(source);
        }
    }
}

fn start_server(root_path: &Path) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let server = Server::http("127.0.0.1:8080")?;
    println!("Listening at `http://{}`", server.server_addr());

    let mut urls_table = UrlsTable::new(root_path);
    let mime_types = MimeTypes::new();

    for request in server.incoming_requests() {
        handle_request(request, &mut urls_table, &mime_types)?;
    }
    Ok(())
}

fn handle_request(
    request: Request,
    urls_table: &mut UrlsTable,
    mime_types: &MimeTypes,
) -> Result<(), IoError> {
    println!("{:?}: {}", request.method(), request.url());

    let requested_url = normalize_url(request.url());
    dbg!(&requested_url);

    let url_entry = match urls_table.get_url_entry_mut(&requested_url) {
        Some(entry) => entry,
        None => {
            let response = Response::from_string(utils::build_not_found_page())
                .with_header(Header::from_str(&mime_types.get_content_type("html")).unwrap())
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
    let content_type =
        mime_types.get_content_type(url_entry.fs_path.extension().unwrap_or("default".as_ref()));

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
