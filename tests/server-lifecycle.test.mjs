import { readFileSync } from "node:fs";
import vm from "node:vm";
import assert from "node:assert/strict";
import test from "node:test";
import ts from "typescript";
import * as stores from "svelte/store";

function fixture() {
  const calls = [];
  const source = readFileSync(new URL("../src/stores/app.ts", import.meta.url), "utf8");
  const exports = {};
  const timers = new Set();
  vm.runInNewContext(ts.transpileModule(source, {compilerOptions: {module: ts.ModuleKind.CommonJS}}).outputText, {
    exports, console: {error() {}},
    setTimeout: callback => {timers.add(callback); return callback;},
    clearTimeout: callback => timers.delete(callback),
    require: name => name === "svelte/store" ? stores : {invoke: (command) => new Promise((resolve, reject) => calls.push({command, resolve, reject}))},
  });
  return {api: exports, calls, state: () => stores.get(exports.serverState)};
}

test("start and stop reject overlapping commands and keep the URL through shutdown", async () => {
  const f = fixture();
  const starting = f.api.startServer();
  await f.api.startServer(); await f.api.stopServer();
  assert.equal(f.calls.length, 1);
  assert.equal(f.state().phase, "starting");
  f.calls[0].resolve("http://fixture/feed"); await starting;
  assert.equal(stores.get(f.api.isServerRunning), true);
  const stopping = f.api.stopServer();
  await f.api.stopServer(); await f.api.startServer();
  assert.equal(f.calls.length, 2);
  assert.equal(f.state().phase, "stopping");
  assert.equal(stores.get(f.api.serverUrl), "http://fixture/feed");
  f.calls[1].resolve(); await stopping;
  assert.equal(f.state().phase, "stopped");
  assert.equal(stores.get(f.api.serverUrl), null);
});

test("failed transitions restore a usable state", async () => {
  const f = fixture();
  const failedStart = f.api.startServer(); f.calls[0].reject("unavailable"); await failedStart;
  assert.equal(f.state().phase, "stopped");
  const started = f.api.startServer(); f.calls[1].resolve("fixture"); await started;
  const failedStop = f.api.stopServer(); f.calls[2].reject("unavailable"); await failedStop;
  assert.equal(f.state().phase, "running");
  assert.equal(stores.get(f.api.serverUrl), "fixture");
});

test("startup status cannot overwrite a newer server command", async () => {
  const f = fixture();
  const loading = f.api.loadState();
  assert.equal(f.calls.length, 2, "Startup reads begin together");
  f.calls[0].resolve({files: []});
  assert.equal(f.calls[1].command, "get_server_url");
  const starting = f.api.startServer();
  f.calls[2].resolve("new-server"); await starting;
  f.calls[1].resolve(null); await loading;
  assert.equal(stores.get(f.api.serverUrl), "new-server");
});
