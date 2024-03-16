# builder stage
FROM rust:1.76.0 AS builder
LABEL authors="richard"

WORKDIR /app
RUN apt update && apt install -y lld clang
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
