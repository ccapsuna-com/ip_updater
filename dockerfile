FROM --platform=${BUILDPLATFORM} rust:latest AS builder
ARG TARGETARCH
# ARG BUILDPLATFORM
WORKDIR /ip_updater

# Install musl toolchain which is needed for static linking.
RUN apt update && apt install -y musl-tools musl-dev lld

# Determine the Rust target based on TARGETARCH
RUN RUST_TARGET=$(if [ "$TARGETARCH" = "amd64" ]; then echo "x86_64-unknown-linux-musl"; elif [ "$TARGETARCH" = "arm64" ]; then echo "aarch64-unknown-linux-musl"; else echo "Unsupported architecture: $TARGETARCH" >&2; exit 1; fi) && \
    rustup target add $RUST_TARGET && \
    echo $RUST_TARGET > /rust_target.txt

COPY src ./src
COPY Cargo.toml .
COPY .cargo/config.toml ./.cargo/
RUN RUST_TARGET=$(cat /rust_target.txt) && \
    cargo build --target $RUST_TARGET --release && \
    cp target/$RUST_TARGET/release/ip_updater /binary

# RUN echo $TARGETARCH
# RUN echo $BUILDPLATFORM
# RUN echo $(cat /rust_target.txt)

FROM scratch
COPY --from=builder /binary /ip_updater
ENTRYPOINT ["/ip_updater"]