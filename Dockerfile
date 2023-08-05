FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --bin server --release

FROM ubuntu
COPY --from=builder /app/target/release/server /server
CMD ["/server"]