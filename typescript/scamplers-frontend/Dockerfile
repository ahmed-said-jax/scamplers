ARG BUN_VERSION=1.2
ARG AUTH_SUBDOMAIN
ARG PUBLIC_URL
ARG MS_AUTH_PATH

FROM oven/bun:${BUN_VERSION}-alpine AS base
WORKDIR /usr/src/app

# install dependencies into temp directory
# this will cache them and speed up future builds
FROM base AS install
RUN mkdir -p /temp/dev
COPY package.json bun.lock /temp/dev/
RUN cd /temp/dev && bun install --frozen-lockfile

# install with --production (exclude devDependencies)
RUN mkdir -p /temp/prod
COPY package.json bun.lock /temp/prod/
RUN cd /temp/prod && bun install --frozen-lockfile --production

# copy node_modules from temp directory
# then copy all (non-ignored) project files into the image
FROM base AS build
COPY --from=install /temp/dev/node_modules node_modules
COPY . .

ENV PUBLIC_AUTH_SUBDOMAIN=${AUTH_SUBDOMAIN} \
    PUBLIC_URL=${PUBLIC_URL} \
    PUBLIC_MS_AUTH_PATH=${MS_AUTH_PATH}

RUN bun --bun run build

RUN mv build /opt/scamplers-frontend
