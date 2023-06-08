FROM rust:latest AS builder
WORKDIR /usr/src/discord_quote_bot
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/discord_quote_bot /usr/local/bin/discord_quote_bot
CMD ["discord_quote_bot"]
