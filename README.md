# thttp

A minimalist static webserver/fileserver written in Rust + Actix

#### WARNING: DO NOT USE IN PRODUCTION, IT IS MEANT FOR THE DEV !

## Usage

`thhtp` defaults on auditing the current (`.`) directory, serving on `0.0.0.0:5050`.

```
USAGE:
    thttp [OPTIONS] [dir]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -h, --host <host>      Server Host [env: THTTP_HOST=]  [default: 0.0.0.0]
    -i, --index <index>    FIle to use as index [env: THTTP_INDEX=]  [default: index.html]
    -p, --port <port>      Server Port [env: THTTP_PORT=]  [default: 5050]

ARGS:
    <dir>    Serving directory file [default: . ] [env: THTTP_DIR=]
```

Options can be specified as enviornment variables, or in a `.env` file, to use in a dockerized image

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

It starts always with one's needs: in this case, to port all dev stack in Rust.
While developing WASM applications I was finding myself always using the `http` python module, `python3 -m "http.server" "8080"`, in order to get to test the application's output.
This tool is a poor man's replacement, but it does its job.
Plus I could extend it should need arise.

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
