# syntax=docker/dockerfile:1

# Comments are provided throughout this file to help you get started.
# If you need more help, visit the Dockerfile reference guide at
# https://docs.docker.com/go/dockerfile-reference/

# Want to help us make this template better? Share your feedback here: https://forms.gle/ybq9Krt8jtBL3iCk7

ARG NODE_VERSION=23.11

ARG AUTH_HOST
ARG AUTH_PORT
ARG MS_AUTH_PATH

################################################################################
# Use node image for base image for all stages.
FROM node:${NODE_VERSION}-alpine as base

ARG AUTH_HOST
ARG AUTH_PORT
ARG MS_AUTH_PATH

# Set working directory for all build stages.
WORKDIR /usr/src/app

RUN echo "PUBLIC_AUTH_HOST=${AUTH_HOST}" >> .env && \
    echo "PUBLIC_AUTH_PORT=${AUTH_PORT}" >> .env && \
    echo "PUBLIC_MS_AUTH_PATH=${MS_AUTH_PATH}" >> .env

################################################################################
# Create a stage for installing production dependecies.
FROM base as deps

# Download dependencies as a separate step to take advantage of Docker's caching.
# Leverage a cache mount to /root/.npm to speed up subsequent builds.
# Leverage bind mounts to package.json and package-lock.json to avoid having to copy them
# into this layer.
RUN --mount=type=bind,source=package.json,target=package.json \
    --mount=type=bind,source=package-lock.json,target=package-lock.json \
    --mount=type=cache,target=/root/.npm \
    npm ci --omit=dev

################################################################################
# Create a stage for building the application.
FROM deps as build

# Download additional development dependencies before building, as some projects require
# "devDependencies" to be installed to build. If you don't need this, remove this step.
RUN --mount=type=bind,source=package.json,target=package.json \
    --mount=type=bind,source=package-lock.json,target=package-lock.json \
    --mount=type=cache,target=/root/.npm \
    npm ci

COPY --from=base /usr/src/app/.env .env

# Copy the rest of the source files into the image.
COPY . .

# Run the build script.
RUN npm run build

RUN mv build /opt/scamplers-frontend
