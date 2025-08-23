FROM rust:latest AS builder
WORKDIR /ip_updater

# Install musl toolchain which is needed for static linking.
RUN apt update && apt install -y musl-tools musl-dev lld

# Add rust musl target.
RUN rustup target add x86_64-unknown-linux-musl

COPY src ./src
COPY Cargo.toml .
COPY .cargo/config.toml ./.cargo/
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch
COPY --from=builder /ip_updater/target/x86_64-unknown-linux-musl/release/ip_updater /ip_updater
ENTRYPOINT ["/ip_updater"]