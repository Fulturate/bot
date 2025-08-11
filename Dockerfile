FROM rust:1.89

WORKDIR /app

COPY . .

RUN cargo build

CMD ["cargo", "run"]
