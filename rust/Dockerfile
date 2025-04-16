ARG RUST_VERSION=1.86
ARG APP_PORT=8080

FROM rust:${RUST_VERSION}-slim-bookworm AS build
WORKDIR /app

RUN apt update && \
    apt install make openssl --yes

    # Workspace-level files
RUN --mount=type=bind,source=rust/Cargo.toml,target=rust/Cargo.toml \
    --mount=type=bind,source=rust/Cargo.lock,target=rust/Cargo.lock \
    --mount=type=bind,source=rust/.cargo,target=rust/.cargo \

    #  The actual packages
    --mount=type=bind,source=rust/scamplers-schema,target=rust/scamplers-schema \
    --mount=type=bind,source=rust/scamplers-core,target=rust/scamplers-core \
    --mount=type=bind,source=rust/scamplers-backend,target=rust/scamplers-backend \

    # The database schema
    --mount=type=bind,source=db,target=db \

    # Caches
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=cache,target=/usr/local/cargo/bin \
    cargo install diesel_cli --no-default-features --features postgres-bundled && \
    cd rust && \
    IN_DOCKER=true cargo build --locked --release && \
    cp ./target/release/scamplers-backend /bin/scamplers-backend

FROM debian:bookworm-slim AS final

RUN mkdir app

RUN --mount=type=bind,source=rust/scamplers-schema,target=/app/scamplers-schema \
    --mount=type=bind,source=db,target=/app/db \
    --mount=type=bind,source=rust/check-schema.sh,target=/bin/check-schema.sh \

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

COPY --from=build /bin/scamplers-backend /bin/ \
    --from=build /usr/local/cargo/bin/diesel /bin/

EXPOSE ${APP_PORT}

CMD ["/bin/check-schema.sh", "&&", "/bin/scamplers-backend", "prod", "--log-dir", "logs", "--host", "0.0.0.0", "--port", "${APP_PORT}", "--public-address", ""]
