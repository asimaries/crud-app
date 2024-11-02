FROM rust:latest

WORKDIR /usr/src/app/

COPY . .

RUN cargo build --release

EXPOSE 3000

CMD ["cargo", "run", "--quiet"]