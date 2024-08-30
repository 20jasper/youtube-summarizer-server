# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.80.1
ARG APP_NAME=youtube-summarizer-server

FROM rust:${RUST_VERSION} AS build
ARG APP_NAME
WORKDIR /app

# Build the application.
# Leverage a cache mount to /usr/local/cargo/registry/
# for downloaded dependencies, a cache mount to /usr/local/cargo/git/db
# for git repository dependencies, and a cache mount to /app/target/ for
# compiled dependencies which will speed up subsequent builds.
# Leverage a bind mount to the src directory to avoid having to copy the
# source code into the container. Once built, copy the executable to an
# output directory before the cache mounted /app/target is unmounted.
ARG CARGO_CACHE=/usr/local/cargo/registry/
ARG GIT_CACHE=/usr/local/cargo/git/db
ARG TARGET_CACHE=/app/target/
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,id=${APP_NAME}-${TARGET_CACHE},target=${TARGET_CACHE} \
    --mount=type=cache,id=${APP_NAME}-${GIT_CACHE},target=${GIT_CACHE} \
    --mount=type=cache,id=${APP_NAME}-${CARGO_REGISTRY},target=${CARGO_REGISTRY} \
    cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server


FROM python:3.12.5-slim-bookworm AS final

COPY --from=build /bin/server /bin/

RUN pip install yt-dlp==2024.8.6 && \
    pip install -U https://github.com/coletdjnz/yt-dlp-youtube-oauth2/archive/refs/heads/master.zip

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

ARG YOUTUBE_USERNAME
ARG YOUTUBE_PASSWORD
RUN test -n "$YOUTUBE_PASSWORD" || (echo "YOUTUBE_PASSWORD not set" && false) && \
    test -n "$YOUTUBE_USERNAME" || (echo "YOUTUBE_USERNAME not set" && false) && \
    touch /var/.netrc && \
    echo 'machine youtube login ${YOUTUBE_USERNAME} password ${YOUTUBE_PASSWORD}' >> /var/.netrc && \
    chmod 666 /var/.netrc

USER appuser

EXPOSE 8080

CMD ["/bin/server"]
