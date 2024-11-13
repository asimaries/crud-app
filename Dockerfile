FROM rust:bookworm AS builder

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libpq-dev \
    libssl-dev \
    pkg-config && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .
RUN cargo build --release

FROM debian:bookworm

RUN apt-get update && \
    apt-get install -y libpq-dev

EXPOSE 3000

COPY --from=builder /app/target/release/crud /app

CMD ["/app"]