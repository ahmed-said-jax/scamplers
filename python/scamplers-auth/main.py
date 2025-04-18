#! /usr/bin/env python3
from dataclasses import dataclass
import json
import os
from typing import Any
from uuid import UUID, uuid4
from pydantic import ValidationError
from pydantic_settings import (
    BaseSettings,
    CliSettingsSource,
    PydanticBaseSettingsSource,
    SecretsSettingsSource,
    SettingsConfigDict,
)
from sanic import Sanic, redirect
from sanic.response import text
import sanic
import msal
import asyncpg
from datetime import datetime, timedelta
import httpx

config_source_config = SettingsConfigDict(
    cli_parse_args=True, secrets_dir="/run/secrets", env_file=".env", env_prefix="SCAMPLERS_", extra="ignore"
)

# This is a workaround for https://github.com/pydantic/pydantic-settings/issues/30
in_docker = bool(os.environ.get("IN_DOCKER"))

if in_docker:
    config_source_config["env_prefix"] = ""

class ConfigContainer(sanic.Config):
    class Config(BaseSettings):
        model_config = config_source_config

        debug: bool = False
        db_host: str
        db_port: int
        db_auth_user_password: str
        db_name: str
        host: str
        port: int
        backend_host: str
        backend_port: str
        auth_subdomain: str
        public_url: str
        ms_auth_path: str
        ms_auth_client_id: str
        ms_auth_client_credential: str
        ms_auth_redirect_path: str

        @classmethod
        def settings_customise_sources(
            cls,
            settings_cls: type[BaseSettings],
            init_settings: PydanticBaseSettingsSource,
            env_settings: PydanticBaseSettingsSource,
            dotenv_settings: PydanticBaseSettingsSource,
            file_secret_settings: PydanticBaseSettingsSource,
        ) -> tuple[PydanticBaseSettingsSource, ...]:
            return (
                init_settings,
                env_settings,
                dotenv_settings,
                file_secret_settings,
            )

    inner: Config = Config() # type: ignore

class Context:
    ms_auth_client: msal.ConfidentialClientApplication
    db_pool: asyncpg.Pool
    http_client: httpx.Client

config = ConfigContainer()  # type: ignore
inner_config = config.inner

type App = Sanic[ConfigContainer, type[Context]]
app: App = Sanic("scamplers-auth", config=config, ctx=Context)


@app.before_server_start
async def attach_msal_auth_client(app: App, loop):
    app.ctx.ms_auth_client = msal.ConfidentialClientApplication(
        client_id=app.config.inner.ms_auth_client_id,
        authority="https://login.microsoftonline.com/common",
        client_credential=app.config.inner.ms_auth_client_credential,
    )


@app.before_server_start
async def attach_db_pool(app: App, loop):
    app_config = app.config.inner

    app.ctx.db_pool = await asyncpg.create_pool(
        user="auth_user",
        password=app_config.db_auth_user_password,
        host=app_config.db_host,
        port=app_config.db_port,
        database=app_config.db_name,
        loop=loop
    )

@app.before_server_start
async def attach_http_client(app: App, loop):
    auth = httpx.BasicAuth(username="auth_user", password=app.config.inner.db_auth_user_password)

    app.ctx.http_client = httpx.Client(http2=True, auth=auth)

@app.after_server_stop
async def close_db_connection(app: App, loop):
    await app.ctx.db_pool.close()

@app.after_server_stop
async def close_http_client(app: App, loop):
    app.ctx.http_client.close()

type Request = sanic.Request[App]

@app.route("/health")
async def health(request: Request) -> sanic.HTTPResponse:
    return text("")

@app.route(app.config.inner.ms_auth_path)
async def initiate_ms_login(request: Request) -> sanic.HTTPResponse:
    auth_client = request.app.ctx.ms_auth_client

    app_config = request.app.config.inner
    redirect_uri = f"https://{app_config.auth_subdomain}.{app_config.public_url}{app_config.ms_auth_redirect_path}"

    auth_flow = auth_client.initiate_auth_code_flow(
        scopes=["email"], redirect_uri=redirect_uri
    )

    redirected_from = f"{request.args.get("redirected_from", "/")}"

    db_pool = request.app.ctx.db_pool
    await db_pool.execute(
        "insert into ms_auth_flow (state, flow, redirected_from, expires_at) values ($1, $2, $3, $4)",
        auth_flow["state"],
        json.dumps(auth_flow),
        redirected_from,
        datetime.now() + timedelta(minutes=10),
    )

    return redirect(auth_flow["auth_uri"])


@app.route(app.config.inner.ms_auth_redirect_path)
async def complete_ms_login(request: Request) -> sanic.HTTPResponse:
    received_auth_flow = request.args

    db_pool = request.app.ctx.db_pool
    http_client = request.app.ctx.http_client
    auth_client = request.app.ctx.ms_auth_client

    app_config = request.app.config.inner

    async with db_pool.acquire() as conn:
        async with conn.transaction():
            auth_flow = await conn.fetchrow("select flow, redirected_from from ms_auth_flow where state = $1", received_auth_flow["state"][0])
            await conn.execute("delete from ms_auth_flow where state = $1", received_auth_flow["state"][0])

            stored_auth_flow = json.loads(auth_flow["flow"])

            user = auth_client.acquire_token_by_auth_code_flow(
                stored_auth_flow, received_auth_flow
            )["id_token_claims"]

            institution_id: str = await conn.fetchval("select id::text from institution where ms_tenant_id = $1", UUID(user["tid"]))

    user["institution_id"] = institution_id
    user["ms_user_id"] = user["oid"]

    result = http_client.post(f"http://{app_config.backend_host}{app_config.backend_port}/api/session", json=user)
    data = result.json()

    response = redirect(f"https://{app_config.public_url}" + auth_flow["redirected_from"])

    if session_id := data.get("session_id"):
        response.add_cookie("SESSION", session_id, httponly=True)

    return response

if __name__ == "__main__":
    config = app.config.inner

    app.run(host=config.host, port=config.port, debug=config.debug)
