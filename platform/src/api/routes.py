from fastapi import APIRouter, HTTPException
import httpx

from src.bridge.runner import bridge_client
from src.api.rust_app_runner import rust_app_runner
from src.config.settings import settings

router = APIRouter()


def _runner_url(path: str) -> str | None:
    base = settings.rust_app_runner_base_url
    if not base:
        return None
    return f"{base.rstrip('/')}{path}"


async def _runner_get(path: str) -> dict:
    url = _runner_url(path)
    if url is None:
        if path == "/rust/app/status":
            return await rust_app_runner.status()
        raise RuntimeError(f"unsupported local GET path: {path}")

    timeout = settings.rust_app_runner_timeout_seconds
    try:
        async with httpx.AsyncClient(timeout=timeout) as client:
            res = await client.get(url)
            data = res.json()
            if not res.is_success:
                detail = data.get("detail") if isinstance(data, dict) else data
                raise HTTPException(status_code=502, detail=f"host-runner-error: {detail}")
            if not isinstance(data, dict):
                raise HTTPException(status_code=502, detail="host-runner-invalid-response")
            return data
    except httpx.HTTPError as exc:
        hint = (
            "Ensure host runner is running and bound to 0.0.0.0:8090 "
            "(not 127.0.0.1) so Docker can reach it via host.docker.internal"
        )
        raise HTTPException(
            status_code=502,
            detail=f"host-runner-unreachable: {exc}. {hint}",
        ) from exc


async def _runner_post(path: str) -> dict:
    url = _runner_url(path)
    if url is None:
        if path == "/rust/app/compile":
            return await rust_app_runner.compile_workspace()
        if path == "/rust/app/start":
            return await rust_app_runner.start_app()
        if path == "/rust/app/stop":
            return await rust_app_runner.stop_app()
        raise RuntimeError(f"unsupported local POST path: {path}")

    timeout = settings.rust_app_runner_timeout_seconds
    try:
        async with httpx.AsyncClient(timeout=timeout) as client:
            res = await client.post(url)
            data = res.json()
            if not res.is_success:
                detail = data.get("detail") if isinstance(data, dict) else data
                raise HTTPException(status_code=502, detail=f"host-runner-error: {detail}")
            if not isinstance(data, dict):
                raise HTTPException(status_code=502, detail="host-runner-invalid-response")
            return data
    except httpx.HTTPError as exc:
        hint = (
            "Ensure host runner is running and bound to 0.0.0.0:8090 "
            "(not 127.0.0.1) so Docker can reach it via host.docker.internal"
        )
        raise HTTPException(
            status_code=502,
            detail=f"host-runner-unreachable: {exc}. {hint}",
        ) from exc


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
    return await _runner_get("/rust/app/status")


@router.post("/rust/app/compile")
async def rust_app_compile() -> dict:
    try:
        return await _runner_post("/rust/app/compile")
    except HTTPException:
        raise
    except FileNotFoundError as exc:
        raise HTTPException(status_code=500, detail="cargo-not-found-on-server") from exc
    except Exception as exc:
        raise HTTPException(status_code=500, detail=f"compile-failed: {exc}") from exc


@router.post("/rust/app/start")
async def rust_app_start() -> dict:
    try:
        return await _runner_post("/rust/app/start")
    except HTTPException:
        raise
    except FileNotFoundError as exc:
        raise HTTPException(status_code=500, detail="cargo-not-found-on-server") from exc
    except Exception as exc:
        raise HTTPException(status_code=500, detail=f"start-failed: {exc}") from exc


@router.post("/rust/app/stop")
async def rust_app_stop() -> dict:
    try:
        return await _runner_post("/rust/app/stop")
    except HTTPException:
        raise
    except Exception as exc:
        raise HTTPException(status_code=500, detail=f"stop-failed: {exc}") from exc
