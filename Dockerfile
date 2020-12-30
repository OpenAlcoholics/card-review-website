FROM frolvlad/alpine-glibc:alpine-3.12_glibc-2.32

WORKDIR /var/app

COPY ./target/release/dgc-review /var/app/dgc-review

CMD ./dgc-review
