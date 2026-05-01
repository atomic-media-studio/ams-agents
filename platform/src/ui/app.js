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
$("rustCompileBtn").addEventListener("click", compileRustApp);
$("rustStartBtn").addEventListener("click", startRustApp);
$("rustStopBtn").addEventListener("click", stopRustApp);

async function refreshAll() {
  await refreshBridgeCards();
  await refreshRustApp();
}

refreshAll();
setInterval(refreshRustApp, 2000);
