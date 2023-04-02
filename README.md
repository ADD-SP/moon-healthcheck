# Moon Healthcheck

An async healthcheck library for Rust.

## Docs

https://docs.rs/moon-healthcheck

## Example CLI

```shell
cargo run -- --help
cargo run -- --protocol http --url "https://www.google.com"
# > https://www.google.com 	 ✅
# > https://www.google.com 	 ✅
# > https://www.google.com 	 ✅
# > https://www.google.com 	 ✅
# > https://www.google.com 	 ✅
# > ^C
```

