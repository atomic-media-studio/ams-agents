from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=None, env_prefix="ARP_", extra="ignore")

    platform_host: str = "127.0.0.1"
    platform_port: int = 8080
    rocket_base_url: str = "http://127.0.0.1:8000"
    rocket_timeout_seconds: float = 10.0
    rust_app_runner_base_url: str | None = None
    rust_app_runner_timeout_seconds: float = 30.0


settings = Settings()
