FROM rust:1.75-slim
WORKDIR /app
COPY . .
RUN rm -f Cargo.lock         
RUN cargo build --release
EXPOSE 8080 9898
CMD ["./target/release/rust-server"]
