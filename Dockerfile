FROM docker.io/library/ubuntu:22.04
WORKDIR /usr/src/
RUN apt-get update && apt-get --assume-yes install cargo rustc pkg-config libusb-1.0-0-dev libxdo-dev && apt-get clean

WORKDIR /usr/src/streamduck
RUN mkdir -p /target
COPY . .

RUN cargo fetch
RUN cargo build --target-dir /target --offline --tests --verbose

