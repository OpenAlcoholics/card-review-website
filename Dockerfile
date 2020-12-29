FROM rust:1.48.0-slim-buster

WORKDIR /var/app

ADD . .

RUN apt update
RUN apt install -y libssl-dev pkg-config

RUN rustup update nightly
RUN cargo -v search --limit 0

RUN cargo +nightly build --release
CMD ./target/release/dgc-review
