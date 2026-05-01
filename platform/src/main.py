from pathlib import Path

from fastapi import FastAPI
from fastapi.responses import FileResponse
from fastapi.staticfiles import StaticFiles

from src.api.routes import router as api_router

app = FastAPI(title="arp-platform", version="0.1.0")
app.include_router(api_router, prefix="/api")

BASE_DIR = Path(__file__).resolve().parent
UI_DIR = BASE_DIR / "ui"
INDEX_FILE = BASE_DIR / "ui" / "index.html"

app.mount("/ui", StaticFiles(directory=UI_DIR), name="ui")


@app.get("/")
async def dashboard() -> FileResponse:
	return FileResponse(INDEX_FILE)
