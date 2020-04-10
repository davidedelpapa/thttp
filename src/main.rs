//! thttp, a minimalist webserver/fileserver in Rust+Actix
//!
//! It starts always with one's needs: in this case, to port all dev stack in Rust.
//! While developing WASM applications I was finding myself always using the
//! `http` python module, `python3 -m "http.server" "8080"`,
//! in order to get to test the application's output.
//! This tool is a poor man's replacement, but it does its job.
//! Plus I could extend it should need arise.
//!
//! # Usage
//!
//! Just run in a shell, in order to get a server in the current location
//!
//! ```Bash
//! thhtp
//! ```
//!
//! It supports options, specifying a working directory,
//! environment variables, and `.env` files.
//! For example:
//!
//! ```Bash
//! thttp -h 127.0.0.1 -p 3000 -i readme.html ./static
//! ```
//!
//! The above line specifies
//!
//! - `-h || --host`: host name. Enivronment/ `.env` variable: THTTP_HOST
//! - `-p || --port`: port number. Enivronment/ `.env` variable: THTTP_PORT
//! - `-i || --index`: name of the html file to use as `index`. Enivronment/ `.env` variable: THTTP_INDEX
//! - `<dir>`: the working directory, to use as base directory of the server.. Enivronment/ `.env` variable: THTTP_DIR

use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpServer, Result};
use dotenv;
use listenfd::ListenFd;
use regex::Regex;
use std::env;
use std::path::PathBuf;
use structopt::StructOpt;

#[doc(hidden)]
const PORT_KEY: &'static str = "THTTP_PORT";
#[doc(hidden)]
const HOST_KEY: &'static str = "THTTP_HOST";
#[doc(hidden)]
const INDEX_KEY: &'static str = "THTTP_INDEX";
#[doc(hidden)]
const DIR_KEY: &'static str = "THTTP_DIR";

#[derive(Debug, StructOpt)]
#[structopt(name = "thttp", about = "Simple http static server")]
struct Opt {
    /// Server Port
    #[structopt(short = "p", long = "port", default_value = "5050", env = PORT_KEY)]
    port: i32,
    /// Server Host
    #[structopt(short = "h", long = "host", default_value = "0.0.0.0", env = HOST_KEY)]
    host: String,
    /// FIle to use as index
    #[structopt(short = "i", long = "index", default_value = "index.html", env = INDEX_KEY)]
    index: String,
    /// Serving directory file [default: . ]
    #[structopt(parse(from_os_str), env = DIR_KEY)]
    dir: Option<PathBuf>,
}

#[doc(hidden)]
async fn get_files(req: HttpRequest) -> Result<fs::NamedFile> {
    let dir = env::var(DIR_KEY).unwrap_or(".".to_string());
    let file_path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let raw_path: PathBuf = PathBuf::new().join(dir).join(file_path);
    let path: PathBuf = match raw_path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            println!("ERR: {} {:?}", e, raw_path);
            raw_path
        }
    };
    Ok(fs::NamedFile::open(path)?)
}

#[doc(hidden)]
async fn get_index(_: HttpRequest) -> Result<fs::NamedFile> {
    let dir = env::var(DIR_KEY).unwrap_or(".".to_string());
    let index_base = env::var(INDEX_KEY).unwrap_or("index.html".to_string());
    let raw_path: PathBuf = PathBuf::new().join(dir).join(index_base);
    let path: PathBuf = match raw_path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            println!("HTTP:404 {} {:?}", e, raw_path);
            raw_path
        }
    };
    Ok(fs::NamedFile::open(path)?)
}

#[doc(hidden)]
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let mut opt = Opt::from_args();
    let index_base: String = opt.index.to_owned();
    let dir = opt.dir.unwrap_or(PathBuf::from("."));
    env::set_var(DIR_KEY, dir.as_os_str());
    env::set_var(INDEX_KEY, index_base.as_str());
    let index: Option<PathBuf> = if dir.join(&index_base).is_file() {
        let serving = Some(dir.clone().join(&index_base));
        println!("Serving {:?}", &serving.as_ref().unwrap());
        serving
    } else {
        println!("No index equivalent found, showing files listing");
        None
    };
    // Simple host validation
    let valid_ip = Regex::new(
        r"^(([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$",
    ).expect("Ip Validation Error");
    if !valid_ip.is_match(&opt.host) {
        println!("Malformed host: {}\tFalling back to 0.0.0.0", &opt.host);
        opt.host = "0.0.0.0".to_string();
    }
    let mut listenfd = ListenFd::from_env(); // Socket
    let routes = move || {
        if index.is_none() {
            App::new().service(fs::Files::new("/", &dir).show_files_listing())
        } else {
            //App::new().service(fs::Files::new("/", &dir).index_file(&index_base))
            App::new()
                .route("/", web::get().to(get_index))
                .route("/{filename:.*}", web::get().to(get_files))
        }
    };
    let mut server = HttpServer::new(routes);
    server = if let Some(listener) = listenfd.take_tcp_listener(0)? {
        println!("Listening on socket through ListenFd");
        server.listen(listener)?
    } else {
        println!("Listening on http://{}:{}/", opt.host, opt.port);
        server.bind(format!("{}:{}", opt.host, opt.port).as_str())?
    };
    server.run().await
}
