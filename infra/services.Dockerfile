# Образ для worker и bot (api собирается отдельно по infra/api.Dockerfile).
# Сборка из корня репозитория:
#   docker build -f infra/services.Dockerfile -t fluxradar-services:<ГГГГММДД> .
# В образе два бинаря; какой запускать — задаётся `command` в compose.
FROM rust:1.95-slim-trixie AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release --locked -p worker -p bot

FROM debian:trixie-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/worker /src/target/release/bot /usr/local/bin/
USER 65534:65534
