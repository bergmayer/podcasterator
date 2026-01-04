import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface AudioFile {
  id: string;
  original_path: string;
  temp_path: string;
  display_name: string;
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
  }
}

export async function addFiles(paths: string[]): Promise<void> {
  try {
    const state = await invoke<AppState>("add_files", { paths });
    appState.set(state);
  } catch (e) {
    console.error("Failed to add files:", e);
  }
}

export async function deleteFile(index: number): Promise<void> {
  try {
    const state = await invoke<AppState>("delete_file", { index });
    appState.set(state);
  } catch (e) {
    console.error("Failed to delete file:", e);
  }
}

export async function renameFile(
  index: number,
  newName: string
): Promise<void> {
  try {
    const state = await invoke<AppState>("rename_file", {
      index,
      newName,
    });
    appState.set(state);
  } catch (e) {
    console.error("Failed to rename file:", e);
  }
}

export async function moveFile(
  index: number,
  direction: "up" | "down"
): Promise<void> {
  try {
    const state = await invoke<AppState>("move_file", { index, direction });
    appState.set(state);
  } catch (e) {
    console.error("Failed to move file:", e);
  }
}

export async function alphabetize(): Promise<void> {
  try {
    const state = await invoke<AppState>("alphabetize");
    appState.set(state);
  } catch (e) {
    console.error("Failed to alphabetize:", e);
  }
}

export async function reverse(): Promise<void> {
  try {
    const state = await invoke<AppState>("reverse");
    appState.set(state);
  } catch (e) {
    console.error("Failed to reverse:", e);
  }
}

export async function clearAll(): Promise<void> {
  try {
    const state = await invoke<AppState>("clear_all");
    appState.set(state);
  } catch (e) {
    console.error("Failed to clear all:", e);
  }
}

export async function setArtwork(path: string): Promise<void> {
  try {
    const state = await invoke<AppState>("set_artwork", { path });
    appState.set(state);
  } catch (e) {
    console.error("Failed to set artwork:", e);
  }
}

export async function deleteArtwork(): Promise<void> {
  try {
    const state = await invoke<AppState>("delete_artwork");
    appState.set(state);
  } catch (e) {
    console.error("Failed to delete artwork:", e);
  }
}

export async function setPodcastName(name: string): Promise<void> {
  try {
    const state = await invoke<AppState>("set_podcast_name", { name });
    appState.set(state);
  } catch (e) {
    console.error("Failed to set podcast name:", e);
  }
}

export async function startServer(): Promise<void> {
  try {
    const url = await invoke<string>("start_server");
    serverUrl.set(url);
    isServerRunning.set(true);
  } catch (e) {
    console.error("Failed to start server:", e);
  }
}

export async function stopServer(): Promise<void> {
  try {
    await invoke("stop_server");
    serverUrl.set(null);
    isServerRunning.set(false);
  } catch (e) {
    console.error("Failed to stop server:", e);
  }
}
