FROM rust:latest AS builder
WORKDIR /usr/src/quote_bot
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/quote_bot /usr/local/bin/quote_bot
CMD ["quote_bot"]
