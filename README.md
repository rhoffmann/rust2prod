# rust2prod

## Set up

- `./scripts/init_db.sh` for fresh setup with docker and migrations
- `SKIP_DOCKER=true ./scripts/init_db.sh` for local setup/migration without docker
- if you want pretty logs, `cargo install bunyan` and pipe application or test through
  it (`cargo watch -x run | bunyan`)
- remember to `DATABASE_URL=CONNECTIONSTRING sqlx migrate run` after changing the schema

## Run

`cargo watch -x run`

## Test

`TEST_LOG=true cargo test [testcase]`

### TODOs

- [ ] add opentelemetry and send to [honeycomb](https://honeycomb.io) or [jaeger](https://www.jaegertracing.io/) (
      see [tracing-opentelemetry](https://docs.rs/tracing-opentelemetry/latest/tracing_opentelemetry/index.html))
- [ ] minimize docker image size wth [rust-musl-bilder](https://github.com/emk/rust-musl-builder)

## Using digital ocean

- `doctl apps create --spec=spec.yaml`
- `doctl apps list`
- `doctl apps update <app_id> --spec=spec.yaml`

- migrate local
