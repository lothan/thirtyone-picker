FROM rust:1.91.0

ADD . /code
WORKDIR /code

RUN cargo install --path .
CMD ["target/release/thirtyone-picker"]

