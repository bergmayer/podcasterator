import { writable } from "svelte/store";
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

export const serverUrl = writable<string | null>(null);
export const isServerRunning = writable(false);
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
    const state = await invoke<AppState>("get_state");
    appState.set(state);

    const url = await invoke<string | null>("get_server_url");
    if (url) {
      serverUrl.set(url);
      isServerRunning.set(true);
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
  try {
    const url = await invoke<string>("start_server");
    serverUrl.set(url);
    isServerRunning.set(true);
  } catch (e) {
    console.error("Failed to start server:", e);
    showError(typeof e === "string" ? e : "Failed to start server");
  }
}

export async function stopServer(): Promise<void> {
  try {
    await invoke("stop_server");
    serverUrl.set(null);
    isServerRunning.set(false);
  } catch (e) {
    console.error("Failed to stop server:", e);
    showError(typeof e === "string" ? e : "Failed to stop server");
  }
}
