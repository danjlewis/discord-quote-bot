FROM rust:latest AS builder
WORKDIR /usr/src/serenity_template
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/serenity_template /usr/local/bin/serenity_template
CMD ["serenity_template"]
