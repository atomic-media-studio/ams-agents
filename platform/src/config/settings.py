from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=None, extra="ignore")

    platform_host: str = "127.0.0.1"
    platform_port: int = 8080
    rocket_base_url: str = "http://127.0.0.1:8000"
    rocket_timeout_seconds: float = 10.0


settings = Settings()
