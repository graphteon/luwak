FROM rust:latest as build

WORKDIR /src
COPY . .
RUN apt-get update && apt install -y libssl-dev libc6-dev ca-certificates cmake protobuf-compiler
RUN cargo build --release

FROM debian:stable-slim
COPY --from=build /src/target/release/luwak /bin/luwak
RUN apt-get update && apt install -y libssl-dev libc6-dev ca-certificates