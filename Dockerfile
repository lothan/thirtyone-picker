FROM rust:1.91.0

ADD . /code
WORKDIR /code

RUN cargo build --release
CMD ["target/release/thirtyone-picker"]

