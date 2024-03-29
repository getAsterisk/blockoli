FROM rust:1.54 as builder

WORKDIR /usr/app

COPY Cargo.toml ./
COPY src ./src

COPY get_grammar.sh ./
RUN chmod +x get_grammar.sh
RUN ./get_grammar.sh

RUN cargo build --release

RUN rm ./target/release/deps/blockoli*
RUN cargo build --release

FROM debian:buster-slim

RUN apt-get update && apt-get install -y openssl

COPY --from=builder /usr/app/target/release/blockoli /usr/local/bin

CMD ["blockoli"]