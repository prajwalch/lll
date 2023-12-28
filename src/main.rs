mod common;
mod mime_types;

use std::error::Error;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{env, fs, io};

use crate::mime_types::MimeTypes;

use getopts::Options;
use tiny_http::{Header, Request, Response, Server};

const DEFAULT_PORT: u16 = 2058;

struct LServer {
    root: PathBuf,
    port: u16,
    mime: MimeTypes,
    cache_age: u64,
}

impl LServer {
    pub fn new(root: PathBuf, port: u16, cache_age: u64) -> LServer {
        LServer {
            root,
            port,
            mime: MimeTypes::new(),
            cache_age,
        }
    }

    pub fn start(&self) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let server = Server::http(("127.0.0.1", self.port))?;
        println!("Listening at `http://{}`", server.server_addr());

        for request in server.incoming_requests() {
            self.handle_request(request)?;
        }
        Ok(())
    }

    fn handle_request(&self, req: Request) -> io::Result<()> {
        println!("{:?}: {}", req.method(), req.url());
        let url = normalize_url(req.url());

        if url != req.url() {
            // If normalized url is not the same as requested url then redirect
            // to it first.
            return self.redirect(req, &url);
        }
        // Convert url to fs path.
        let mut path = self.root.join(url.trim_start_matches('/'));

        if !path.exists() {
            return self.respond_html(req, common::build_not_found_page(), 404);
        } else if path.is_dir() {
            // Extend path with index page and check if exists.
            //
            // Eg: /home/x/one -> /home/x/one/index.html.
            path.push("index.html");

            if !path.exists() {
                // Index page doesn't exists. Truncate path back to directory.
                //
                // Eg: /home/x/one/index.html -> /home/x/one.
                path.pop();
                // Generate listing page and respond that.
                return self.respond_html(
                    req,
                    common::build_directory_listing_page(&url, &self.root, &path)?,
                    200,
                );
            }
            // If index page exists then following code will handle the rest.
        }
        self.respond_file(req, &path)
    }

    fn redirect(&self, request: Request, url: &str) -> io::Result<()> {
        request
            .respond(Response::empty(301).with_header(Header::from_bytes("Location", url).unwrap()))
    }

    fn respond_html(&self, req: Request, data: String, status: u32) -> io::Result<()> {
        let content_type = self.mime.get_content_type("html");
        let cache_control = format!("Cache-Control: max-age={}", self.cache_age);

        let res = Response::from_string(data)
            .with_status_code(status)
            .with_header(Header::from_str(&content_type).unwrap())
            .with_header(Header::from_str(&cache_control).unwrap());

        req.respond(res)
    }

    fn respond_file(&self, req: Request, path: &Path) -> io::Result<()> {
        assert!(path.is_file());

        let content = fs::read(path)?;
        let content_type = self.mime.get_content_type(path.extension().unwrap());
        let cache_control = format!("Cache-Control: max-age={}", self.cache_age);

        let res = Response::from_data(content)
            .with_header(Header::from_str(&content_type).unwrap())
            .with_header(Header::from_str(&cache_control).unwrap());

        req.respond(res)
    }
}

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

    let Ok(cache_expiration_time) = args.opt_get_default("cache-exp-time", 60) else {
        eprintln!("Error: Given Maximum cache expiration time is not valid");
        return;
    };
    println!("Serving {path:?} directory");

    let lserver = LServer::new(path, port, cache_expiration_time);

    if let Err(e) = lserver.start() {
        eprintln!("Internal error: {e}");

        #[cfg(debug_assertions)]
        if let Some(source) = e.source() {
            dbg!(source);
        }
    }
}

fn normalize_url(url: &str) -> String {
    let mut normalized_url = String::from('/');

    // Split the url into many components, skip which we don't want and use the
    // rest to make full normalized url.
    //
    // A component roughly corresponds to a each path of url with trailing
    // slash (/) and query parameter.
    //
    // Example: /one/two/index.html     -> [/, one/, two/, index.html]
    //          /one/two/?search=hello  -> [/, one/, two/, ?search=hello]
    let filtered_components = url
        .split_inclusive('/')
        .filter(|&comp| comp != "/" && comp != "index.html" && !comp.starts_with('?'))
        .map(|comp| comp.find('?').map_or(comp, |i| &comp[..i]));

    normalized_url.extend(filtered_components);
    normalized_url
}
