# rust2prod

## Set up locally

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

Fresh set up:

- `cargo sqlx prepare -- --lib` after updating schemas and commit that to gh
- `doctl apps create --spec=spec.yaml`
- set up env var with email client token: `APP_EMAIL_CLIENT__AUTHORIZATION_TOKEN=<secret>`
- `doctl apps list --format ID`
- `doctl apps update <app_id> --spec=spec.yaml`
