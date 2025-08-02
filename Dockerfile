# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.88.0
ARG APP_NAME=youtube-summarizer-server

FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION} AS chef

ARG APP_NAME=youtube-summarizer-server

WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin ${APP_NAME}


FROM python:3.13-slim-bookworm AS final

WORKDIR /app

ARG APP_NAME=youtube-summarizer-server
COPY --from=builder /app/target/release/${APP_NAME} /usr/local/bin

COPY --from=builder /app/public /var/www
ENV PUBLIC_DIR=/var/www

RUN pip install "yt-dlp[default,curl-cffi]" && \
    pip install requests

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser && \
    mkdir -p .cache/yt-dlp/ && \
    chown -R appuser: .cache/yt-dlp/ && \
    mkdir -p /var/transcripts/ && \
    chown -R appuser: /var/transcripts/ && \
    mkdir -p /var/dist/ && \
    chown -R appuser: /var/dist/

USER appuser

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/youtube-summarizer-server"]
