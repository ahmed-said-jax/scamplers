from typing import Any
import msal
import tomllib as toml
from pathlib import Path


def main(config_path: str):
    config = Path(config_path)

    config_dict = toml.loads(config.read_text())
    auth_config: dict[str, Any] = config_dict["ms_auth"]

    ms_auth = msal.ConfidentialClientApplication(client_id=auth_config["client_id"], client_credential=auth_config["client_credential"], )
    
    auth_config["client_id"]
