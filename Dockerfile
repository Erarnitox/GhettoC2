FROM messense/rust-musl-cross:x86_64-musl as builder
ENV SQLX_OFFLINE=true
WORKDIR /deploy

# Build the stuffs
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# Deploy the stuffs
FROM scratch
COPY --from=builder /deploy/target/x86_64-unknown-linux-musl/release/backend /backend
ENTRYPOINT ["backend"]
EXPOSE 3000

