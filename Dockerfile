# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.89.0
ARG APP_NAME=youtube-summarizer-server

FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION} AS chef

ARG APP_NAME=youtube-summarizer-server

WORKDIR /app

FROM python:3.13-slim-bookworm AS pydeps
WORKDIR /opt/py

COPY requirements.txt requirements.txt
# Install into a relocatable prefix we can later copy wholesale
RUN pip install --no-cache-dir --prefix /install -r requirements.txt

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin ${APP_NAME} --features "axiom"


FROM python:3.13-slim-bookworm AS final

WORKDIR /app

ARG APP_NAME=youtube-summarizer-server

ENV XDG_CACHE_HOME=/var/cache
ENV YOUTUBE_CACHE_DIR=${XDG_CACHE_HOME}/yt-dlp
ENV YOUTUBE_OUTPUT_PATH=${YOUTUBE_CACHE_DIR}
ENV APPLICATION_PUBLIC_DIR=/var/www

COPY --from=pydeps /install /usr/local
COPY --from=builder /app/target/release/${APP_NAME} /usr/local/bin
COPY --from=builder /app/public ${APPLICATION_PUBLIC_DIR}

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser && \
    mkdir -p ${YOUTUBE_CACHE_DIR} && \
    chown -R appuser: ${YOUTUBE_CACHE_DIR} && \
    mkdir -p ${YOUTUBE_OUTPUT_PATH} && \
    chown -R appuser: ${YOUTUBE_OUTPUT_PATH}

USER appuser

ENV APPLICATION_PORT=8080
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/youtube-summarizer-server"]
