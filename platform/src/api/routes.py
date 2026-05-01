from fastapi import APIRouter, HTTPException
import httpx

from src.bridge.runner import bridge_client
from src.api.rust_app_runner import rust_app_runner

router = APIRouter()


@router.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok", "service": "arp-platform"}


@router.get("/rust/health")
async def rust_health() -> dict:
    try:
        return await bridge_client.health()
    except httpx.HTTPError as exc:
        raise HTTPException(status_code=502, detail=f"rust-health-error: {exc}") from exc


@router.get("/rust/capabilities")
async def rust_capabilities() -> dict:
    try:
        return await bridge_client.capabilities()
    except httpx.HTTPError as exc:
        raise HTTPException(status_code=502, detail=f"rust-capabilities-error: {exc}") from exc


@router.post("/rust/bridge/ping")
async def rust_bridge_ping(payload: dict) -> dict:
    message = str(payload.get("message", "ping"))
    try:
        return await bridge_client.bridge_ping(message)
    except httpx.HTTPError as exc:
        raise HTTPException(status_code=502, detail=f"rust-bridge-ping-error: {exc}") from exc


@router.get("/rust/app/status")
async def rust_app_status() -> dict:
    return await rust_app_runner.status()


@router.post("/rust/app/compile")
async def rust_app_compile() -> dict:
    try:
        return await rust_app_runner.compile_workspace()
    except FileNotFoundError as exc:
        raise HTTPException(status_code=500, detail="cargo-not-found-on-server") from exc
    except Exception as exc:
        raise HTTPException(status_code=500, detail=f"compile-failed: {exc}") from exc


@router.post("/rust/app/start")
async def rust_app_start() -> dict:
    try:
        return await rust_app_runner.start_app()
    except FileNotFoundError as exc:
        raise HTTPException(status_code=500, detail="cargo-not-found-on-server") from exc
    except Exception as exc:
        raise HTTPException(status_code=500, detail=f"start-failed: {exc}") from exc


@router.post("/rust/app/stop")
async def rust_app_stop() -> dict:
    try:
        return await rust_app_runner.stop_app()
    except Exception as exc:
        raise HTTPException(status_code=500, detail=f"stop-failed: {exc}") from exc
