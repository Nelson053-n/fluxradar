# Образ api для прода. Сборка из корня репозитория:
#   docker build -f infra/api.Dockerfile -t fluxradar-api:<ГГГГММДД> .
# В контекст попадают только Cargo.toml, Cargo.lock и crates/ (см. .dockerignore).

FROM rust:1.95-slim-trixie AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release --locked -p api

# Runtime на python-slim, а не на голом debian: healthcheck прод-compose
# проверяет /api/v1/health через python3 (urllib), curl/wget в образе нет.
FROM python:3.12-slim-trixie
COPY --from=build /src/target/release/api /usr/local/bin/flux-api
USER 65534:65534
EXPOSE 5049
ENTRYPOINT ["/usr/local/bin/flux-api"]
