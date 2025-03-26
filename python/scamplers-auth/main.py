from pydantic_settings import (
    BaseSettings,
    PydanticBaseSettingsSource,
    SettingsConfigDict,
    TomlConfigSettingsSource,
)
from sanic import Sanic
import asyncpg
from sanic.response import text
import sanic
import uvloop

class Cli(BaseSettings):
    model_config = SettingsConfigDict(cli_parse_args=True)

    config_path: str = "/run/secrets/scamplers-auth.toml"

cli = Cli() # type: ignore

class AppConfig(sanic.Config):
    class AppConfigInner(BaseSettings):
        model_config = SettingsConfigDict(toml_file=cli.config_path)

        client_id: str
        client_credential: str
        redirect_uri: str
        db_auth_user_url: str = "blah"

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

    inner_config: AppConfigInner

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.inner_config = self.AppConfigInner() # type: ignore


config = AppConfig() # type: ignore

app = Sanic("scamplers-auth", config=config)

@app.before_server_start
async def attach_db(app: Sanic, loop):
    ...

@app.route("/")
async def hello(request):
    return text("dsf")

if __name__ == "__main__":
    app.run()
