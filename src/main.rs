mod common;
mod mime_types;
mod urls_table;

use std::env;
use std::error::Error;
use std::fs;
use std::io::Error as IoError;
use std::path::PathBuf;
use std::str::FromStr;

use getopts::Options;

use crate::mime_types::MimeTypes;
use crate::urls_table::{Seconds, UrlsTable};

use tiny_http::{Header, Request, Response, Server};

const DEFAULT_PORT: u16 = 2058;

fn main() {
    let mut opts = Options::new();
    opts.optopt("d", "dir", "directory to serve (default: current)", "PATH");
    opts.optopt("p", "port", "Port to bind (default: 2058)", "PORT_NUM");
    opts.optopt(
        "t",
        "cache-exp-time",
        "cache expiration time in seconds (default: 60)",
        "SECS",
    );
    opts.optflag("h", "help", "Display help and exit");

    let args = match opts.parse(env::args().skip(1)) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };

    if args.opt_present("help") {
        println!("{}", opts.usage("Usage: lll [options]"));
        return;
    }
    let current_dir = env::current_dir().unwrap_or(PathBuf::from("."));
    let path = args.opt_get_default("dir", current_dir).unwrap();
    let path = path.canonicalize().unwrap_or(path);

    if !path.is_dir() {
        eprintln!("Please provide a directory to serve");
        return;
    }

    let Ok(port) = args.opt_get_default("port", DEFAULT_PORT) else {
        eprintln!("error: Given port is not valid");
        return;
    };

    let Ok(cache_expiration_time) = args.opt_get::<Seconds>("cache-exp-time") else {
        eprintln!("Error: Given Maximum cache expiration time is not valid");
        return;
    };
    println!("Serving {path:?} directory");

    let mut urls_table = UrlsTable::new(path, cache_expiration_time);
    let mime_types = MimeTypes::new();

    if let Err(e) = start_server(port, &mut urls_table, &mime_types) {
        eprintln!("Internal error: {e}");

        #[cfg(debug_assertions)]
        if let Some(source) = e.source() {
            dbg!(source);
        }
    }
}

fn start_server(
    port: u16,
    urls_table: &mut UrlsTable,
    mime_types: &MimeTypes,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let server = Server::http(("127.0.0.1", port))?;
    println!("Listening at `http://{}`", server.server_addr());

    for request in server.incoming_requests() {
        handle_request(request, urls_table, mime_types)?;
    }
    Ok(())
}

fn handle_request(
    request: Request,
    urls_table: &mut UrlsTable,
    mime_types: &MimeTypes,
) -> Result<(), IoError> {
    println!("{:?}: {}", request.method(), request.url());

    let mut requested_url = normalize_url(request.url());
    urls_table.update_if_needed(&requested_url).ok();

    let url_entry = match urls_table.get_url_entry(&requested_url) {
        Some(entry) => entry,
        None => {
            let response = Response::from_string(common::build_not_found_page())
                .with_header(Header::from_str(&mime_types.get_content_type("html")).unwrap());

            // Put a trailing slash to url if not present and response `301 Moved Permanently`
            // if we have a url entry associated with that url
            //
            // (see: https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/301)
            if !requested_url.ends_with('/') {
                requested_url.push('/');

                if urls_table.contains_url_entry(&requested_url) {
                    let response = response
                        .with_header(Header::from_bytes("Location", requested_url).unwrap())
                        .with_status_code(301);
                    return request.respond(response);
                }
            }
            return request.respond(response.with_status_code(404));
        }
    };

    if let Some(ref cache) = url_entry.cache {
        if !cache.is_expired() {
            let res = Response::from_data(cache.content.clone())
                .with_header(Header::from_str(&cache.content_type).unwrap());
            return request.respond(res);
        }
    }

    let content = fs::read(&url_entry.fs_path)?;
    let content_type =
        mime_types.get_content_type(url_entry.fs_path.extension().unwrap_or("default".as_ref()));

    request.respond(
        Response::from_data(content.clone()).with_header(Header::from_str(&content_type).unwrap()),
    )?;
    urls_table.update_or_set_cache_of(&requested_url, content, content_type);
    Ok(())
}

fn normalize_url(url: &str) -> String {
    let mut normalized_url = String::from('/');

    // Split the url into many components, skip which we don't want and use the rest to make
    // full normalized url.
    //
    // A component roughly corresponds to a each path of url with trailing slash (/) and query parameter.
    // Example: /one/two/index.html     -> [/, one/, two/, index.html]
    //          /one/two/?search=hello  -> [/, one/, two/, ?search=hello]
    normalized_url.extend(url.split_inclusive('/').filter(|component| {
        *component != "/" && *component != "index.html" && !component.starts_with('?')
    }));
    normalized_url
}
