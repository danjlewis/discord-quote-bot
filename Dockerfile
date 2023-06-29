FROM rust:latest AS build
WORKDIR /usr/src/quote_bot
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=build /usr/local/cargo/bin/quote_bot /usr/local/bin/quote_bot
RUN apt-get update && apt-get install -y ca-certificates
CMD ["quote_bot"]
