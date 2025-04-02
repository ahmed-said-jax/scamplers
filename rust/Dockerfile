ARG ALPINE_VERSION
ARG RUST_VERSION
ARG APP_NAME=scamplers

FROM rust:${RUST_VERSION}-alpine${ALPINE_VERSION} AS build
ARG APP_NAME
WORKDIR /app

RUN apk add --no-cache clang lld musl-dev git

ENV IN_DOCKER="true"

RUN --mount=type=bind,source=rust/Cargo.toml,target=rust/Cargo.toml \
    --mount=type=bind,source=rust/Cargo.lock,target=rust/Cargo.lock \
    --mount=type=bind,source=rust/.cargo,target=rust/.cargo \
    --mount=type=bind,source=rust/diesel.toml,target=rust/diesel.toml \
    --mount=type=bind,source=rust/scamplers/src,target=rust/scamplers/src \
    --mount=type=bind,source=rust/scamplers/Cargo.toml,target=rust/scamplers/Cargo.toml \
    --mount=type=bind,source=rust/scamplers/diesel.toml,target=rust/scamplers/diesel.toml \
    --mount=type=bind,source=rust/build-scamplers/Cargo.toml,target=rust/build-scamplers/Cargo.toml \
    --mount=type=bind,source=db/migrations,target=db/migrations \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cd rust && \
    cargo build --locked --release && \
    cp ./target/release/${APP_NAME} /bin/server

FROM alpine:${ALPINE_VERSION} AS final

ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
RUN mkdir logs && chown appuser logs
USER appuser

COPY --from=build /bin/server /bin/
COPY --from=scamplers-web /opt/scamplers-web /opt/scamplers-web

EXPOSE 8000

CMD ["/bin/server", "prod", "--log-dir", "logs"]
