from identity.flask import Auth
import flask
from pydantic_settings import (
    BaseSettings,
    PydanticBaseSettingsSource,
    SettingsConfigDict,
    TomlConfigSettingsSource,
)

import msal_config


# This application is generally only run in the context of a Docker container, so hardcoding the path of the config file is fine. However, this can be easily changed by dynamically instantiating this class with a CLI-provided config path
class AppConfig(BaseSettings):
    model_config = SettingsConfigDict(toml_file="/run/secrets/auth-config.toml")

    client_id: str
    client_credential: str
    redirect_uri: str

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

config = AppConfig() # type: ignore
app = flask.Flask(__name__)
app.config.from_object(msal_config)

auth = Auth(
    app,
    client_id=config.client_id,
    client_credential=config.client_credential,
    redirect_uri=config.redirect_uri,
    authority="https://login.microsoftonline.com/common",
)

@app.route("/")
@auth.login_required
def hello(*, context):
    return f'you are: {context["user"]["name"]}'

@app.route("/redirect")
@auth.login_required
def hello2(*, context):
    return "hello"
