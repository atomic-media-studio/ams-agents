const $ = (id) => document.getElementById(id);

function setStatus(el, value, ok = true) {
  el.textContent = value;
  el.classList.remove("ok", "err");
  el.classList.add(ok ? "ok" : "err");
}

async function readJson(path, opts = undefined) {
  const res = await fetch(path, opts);
  const maybeJson = await res.json().catch(() => ({}));
  if (!res.ok) {
    throw new Error(maybeJson.detail || `HTTP ${res.status}`);
  }
  return maybeJson;
}

function setLog(text) {
  $("rustLog").textContent = text || "";
}

async function refreshBridgeCards() {
  $("platformTime").textContent = new Date().toLocaleTimeString();

  try {
    const platform = await readJson("/api/health");
    $("platformService").textContent = platform.service || "arp-platform";
    setStatus($("platformStatus"), platform.status || "ok", true);
  } catch (err) {
    $("platformService").textContent = "arp-platform";
    setStatus($("platformStatus"), String(err), false);
  }

  try {
    const rust = await readJson("/api/rust/health");
    $("rustService").textContent = rust.service || "ams-agents";
    setStatus($("rustStatus"), rust.status || "ok", true);
  } catch (err) {
    $("rustService").textContent = "ams-agents";
    setStatus($("rustStatus"), String(err), false);
  }

  try {
    const caps = await readJson("/api/rust/capabilities");
    $("rustVersion").textContent = caps.api_version || "-";
    $("rustEndpoints").textContent = Array.isArray(caps.endpoints)
      ? String(caps.endpoints.length)
      : "-";
  } catch (_err) {
    $("rustVersion").textContent = "-";
    $("rustEndpoints").textContent = "-";
  }
}

// --- Endpoint triggers ---
const EP_DEFS = [
  { id: "epHealth",    method: "GET",  path: "/api/health" },
  { id: "epRustHealth",method: "GET",  path: "/api/rust/health" },
  { id: "epCaps",      method: "GET",  path: "/api/rust/capabilities" },
  { id: "epPing",      method: "POST", path: "/api/rust/bridge/ping",
    body: { message: "hello from capabilities panel" } },
  { id: "epAppStatus", method: "GET",  path: "/api/rust/app/status" },
];

async function callEndpoint(def) {
  const el = $(def.id);
  el.textContent = "…";
  el.classList.add("visible");
  try {
    const opts = def.method === "POST"
      ? { method: "POST", headers: { "Content-Type": "application/json" },
          body: JSON.stringify(def.body || {}) }
      : undefined;
    const data = await readJson(def.path, opts);
    el.textContent = JSON.stringify(data, null, 2);
  } catch (err) {
    el.textContent = String(err);
  }
}

function wireCapToggle() {
  const btn = $("capToggle");
  const body = $("capBody");
  btn.addEventListener("click", () => {
    const expanded = btn.getAttribute("aria-expanded") === "true";
    btn.setAttribute("aria-expanded", String(!expanded));
    body.style.display = expanded ? "none" : "";
  });
}

function wireEndpointButtons() {
  document.querySelectorAll(".ep-btn").forEach((btn) => {
    const id = btn.dataset.epId;
    const def = EP_DEFS.find((d) => d.id === id);
    if (def) btn.addEventListener("click", () => callEndpoint(def));
  });
}

async function pingRust() {
  try {
    const payload = { message: "hello from dashboard" };
    const data = await readJson("/api/rust/bridge/ping", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
    $("pingEcho").textContent = data.echoed_message || "ok";
  } catch (err) {
    $("pingEcho").textContent = String(err);
  }
}

async function refreshRustApp() {
  try {
    const data = await readJson("/api/rust/app/status");
    setStatus($("rustProcStatus"), data.running ? "running" : "stopped", !!data.running);
    $("rustProcPid").textContent = data.pid ?? "-";
    $("rustTargetDir").textContent = data.target_dir ?? "-";
    setLog(data.log_tail || "");
  } catch (err) {
    setStatus($("rustProcStatus"), String(err), false);
  }
}

async function compileRustApp() {
  setStatus($("rustCompileStatus"), "compiling...", true);
  try {
    const data = await readJson("/api/rust/app/compile", { method: "POST" });
    setStatus($("rustCompileStatus"), data.ok ? "compile ok" : "compile failed", !!data.ok);
    const logText = [data.stdout || "", data.stderr || ""].filter(Boolean).join("\n");
    if (logText) {
      setLog(logText);
    }
  } catch (err) {
    setStatus($("rustCompileStatus"), String(err), false);
  }
  await refreshRustApp();
}

async function startRustApp() {
  try {
    await readJson("/api/rust/app/start", { method: "POST" });
  } catch (err) {
    setStatus($("rustProcStatus"), String(err), false);
  }
  await refreshRustApp();
}

async function stopRustApp() {
  try {
    await readJson("/api/rust/app/stop", { method: "POST" });
  } catch (err) {
    setStatus($("rustProcStatus"), String(err), false);
  }
  await refreshRustApp();
}

$("refreshBtn").addEventListener("click", refreshBridgeCards);
$("pingBtn").addEventListener("click", pingRust);
$("rustCompileBtn").addEventListener("click", compileRustApp);
$("rustStartBtn").addEventListener("click", startRustApp);
$("rustStopBtn").addEventListener("click", stopRustApp);

async function refreshAll() {
  await refreshBridgeCards();
  await refreshRustApp();
}

wireCapToggle();
wireEndpointButtons();
refreshAll();
setInterval(refreshRustApp, 2000);
