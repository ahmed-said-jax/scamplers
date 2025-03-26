import json
from typing import Any
from identity.quart import Auth
import flask
from pydantic_settings import (
    BaseSettings,
    PydanticBaseSettingsSource,
    SettingsConfigDict,
    TomlConfigSettingsSource,
)



# This application is generally only run in the context of a Docker container, so hardcoding the path of the config file is fine. However, this can be easily changed by dynamically instantiating this class with a CLI-provided config path
class AppConfig(BaseSettings):
    model_config = SettingsConfigDict(toml_file="/Users/saida/.config/auth-config.toml")

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
app = quart.Quart(__name__)

auth = Auth(
    app,
    client_id=config.client_id,
    client_credential=config.client_credential,
    redirect_uri=config.redirect_uri,
    authority="https://login.microsoftonline.com/common",
)

@app.route("/login")
@auth.login_required
def hello(*, context: dict[str, Any]):
    print(json.dumps(context, indent=2))
    return json.dumps(context, indent=2)


if __name__ == "__main__":
    print(flask_session.__version__)
    print("hello")
