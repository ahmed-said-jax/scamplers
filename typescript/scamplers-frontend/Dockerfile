# syntax=docker/dockerfile:1

ARG NODE_VERSION=24

FROM node:${NODE_VERSION} as base
RUN npm install --global npm@latest

WORKDIR /usr/src/app

FROM base as deps

RUN --mount=type=cache,target=/root/.npm \
    --mount=type=bind,source=typescript/scamplers-frontend/package.json,target=typescript/scamplers-frontend/package.json \
    --mount=type=bind,source=typescript/scamplers-frontend/package-lock.json,target=typescript/scamplers-frontend/package-lock.json \
    cd typescript/scamplers-frontend && \
    npm ci --omit=dev

FROM deps as build

# Download additional development dependencies before building, as some projects require
# "devDependencies" to be installed to build. If you don't need this, remove this step.
RUN --mount=type=bind,source=typescript/scamplers-frontend/package.json,target=typescript/scamplers-frontend/package.json \
    --mount=type=bind,source=typescript/scamplers-frontend/package-lock.json,target=typescript/scamplers-frontend/package-lock.json \
    --mount=type=cache,target=/root/.npm \
    cd typescript/scamplers-frontend && \
    npm ci

COPY typescript/scamplers-frontend typescript/scamplers-frontend
COPY typescript/scamplers-core typescript/scamplers-core

RUN cd typescript/scamplers-frontend && npm run build

FROM base as final

ENV NODE_ENV production

USER node

COPY typescript/scamplers-frontend/package.json .

COPY --from=deps /usr/src/app/typescript/scamplers-frontend/node_modules ./node_modules
COPY --from=build /usr/src/app/typescript/scamplers-frontend/build ./build

EXPOSE ${PORT}

CMD node build
