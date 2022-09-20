ARG ARCH=
FROM ${ARCH}/debian:stable-slim

WORKDIR /bin
RUN apt-get update && apt install -y libssl-dev libc6-dev ca-certificates

ADD build/luwak-${ARCH} .