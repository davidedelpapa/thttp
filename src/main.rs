use actix_files as fs;
use actix_web::{App, HttpServer};
use listenfd::ListenFd;
use regex::Regex;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "thttp", about = "Simple http static server")]
struct Opt {
    /// Server Port
    #[structopt(short = "p", long = "port", default_value = "5050")]
    port: i32,
    /// Server Host
    #[structopt(short = "h", long = "host", default_value = "0.0.0.0")]
    host: String,
    /// FIle to use as index
    #[structopt(short = "i", long = "index", default_value = "index.html")]
    index: String,
    /// Serving directory file [default: . ]
    #[structopt(parse(from_os_str))]
    dir: Option<PathBuf>,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let mut opt = Opt::from_args();
    let index_base: String = opt.index.to_owned();
    // Directory to listen to and index
    let dir = opt.dir.unwrap_or(PathBuf::from("."));
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
    let closure = move || {
        if index.is_none() {
            App::new().service(fs::Files::new("/", &dir).show_files_listing())
        } else {
            App::new().service(fs::Files::new("/", &dir).index_file(&index_base))
        }
    };
    let mut server = HttpServer::new(closure);
    server = if let Some(listener) = listenfd.take_tcp_listener(0)? {
        println!("Listening on socket through ListenFd");
        server.listen(listener)?
    } else {
        println!("Listening on {}:{}", opt.host, opt.port);
        server.bind(format!("{}:{}", opt.host, opt.port).as_str())?
    };
    server.run().await
}
