FROM rust:1.39
WORKDIR /usr/src/garbage
COPY . .
RUN pwd
RUN ls -l
RUN cargo build --release

FROM alpine:latest
COPY --from=0 /usr/src/garbage/target/release/garbage .
