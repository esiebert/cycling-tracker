# Build time
FROM rust:1.77.1-slim as build

RUN apt-get update && \
    apt-get install -y protobuf-compiler

WORKDIR /app

COPY ./src ./src
COPY ./migrations ./migrations
COPY ./.sqlx ./.sqlx
COPY ./proto ./proto
COPY ./Cargo.lock .
COPY ./Cargo.toml .
COPY ./build.rs .

ENV SQLX_OFFLINE=true
RUN cargo build --release

# Run time
FROM rust:1.77.1-slim

COPY --from=build /app/target/release/cycling-tracker /app/cycling-tracker
COPY ./data/tls /app/data/tls

ENTRYPOINT ["/app/cycling-tracker"]