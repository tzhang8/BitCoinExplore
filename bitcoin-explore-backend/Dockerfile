FROM rust:1.70 as builder

WORKDIR /app

COPY Cargo.toml .
COPY src ./src

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y \
    sqlite3 \
    libsqlite3-dev \
    libssl1.1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /usr/local/bin

COPY --from=builder /app/target/release/bitcoin-explore-backend /usr/local/bin/bitcoin-explore-backend

RUN chmod +x /usr/local/bin/bitcoin-explore-backend

EXPOSE 8080

CMD ["bitcoin-explore-backend"]
