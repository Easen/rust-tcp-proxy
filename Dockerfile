FROM rust:alpine
RUN apk add --no-cache musl-dev
WORKDIR /app/src
COPY . .
RUN cargo install --path .
CMD ["rust-tcp-proxy"]