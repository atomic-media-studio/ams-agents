from fastapi import FastAPI, HTTPException

from src.api.rust_app_runner import rust_app_runner

app = FastAPI(title="arp-host-runner", version="0.1.0")


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok", "service": "arp-host-runner"}


@app.get("/rust/app/status")
async def rust_app_status() -> dict:
    return await rust_app_runner.status()


@app.post("/rust/app/compile")
async def rust_app_compile() -> dict:
    try:
        return await rust_app_runner.compile_workspace()
    except FileNotFoundError as exc:
        raise HTTPException(status_code=500, detail="cargo-not-found-on-host") from exc
    except Exception as exc:
        raise HTTPException(status_code=500, detail=f"compile-failed: {exc}") from exc


@app.post("/rust/app/start")
async def rust_app_start() -> dict:
    try:
        return await rust_app_runner.start_app()
    except FileNotFoundError as exc:
        raise HTTPException(status_code=500, detail="cargo-not-found-on-host") from exc
    except Exception as exc:
        raise HTTPException(status_code=500, detail=f"start-failed: {exc}") from exc


@app.post("/rust/app/stop")
async def rust_app_stop() -> dict:
    try:
        return await rust_app_runner.stop_app()
    except Exception as exc:
        raise HTTPException(status_code=500, detail=f"stop-failed: {exc}") from exc
