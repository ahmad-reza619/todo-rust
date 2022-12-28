FROM rust:buster as builder
WORKDIR /code
COPY . .
RUN cargo fetch
RUN cargo build

FROM debian:buster-slim

WORKDIR /code
RUN apt-get update && apt-get install -y lsb-base lsb-release wget gnupg

RUN wget https://dev.mysql.com/get/mysql-apt-config_0.8.22-1_all.deb
RUN dpkg -i mysql-apt-config*

RUN apt update && apt install -y openssl iputils-ping mariadb-server && rm -rf /var/lib/apt/lists/*
COPY --from=builder /code/target/debug/todo-rust /code/todo-rust
COPY --from=builder /code/src/schema /usr/local/bin/schema
ENV RUST_LOG=debug

EXPOSE 3030

CMD ["./todo-rust"]
