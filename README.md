## Getting started

### Server

Watch `server/src` and `shared` for server-related source changes, and rerun server:

```watchexec -w server/src -w shared -r -e rs -- cargo run -p server```

### Client

Watch `src` and `shared` for client-related source changes, and rebuild deployable:

#### Live

```watchexec -w src -w shared -r -e rs -- wasm-pack build --target web --debug --out-name maginet_aee75fc --out-dir static/js/pkg -- --features deploy```

#### Local

If running locally via the tunnel, do _not_ enable the `deploy` feature:

```watchexec -w src -w shared -r -e rs -- wasm-pack build --target web --debug --out-name maginet_aee75fc --out-dir static/js/pkg```
