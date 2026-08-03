# https://www.lpalmieri.com/posts/fast-rust-docker-builds/

FROM rust:1.97-bookworm AS chef

RUN apt-get update && \
    apt-get install -y protobuf-compiler libpq5 && \
    rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-chef


FROM chef AS planner
WORKDIR /app
COPY ./common ./common
COPY ./server ./server
COPY ./proto  ./proto
RUN printf '[workspace]\nresolver = "2"\nmembers = ["common", "server"]\n' > Cargo.toml

COPY ./Cargo.lock ./Cargo.lock

RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS builder 
RUN git --version
COPY --from=planner /app/recipe.json recipe.json
COPY --from=planner /app/Cargo.toml ./Cargo.toml
COPY --from=planner /app/Cargo.lock ./Cargo.lock

# Build dependencies - this is the caching Docker layer
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY ./common ./common
COPY ./server ./server
COPY ./proto ./proto

RUN cargo build -p server --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y libpq5 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /target/release/rpg-server /app/server/rpg-server
COPY assets/maps/ /app/assets/maps/

EXPOSE 2121

WORKDIR /app/server

CMD ["./rpg-server"]