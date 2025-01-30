FROM rust:1-alpine AS builder
WORKDIR /random-file-server
COPY . .
RUN apk update && apk upgrade
RUN apk add musl-dev
RUN cargo install --path .

FROM alpine
COPY --from=builder /usr/local/cargo/bin/random-file-server /usr/local/bin/random-file-server
CMD ["random-file-server"]
