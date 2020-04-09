use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpServer, Result};
use listenfd::ListenFd;
use regex::Regex;
use std::env;
use std::path::PathBuf;
use structopt::StructOpt;

const DIR_KEY: &'static str = "WORKING_DIR";
const INDEX_KEY: &'static str = "INDEX_NAME";

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

async fn get_files(req: HttpRequest) -> Result<fs::NamedFile> {
    let dir = env::var(DIR_KEY).unwrap_or(".".to_string());
    let file_path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let path: PathBuf = PathBuf::new().join(dir).join(file_path);
    Ok(fs::NamedFile::open(path)?)
}
async fn get_index(_: HttpRequest) -> Result<fs::NamedFile> {
    let dir = env::var(DIR_KEY).unwrap_or(".".to_string());
    let index_base = env::var(INDEX_KEY).unwrap_or("index.html".to_string());
    //let index_path: PathBuf = PathBuf::from("./index.html");
    let path: PathBuf = PathBuf::new().join(dir).join(index_base);
    Ok(fs::NamedFile::open(path)?)
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
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
