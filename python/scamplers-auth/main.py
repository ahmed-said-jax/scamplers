#! /usr/bin/env python3
from dataclasses import dataclass
import datetime
import json
from pydantic_settings import (
    BaseSettings,
    CliSettingsSource,
    DotEnvSettingsSource,
    PydanticBaseSettingsSource,
    SettingsConfigDict,
    TomlConfigSettingsSource,
    SecretsSettingsSource,

)
from sanic import Sanic, redirect
from sanic.response import text
import sanic
import msal
import asyncpg
from datetime import datetime, timedelta

@dataclass
class ConfigContainer(sanic.Config):
    class Config(BaseSettings):
        model_config = SettingsConfigDict(cli_parse_args=True, secrets_dir="/run/secrets", env_prefix="SCAMPLERS_AUTH_", env_file=".env")

        db_host: str
        db_port: int
        db_auth_user_password: str
        auth_host: str
        auth_port: int
        ms_auth_path: str
        ms_auth_client_id: str
        ms_auth_client_credential: str
        ms_auth_redirect_uri: str
        ms_auth_flow_expiration: datetime

        @classmethod
        def settings_customise_sources(
            cls,
            settings_cls: type[BaseSettings],
            init_settings: PydanticBaseSettingsSource,
            env_settings: PydanticBaseSettingsSource,
            dotenv_settings: PydanticBaseSettingsSource,
            file_secret_settings: PydanticBaseSettingsSource,
        ) -> tuple[PydanticBaseSettingsSource, ...]:
            return init_settings, CliSettingsSource(settings_cls), env_settings, dotenv_settings, file_secret_settings

    app_config: Config

class Context:
    ms_auth_client: msal.ConfidentialClientApplication
    db_pool: asyncpg.Pool

config = ConfigContainer()  # type: ignore

type App = Sanic[ConfigContainer, type[Context]]
app: App = Sanic("scamplers-auth", config=config, ctx=Context)

@app.before_server_start
async def attach_msal_auth_client(app: App, loop):
    app.ctx.ms_auth_client = msal.ConfidentialClientApplication(
        client_id=app.config.app_config.ms_auth_client_id,
        authority="https://login.microsoftonline.com/common",
        client_credential=app.config.app_config.ms_auth_client_credential,
    )

    app_config = app.config.app_config

    app.ctx.db_pool = await asyncpg.create_pool(user="auth_user", password=app_config.db_auth_user_password, host=app_config.db_host, port=app_config.db_port)


@app.route(app.config.app_config.ms_auth_path)
async def ms_login(request: sanic.Request):
    auth_client = app.ctx.ms_auth_client
    redirect_uri = app.config.inner.ms_auth_redirect_uri

    auth_flow = auth_client.initiate_auth_code_flow(
        scopes=[], redirect_uri=redirect_uri
    )

    redirected_from = request.args.get("redirected_from", "/")

    db_pool = app.ctx.db_pool
    await db_pool.execute("insert into ms_auth_flow (state, flow, redirected_from, expires_at) values ($1, $2::jsonb, $3, $4)", auth_flow["state"], json.dumps(auth_flow), redirected_from, datetime.utcnow() + timedelta(minutes=10))

    response = redirect(auth_flow["auth_uri"])

    return response


@app.route("/auth/ms-redirect")
async def ms_auth(request: sanic.Request):
    received_auth_flow = request.args
    stored_auth_flow = AUTH_FLOWS.pop(received_auth_flow["state"][0])
    print(received_auth_flow)

    auth_client = app.ctx.ms_auth_client
    result = auth_client.acquire_token_by_auth_code_flow(
        stored_auth_flow, received_auth_flow
    )
    print(result)

    return text("logged tf in")

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=8001)
