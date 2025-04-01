#! /usr/bin/env python3
from dataclasses import dataclass
import json
from uuid import UUID
from pydantic_settings import (
    BaseSettings,
    CliSettingsSource,
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
        model_config = SettingsConfigDict(
            cli_parse_args=True, secrets_dir="/run/secrets", env_file=".env"
        )

        db_host: str
        db_port: int
        db_auth_user_password: str
        auth_host: str
        auth_port: int
        ms_auth_path: str
        ms_auth_client_id: str
        ms_auth_client_credential: str
        ms_auth_redirect_url: str
        ms_auth_flow_expiration: datetime
        session_id_salt_string: str

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
                CliSettingsSource(settings_cls),
                env_settings,
                dotenv_settings,
                file_secret_settings,
            )

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


@app.before_server_start
async def attach_db_pool(app: App, loop):
    app_config = app.config.app_config

    app.ctx.db_pool = await asyncpg.create_pool(
        user="auth_user",
        password=app_config.db_auth_user_password,
        host=app_config.db_host,
        port=app_config.db_port,
    )


@app.route(app.config.app_config.ms_auth_path)
async def ms_login(request: sanic.Request):
    auth_client = app.ctx.ms_auth_client
    redirect_uri = app.config.inner.ms_auth_redirect_uri

    auth_flow = auth_client.initiate_auth_code_flow(
        scopes=[], redirect_uri=redirect_uri
    )

    redirected_from = request.args.get("redirected_from", "/")

    db_pool = request.app.ctx.db_pool
    await db_pool.execute(
        "insert into ms_auth_flow (state, flow, redirected_from, expires_at) values ($1, $2, $3, $4)",
        auth_flow["state"],
        json.dumps(auth_flow),
        redirected_from,
        datetime.utcnow() + timedelta(minutes=10),
    )

    response = redirect(auth_flow["auth_uri"])

    return response


@app.route("/auth/microsoft-redirect")
async def ms_auth(request: sanic.Request[App]):
    received_auth_flow = request.args

    db_pool = request.app.ctx.db_pool
    auth_flow = await db_pool.fetchrow("select flow, redirected_from where state = $1", received_auth_flow["state"])

    stored_auth_flow = auth_flow["flow"]

    auth_client = app.ctx.ms_auth_client
    user = auth_client.acquire_token_by_auth_code_flow(
        stored_auth_flow, received_auth_flow
    )

    institution_id: UUID = await db_pool.fetchval("select id from institution where ms_tenant_id = $1", UUID(user["tid"]))

    base_insert_query = "insert into person (name, email, ms_user_id, institution_id) values ($1, $2, $3, $4) on conflict"
    params = (user["name"], user["email"], UUID(user["oid"]), institution_id)
    result = await db_pool.execute(f"{base_insert_query} (email) do update set name = $1, ms_user_id = $3, institution_id = $4", *params)
    result = await db_pool.execute(f"{base_insert_query} (ms_user_id) do update set name = $1, email = $2, institution_id = $4", *params)

    # TODO: save a session to the database and store session key in cookie

    return redirect(auth_flow["redirected_from"])


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=8001)
