FROM rust:1.76.0
LABEL authors="richard"

WORKDIR /app
RUN apt update && apt install -y lld clang

COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release
ENV APP_ENVIRONMENT production

ENTRYPOINT ["/app/target/release/rust2prod"]
