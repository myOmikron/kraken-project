ARG RUST_VERSION=1.85.0

FROM rust:${RUST_VERSION}-slim-bookworm AS buildrust

WORKDIR /app

RUN <<EOF
apt-get update
apt-get install openssl libssl-dev pkg-config curl -y
apt-get install libprotobuf-dev protobuf-compiler build-essential -y
EOF

RUN --mount=type=bind,source=kraken/,target=kraken/ \
    --mount=type=bind,source=kraken-proto/,target=kraken-proto/ \
    --mount=type=bind,source=sdk/rust-kraken-sdk/,target=sdk/rust-kraken-sdk/ \
    --mount=type=bind,source=leech/,target=leech/ \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cargo build --release --package kraken --features bin --locked
cp ./target/release/kraken /bin/server
EOF


FROM debian:bookworm-slim AS final

RUN <<EOF
apt-get update
apt-get install -y libssl-dev
EOF

# Copy startup script
COPY ./build/webserver/startup.sh /
RUN chmod +x /startup.sh

# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/develop/develop-images/dockerfile_best-practices/   #user
ARG UID=10000
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser

RUN mkdir -p /var/lib/kraken
RUN chown ${UID} -R /var/lib/kraken

# Copy migrations
COPY ./kraken/migrations /migrations

# Copy the executable from the "build" stage.
COPY --from=buildrust /bin/server /bin/

USER appuser

# What the container should run when it is started.
CMD ["/startup.sh"]