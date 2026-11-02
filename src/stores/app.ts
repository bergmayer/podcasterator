import { writable, derived, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface AudioFile {
  id: string;
  original_path: string;
  temp_path: string;
  display_name: string;
  added_at: string;
}

export interface AppState {
  files: AudioFile[];
  podcast_name: string;
  artwork_path: string | null;
}

export const appState = writable<AppState>({
  files: [],
  podcast_name: "My Podcast",
  artwork_path: null,
});

export type ServerState =
  | { phase: "stopped" }
  | { phase: "starting" }
  | { phase: "running"; url: string }
  | { phase: "stopping"; url: string };
export const serverState = writable<ServerState>({ phase: "stopped" });
export const serverUrl = derived(serverState, (state) => "url" in state ? state.url : null);
export const isServerRunning = derived(serverState, (state) => state.phase === "running" || state.phase === "stopping");
export const isServerBusy = derived(serverState, (state) => state.phase === "starting" || state.phase === "stopping");
export const lastError = writable<string | null>(null);

let errorTimeout: ReturnType<typeof setTimeout> | null = null;

function showError(message: string) {
  if (errorTimeout) clearTimeout(errorTimeout);
  const msg = typeof message === "string" ? message : "An error occurred";
  lastError.set(msg);
  errorTimeout = setTimeout(() => {
    lastError.set(null);
    errorTimeout = null;
  }, 5000);
}

async function invokeAndSync(
  cmd: string,
  args?: Record<string, unknown>
): Promise<AppState | null> {
  try {
    const state = await invoke<AppState>(cmd, args);
    appState.set(state);
    return state;
  } catch (e) {
    console.error(`Command failed: ${cmd}`, e);
    showError(typeof e === "string" ? e : `Failed: ${cmd}`);
    return null;
  }
}

export async function loadState(): Promise<void> {
  try {
    const initial = get(serverState);
    const [state, url] = await Promise.all([
      invoke<AppState>("get_state"),
      invoke<string | null>("get_server_url"),
    ]);
    appState.set(state);
    if (get(serverState) === initial) {
      serverState.set(url ? { phase: "running", url } : { phase: "stopped" });
    }
  } catch (e) {
    console.error("Failed to load state:", e);
    showError("Failed to load application state");
  }
}

export async function addFiles(paths: string[]): Promise<void> {
  await invokeAndSync("add_files", { paths });
}

export async function deleteFile(id: string): Promise<void> {
  await invokeAndSync("delete_file", { id });
}

export async function renameFile(
  id: string,
  newName: string
): Promise<void> {
  await invokeAndSync("rename_file", { id, newName });
}

export async function moveFile(
  id: string,
  direction: "up" | "down"
): Promise<void> {
  await invokeAndSync("move_file", { id, direction });
}

export async function alphabetize(): Promise<void> {
  await invokeAndSync("alphabetize");
}

export async function reverse(): Promise<void> {
  await invokeAndSync("reverse");
}

export async function clearAll(): Promise<void> {
  await invokeAndSync("clear_all");
}

export async function setArtwork(path: string): Promise<void> {
  await invokeAndSync("set_artwork", { path });
}

export async function deleteArtwork(): Promise<void> {
  await invokeAndSync("delete_artwork");
}

export async function setPodcastName(name: string): Promise<void> {
  await invokeAndSync("set_podcast_name", { name });
}

export async function startServer(): Promise<void> {
  if (get(serverState).phase !== "stopped") return;
  serverState.set({ phase: "starting" });
  try {
    const url = await invoke<string>("start_server");
    serverState.set({ phase: "running", url });
  } catch (e) {
    serverState.set({ phase: "stopped" });
    console.error("Failed to start server:", e);
    showError(typeof e === "string" ? e : "Failed to start server");
  }
}

export async function stopServer(): Promise<void> {
  const current = get(serverState);
  if (current.phase !== "running") return;
  serverState.set({ phase: "stopping", url: current.url });
  try {
    await invoke("stop_server");
    serverState.set({ phase: "stopped" });
  } catch (e) {
    serverState.set(current);
    console.error("Failed to stop server:", e);
    showError(typeof e === "string" ? e : "Failed to stop server");
  }
}
