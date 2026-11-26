FROM rust:1.93-bookworm AS builder
WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY migrations ./migrations

RUN cargo build --release -p gf-server

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates git libgomp1 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home --home-dir /var/lib/graphfinder graphfinder \
    && mkdir -p /var/lib/graphfinder/models /var/lib/graphfinder/repo-cache \
    && chown -R graphfinder:graphfinder /var/lib/graphfinder

COPY --from=builder /build/target/release/gf-server /usr/local/bin/graphfinder-server

USER graphfinder
WORKDIR /var/lib/graphfinder
ENV GF_MODEL_CACHE_DIR=/var/lib/graphfinder/models \
    GF_REPO_CACHE_ROOT=/var/lib/graphfinder/repo-cache

ENTRYPOINT ["graphfinder-server"]
CMD ["api", "--bind", "0.0.0.0:8080"]
