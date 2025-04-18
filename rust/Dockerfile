ARG RUST_VERSION=1.86

FROM rust:${RUST_VERSION}-bookworm AS build
WORKDIR /app

RUN apt update && \
    apt install make openssl --yes && \
    cargo install diesel_cli --no-default-features --features postgres-bundled

RUN --mount=type=bind,source=rust/Cargo.toml,target=rust/Cargo.toml \
    --mount=type=bind,source=rust/Cargo.lock,target=rust/Cargo.lock \
    --mount=type=bind,source=rust/.cargo,target=rust/.cargo \
    --mount=type=bind,source=rust/scamplers-schema,target=rust/scamplers-schema \
    --mount=type=bind,source=rust/uuid,target=rust/uuid \
    --mount=type=bind,source=rust/scamplers-core,target=rust/scamplers-core \
    --mount=type=bind,source=rust/scamplers-backend,target=rust/scamplers-backend \
    --mount=type=bind,source=db,target=db \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cd rust && \
    IN_DOCKER=true cargo build --locked --release && \
    cp ./target/release/scamplers-backend /bin/scamplers-backend

FROM debian:bookworm AS final

RUN apt update && apt install curl --yes

RUN mkdir app

COPY --from=build /bin/scamplers-backend /bin/
COPY --from=build /usr/local/cargo/bin/diesel /bin/
COPY rust/scamplers-schema /app/scamplers-schema
COPY db /app/db
COPY rust/check-schema.sh /bin/check-schema.sh

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

EXPOSE ${PORT}

CMD /bin/check-schema.sh && /bin/scamplers-backend prod --log-dir logs --host 0.0.0.0 --port $PORT --secrets-dir /run/secrets --db-host $DB_HOST --db-port $DB_PORT
