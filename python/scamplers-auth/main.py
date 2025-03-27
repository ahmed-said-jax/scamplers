#! /usr/bin/env python3
from pydantic_settings import (
    BaseSettings,
    PydanticBaseSettingsSource,
    SettingsConfigDict,
    TomlConfigSettingsSource,
)
from sanic import Sanic, redirect
from sanic.response import text
import sanic
import msal


class Cli(BaseSettings):
    model_config = SettingsConfigDict(
        cli_parse_args=True, env_prefix="SCAMPLERS_AUTH_", env_file=".env"
    )

    config_path: str = "/run/secrets/scamplers-auth_config"


cli = Cli()  # type: ignore


class AppConfig(sanic.Config):
    class AppConfigInner(BaseSettings):
        model_config = SettingsConfigDict(toml_file=cli.config_path)

        ms_auth_client_id: str
        ms_auth_client_credential: str
        ms_auth_redirect_uri: str

        @classmethod
        def settings_customise_sources(
            cls,
            settings_cls: type[BaseSettings],
            init_settings: PydanticBaseSettingsSource,
            env_settings: PydanticBaseSettingsSource,
            dotenv_settings: PydanticBaseSettingsSource,
            file_secret_settings: PydanticBaseSettingsSource,
        ) -> tuple[PydanticBaseSettingsSource, ...]:
            return (TomlConfigSettingsSource(settings_cls),)

    inner: AppConfigInner

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.inner = self.AppConfigInner()  # type: ignore


class AppContext:
    ms_auth_client: msal.ConfidentialClientApplication


config = AppConfig()  # type: ignore
app = Sanic("scamplers-auth", config=config, ctx=AppContext)

type App = Sanic[AppConfig, AppContext]

AUTH_FLOWS = {}


@app.before_server_start
async def attach_msal_auth_client(app: App, loop):
    app.ctx.ms_auth_client = msal.ConfidentialClientApplication(
        app.config.inner.ms_auth_client_id,
        authority="https://login.microsoftonline.com/common",
        client_credential=app.config.inner.ms_auth_client_credential,
    )


@app.route("/login")
async def login(request: sanic.Request):
    request.get_args()
    auth_client = app.ctx.ms_auth_client
    redirect_uri = app.config.inner.ms_auth_redirect_uri

    auth_flow = auth_client.initiate_auth_code_flow(
        scopes=[], redirect_uri=redirect_uri
    )

    AUTH_FLOWS[auth_flow["state"]] = auth_flow

    response = redirect(auth_flow["auth_uri"])

    return response


@app.route("/auth/ms")
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


@app.route("/")
async def hello(request: sanic.Request):
    return text("hi")


if __name__ == "__main__":
    app.run(host="localhost", port=8001)
