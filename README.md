# rust2prod

## Set up locally

- `./scripts/init_db.sh` for fresh setup with docker and migrations
- `SKIP_DOCKER=true ./scripts/init_db.sh` for local setup/migration without docker
- if you want pretty logs, `cargo install bunyan` and pipe application or test through
  it (`cargo watch -x run | bunyan`)
- remember to `DATABASE_URL=CONNECTIONSTRING sqlx migrate run` after changing the schema
- `npm install`

## Run

- `npm start` (we currently use tailwind to generate classes on the fly)
- `cargo watch -x run`

## Test

- `TEST_LOG=true cargo test [testcase]`
- debugging sqlx: `export RUST_LOG="sqlx=error,info"`

### TODOs

- [ ] auto-build frontend assets in docker (currently we have to build it manually and commit it in the repo)
- [ ] add opentelemetry and send to [honeycomb](https://honeycomb.io) or [jaeger](https://www.jaegertracing.io/) (
      see [tracing-opentelemetry](https://docs.rs/tracing-opentelemetry/latest/tracing_opentelemetry/index.html))
- [ ] minimize docker image size wth [rust-musl-bilder](https://github.com/emk/rust-musl-builder)

## Using digital ocean

Fresh set up:

- `cargo sqlx prepare -- --lib` after updating schemas and commit that to gh
- `doctl apps create --spec=spec.yaml`
- set up env var with email client token: `APP_EMAIL_CLIENT__AUTHORIZATION_TOKEN=<secret>`
- you need to temporarily remove trusted sources from DB settings to remotely access the database e.g. for migration or remote debugging with client, or set up a trusted source firewall rule if you are using a cluster https://docs.digitalocean.com/products/databases/postgresql/how-to/secure/#firewalls
- `doctl apps list --format ID`
- `doctl apps update <app_id> --spec=spec.yaml`
