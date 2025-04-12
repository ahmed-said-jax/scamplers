ARG ALPINE_VERSION=3.21
ARG RUST_VERSION=1.86
ARG APP_PORT

FROM rust:${RUST_VERSION}-alpine${ALPINE_VERSION} AS build
WORKDIR /app

RUN apk add --no-cache clang lld musl-dev git curl

ENV IN_DOCKER="true"

RUN --mount=type=bind,source=rust/Cargo.toml,target=rust/Cargo.toml \
    --mount=type=bind,source=rust/Cargo.lock,target=rust/Cargo.lock \
    --mount=type=bind,source=rust/.cargo,target=rust/.cargo \
    --mount=type=bind,source=rust/scamplers-backend/src,target=rust/scamplers-backend/src \
    --mount=type=bind,source=rust/scamplers-backend/Cargo.toml,target=rust/scamplers-backend/Cargo.toml \
    --mount=type=bind,source=rust/build-scamplers/Cargo.toml,target=rust/build-scamplers/Cargo.toml \
    --mount=type=bind,source=db/migrations,target=db/migrations \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cd rust && \
    cargo build --locked --release && \
    cp ./target/release/scamplers-backend /bin/scamplers-backend

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

COPY --from=build /bin/scamplers-backend /bin/

EXPOSE ${APP_PORT}

CMD ["/bin/scamplers-backend", "prod", "--log-dir", "logs"]
