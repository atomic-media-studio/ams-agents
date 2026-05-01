from typing import Any

import httpx

from src.config.settings import settings


class RustBridgeClient:
    def __init__(self, base_url: str, timeout_seconds: float) -> None:
        self.base_url = base_url.rstrip("/")
        self.timeout_seconds = timeout_seconds

    async def health(self) -> dict[str, Any]:
        return await self._get_json("/api/health")

    async def capabilities(self) -> dict[str, Any]:
        return await self._get_json("/api/capabilities")

    async def bridge_ping(self, message: str) -> dict[str, Any]:
        return await self._post_json("/api/bridge/ping", {"message": message})

    async def _get_json(self, path: str) -> dict[str, Any]:
        async with httpx.AsyncClient(timeout=self.timeout_seconds) as client:
            response = await client.get(f"{self.base_url}{path}")
            response.raise_for_status()
            return response.json()

    async def _post_json(self, path: str, payload: dict[str, Any]) -> dict[str, Any]:
        async with httpx.AsyncClient(timeout=self.timeout_seconds) as client:
            response = await client.post(f"{self.base_url}{path}", json=payload)
            response.raise_for_status()
            return response.json()


bridge_client = RustBridgeClient(
    base_url=settings.rocket_base_url,
    timeout_seconds=settings.rocket_timeout_seconds,
)
