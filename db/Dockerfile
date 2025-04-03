ARG POSTGRES_VERSION=17.4
ARG ALPINE_VERSION=3.21

FROM postgres:${POSTGRES_VERSION}-alpine${ALPINE_VERSION}

COPY docker-entrypoint-initdb.d/* /docker-entrypoint-initdb.d/
