# thttp

A minimalist static webserver written in Rust + Actix

#### WARNING: DO NOT USE IN PRODUCTION!!!

## Usage

`thhtp` defaults on auditing the current (`.`) directory, serving on `0.0.0.0:5050`.

```
USAGE:
    thttp [OPTIONS] [dir]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -h, --host <host>      Server Host [default: 0.0.0.0]
    -i, --index <index>    FIle to use as index [default: index.html]
    -p, --port <port>      Server Port [default: 5050]

ARGS:
    <dir>    Serving directory file [default: . ]
```

## Build & Run

The usual cargo combination:

```bash
cargo build --release
cargo run
```

The release is created in _target/release/_.

Please notice that in order to pass arguments to `cargo run` you must inject a double dash (`--`) between `cargo run` and `thttp` parameters.
For example:

```bash
cargo run -- -i readme.html -h 127.0.0.1 -p 7000 ./static
```

**The server stops with CTRL+C**

## Rationale

I found myself using `python3 -m "http.server" "8080"` to run a basic server, in order to develop a basic WASM environment. I just wanted to have the whole stack in Rust.

## Behavior

The `thttp` executable will look first for a socket, through [listenfd](https://github.com/mitsuhiko/rust-listenfd), if not present it will listen normally to the `host:port`.

This is useful for production environments, when using [cargo-watch](https://github.com/passcod/cargo-watch). Just use it in combination with [systemfd](https://github.com/mitsuhiko/systemfd)

```bash
systemfd --no-pid -s http::0.0.0.0:5050 -- cargo watch -x run
```

See also [Actix's docs](https://actix.rs/docs/autoreload/) on the issue.

## Todo (maybe)

Using `cargo-watch` for the projects files is well and good, but when it's not needed, `thttp` is missing the function to watch over the filesystem and reload itself for any change. This would be a good next step for the development.

## License

MIT or Apache-2.0
