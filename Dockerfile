# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.80.1
ARG APP_NAME=youtube-summarizer-server

FROM rust:${RUST_VERSION} AS build
ARG APP_NAME
WORKDIR /app

ADD https://github.com/yt-dlp/yt-dlp/releases/download/2024.08.06/yt-dlp_linux /bin/

# Build the application.
# Leverage a cache mount to /usr/local/cargo/registry/
# for downloaded dependencies, a cache mount to /usr/local/cargo/git/db
# for git repository dependencies, and a cache mount to /app/target/ for
# compiled dependencies which will speed up subsequent builds.
# Leverage a bind mount to the src directory to avoid having to copy the
# source code into the container. Once built, copy the executable to an
# output directory before the cache mounted /app/target is unmounted.
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server


FROM debian:12.6-slim AS final

COPY --from=build /bin/server /bin/

COPY --chmod=0755 --from=build /bin/yt-dlp_linux /bin/yt-dlp

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

CMD ["/bin/server"]
