from __future__ import annotations

import asyncio
import os
import signal
from pathlib import Path
from typing import Any


class RustAppRunner:
    def __init__(self) -> None:
        # platform/src/api/rust_app_runner.py -> repo root is three parents up.
        self.repo_root = Path(__file__).resolve().parents[3]
        self.app_dir = self.repo_root / "apps" / "arpsci"
        self.app_target_dir = self.app_dir / "target"
        self.app_binary = self.app_target_dir / "debug" / "arpsci"
        self._lock = asyncio.Lock()
        self._process: asyncio.subprocess.Process | None = None
        self._reader_task: asyncio.Task[None] | None = None
        self._log_tail = ""

    def _append_log(self, text: str) -> None:
        self._log_tail += text
        # Keep only recent output to avoid unbounded memory growth.
        if len(self._log_tail) > 20000:
            self._log_tail = self._log_tail[-20000:]

    async def _capture_output(self, process: asyncio.subprocess.Process) -> None:
        if process.stdout is None:
            return
        while True:
            chunk = await process.stdout.readline()
            if not chunk:
                break
            self._append_log(chunk.decode("utf-8", errors="replace"))
        await process.wait()

    def _status_payload(self) -> dict[str, Any]:
        process = self._process
        running = process is not None and process.returncode is None
        return {
            "running": running,
            "pid": process.pid if process else None,
            "returncode": process.returncode if process else None,
            "repo_root": str(self.repo_root),
            "app_dir": str(self.app_dir),
            "target_dir": str(self.app_target_dir),
            "log_tail": self._log_tail[-5000:],
        }

    async def compile_workspace(self) -> dict[str, Any]:
        proc = await asyncio.create_subprocess_exec(
            "cargo",
            "build",
            "-p",
            "arpsci",
            "--target-dir",
            "target",
            cwd=str(self.app_dir),
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )
        stdout, stderr = await proc.communicate()
        out = stdout.decode("utf-8", errors="replace")
        err = stderr.decode("utf-8", errors="replace")

        # Persist compile output in the same log tail panel for easy inspection.
        self._append_log("\n$ (cd apps/arpsci && cargo build -p arpsci --target-dir target)\n")
        if out:
            self._append_log(out)
        if err:
            self._append_log(err)

        return {
            "ok": proc.returncode == 0,
            "returncode": proc.returncode,
            "stdout": out[-5000:],
            "stderr": err[-5000:],
        }

    async def start_app(self) -> dict[str, Any]:
        async with self._lock:
            if self._process is not None and self._process.returncode is None:
                payload = self._status_payload()
                payload["message"] = "already running"
                return payload

            if not self.app_binary.exists():
                compile_result = await self.compile_workspace()
                if not compile_result.get("ok", False):
                    payload = self._status_payload()
                    payload["message"] = "compile failed; app not started"
                    payload["compile"] = compile_result
                    return payload

            env = os.environ.copy()
            env.setdefault("ARPSCI_WEB_ENABLED", "true")
            # When platform runs in Docker and Rust runs on host, Rocket must bind
            # beyond loopback so the container can reach it via host.docker.internal.
            env.setdefault("ROCKET_ADDRESS", "0.0.0.0")

            self._append_log("\n$ ARPSCI_WEB_ENABLED=true apps/arpsci/target/debug/arpsci\n")
            process = await asyncio.create_subprocess_exec(
                str(self.app_binary),
                cwd=str(self.repo_root),
                env=env,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.STDOUT,
                start_new_session=True,
            )

            self._process = process
            self._reader_task = asyncio.create_task(self._capture_output(process))

            await asyncio.sleep(0.25)
            payload = self._status_payload()
            payload["message"] = "started"
            return payload

    async def stop_app(self) -> dict[str, Any]:
        async with self._lock:
            process = self._process
            if process is None or process.returncode is not None:
                payload = self._status_payload()
                payload["message"] = "already stopped"
                return payload

            try:
                os.killpg(process.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                await asyncio.wait_for(process.wait(), timeout=8.0)
            except asyncio.TimeoutError:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                await process.wait()

            if self._reader_task is not None:
                try:
                    await asyncio.wait_for(self._reader_task, timeout=1.0)
                except asyncio.TimeoutError:
                    self._reader_task.cancel()

            payload = self._status_payload()
            payload["message"] = "stopped"
            return payload

    async def status(self) -> dict[str, Any]:
        return self._status_payload()


rust_app_runner = RustAppRunner()
