# chefing
FROM lukemathwalker/cargo-chef:latest-rust-1.76.0 as chef
WORKDIR /app
RUN apt update && apt install -y lld clang

FROM chef AS planner
COPY . .
# Compute a lock-like file for the project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# build project from dependencies
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

# runtime stage
FROM debian:bookworm-slim AS runtime
WORKDIR /app
# install runtime dependencies
RUN apt update -y \
    && apt install -y --no-install-recommends openssl ca-certificates \
    && apt autoremove -y \
    && apt clean -y \
    && rm -rf /var/lib/apt/lists/*
# copy compiled binary from builder stage
COPY --from=builder /app/target/release/rust2prod rust2prod
COPY configuration configuration
COPY static static
ENV APP_ENVIRONMENT production

ENTRYPOINT ["./rust2prod"]
