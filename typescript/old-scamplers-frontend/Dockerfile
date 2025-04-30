# syntax=docker/dockerfile:1

ARG NODE_VERSION=23.11.0

FROM node:${NODE_VERSION} as base
RUN npm install --global npm@latest


WORKDIR /usr/src/app

FROM base as deps

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    . "$HOME/.cargo/env" && \
    cargo install wasm-pack && \
    rustup target add wasm32-unknown-unknown

RUN --mount=type=bind,source=rust/Cargo.toml,target=rust/Cargo.toml \
    --mount=type=bind,source=rust/Cargo.lock,target=rust/Cargo.lock \
    --mount=type=bind,source=rust/.cargo,target=rust/.cargo \
    --mount=type=bind,source=rust/scamplers-schema,target=rust/scamplers-schema \
    --mount=type=bind,source=rust/uuid,target=rust/uuid \
    --mount=type=bind,source=rust/scamplers-core,target=rust/scamplers-core \
    --mount=type=bind,source=rust/scamplers-backend,target=rust/scamplers-backend \
    --mount=type=cache,target=/root/.npm \
    --mount=type=bind,source=typescript/scamplers-frontend/package.json,target=typescript/scamplers-frontend/package.json \
    --mount=type=bind,source=typescript/scamplers-frontend/package-lock.json,target=typescript/scamplers-frontend/package-lock.json \
    . "$HOME/.cargo/env" && \
    wasm-pack build rust/scamplers-core --release --out-dir ../../typescript/scamplers-core -- --features web && \
    cd typescript/scamplers-frontend && \
    npm ci --omit=dev


FROM deps as build

COPY --from=deps /usr/src/app/typescript/scamplers-core typescript/scamplers-core

# Download additional development dependencies before building, as some projects require
# "devDependencies" to be installed to build. If you don't need this, remove this step.
RUN --mount=type=bind,source=typescript/scamplers-frontend/package.json,target=typescript/scamplers-frontend/package.json \
    --mount=type=bind,source=typescript/scamplers-frontend/package-lock.json,target=typescript/scamplers-frontend/package-lock.json \
    --mount=type=cache,target=/root/.npm \
    cd typescript/scamplers-frontend && \
    npm ci

COPY typescript/scamplers-frontend typescript/scamplers-frontend

RUN cd typescript/scamplers-frontend && npm run build

FROM base as final

ENV NODE_ENV production

USER node

COPY typescript/scamplers-frontend/package.json .

COPY --from=deps /usr/src/app/typescript/scamplers-frontend/node_modules ./node_modules
COPY --from=build /usr/src/app/typescript/scamplers-frontend/build ./build

EXPOSE ${PORT}

CMD node build
