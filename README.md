# Rust2Prod


## Set up

- `./scripts/init_db.sh` for fresh setup with docker and migrations
- `SKIP_DOCKER=true ./scripts/init_db.sh` for local setup without docker

- `cargo install bunyan`

## Run 

`cargo watch -x run | bunyan`

## Test

- `TEST_LOG=true cargo test [testcase] | bunyan`