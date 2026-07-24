<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { onMount } from "svelte";
  import { appState, setArtwork, deleteArtwork } from "../stores/app";
  import { isPositionInside } from "./dragPosition";

  let artworkUrl = $derived(
    $appState.artwork_path ? convertFileSrc($appState.artwork_path) : null
  );
  let artworkDisplay: HTMLDivElement | null = null;
  let draggedImagePath: string | null = null;
  let isDragOver = $state(false);

  const IMAGE_EXTENSIONS = new Set([
    "png",
    "jpg",
    "jpeg",
    "gif",
    "bmp",
    "tiff",
  ]);

  function findImagePath(paths: string[]): string | null {
    return (
      paths.find((path) => {
        const extension = path.split(".").pop()?.toLowerCase();
        return extension ? IMAGE_EXTENSIONS.has(extension) : false;
      }) ?? null
    );
  }

  onMount(() => {
    const unlisten = getCurrentWebview().onDragDropEvent(async (event) => {
      const payload = event.payload;

      if (payload.type === "enter") {
        draggedImagePath = findImagePath(payload.paths);
        isDragOver =
          draggedImagePath !== null &&
          isPositionInside(payload.position, artworkDisplay);
      } else if (payload.type === "over") {
        isDragOver =
          draggedImagePath !== null &&
          isPositionInside(payload.position, artworkDisplay);
      } else if (payload.type === "drop") {
        const imagePath = findImagePath(payload.paths);
        const droppedHere = isPositionInside(
          payload.position,
          artworkDisplay
        );
        draggedImagePath = null;
        isDragOver = false;

        if (droppedHere && imagePath) {
          await setArtwork(imagePath);
        }
      } else {
        draggedImagePath = null;
        isDragOver = false;
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  });

  async function selectArtwork() {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "Images",
          extensions: ["png", "jpg", "jpeg", "gif", "bmp", "tiff"],
        },
      ],
    });

    if (selected && !Array.isArray(selected)) {
      await setArtwork(selected);
    }
  }

  async function handleDelete() {
    await deleteArtwork();
  }
</script>

<div class="artwork-container">
  <div
    class="artwork-display"
    class:dragover={isDragOver}
    bind:this={artworkDisplay}
  >
    <button
      type="button"
      class="artwork-picker"
      aria-label={artworkUrl
        ? "Replace podcast artwork"
        : "Choose podcast artwork"}
      onclick={selectArtwork}
    >
      {#if artworkUrl}
        <img src={artworkUrl} alt="Podcast artwork" />
      {:else}
        <span class="no-artwork">
          <span>No artwork set</span>
          <span class="hint">Click or drop image</span>
        </span>
      {/if}
    </button>

    {#if artworkUrl}
      <button
        type="button"
        class="delete-overlay"
        onclick={handleDelete}
        aria-label="Delete podcast artwork"
        title="Delete artwork"
      >
        🗑
      </button>
    {/if}

    {#if isDragOver}
      <div class="drop-overlay">Drop image here</div>
    {/if}
  </div>
</div>

<style>
  .artwork-container {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .artwork-display {
    width: 150px;
    height: 150px;
    border: 2px solid #ddd;
    border-radius: 8px;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    background: white;
    transition: border-color 0.2s;
    position: relative;
  }

  .artwork-display:hover {
    border-color: #4a90d9;
  }

  .artwork-display.dragover {
    border-color: #4a90d9;
    box-shadow: 0 0 0 3px rgba(74, 144, 217, 0.2);
  }

  .artwork-picker {
    width: 100%;
    height: 100%;
    min-width: 0;
    padding: 0;
    margin: 0;
    border: 0;
    border-radius: 0;
    background: white;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .artwork-picker:focus-visible {
    outline: 3px solid rgba(74, 144, 217, 0.45);
    outline-offset: -3px;
  }

  .artwork-picker img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .delete-overlay {
    position: absolute;
    z-index: 1;
    top: 6px;
    right: 6px;
    width: 38px;
    height: 38px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.5);
    opacity: 0;
    transition: opacity 0.2s;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 20px;
    padding: 0;
    margin: 0;
    min-width: unset;
  }

  .artwork-display:hover .delete-overlay {
    opacity: 1;
  }

  .delete-overlay:focus-visible {
    opacity: 1;
    outline: 3px solid rgba(74, 144, 217, 0.55);
  }

  .artwork-display.dragover .delete-overlay {
    opacity: 0;
  }

  .drop-overlay {
    position: absolute;
    inset: 0;
    z-index: 2;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(240, 247, 255, 0.92);
    color: #3a7cc0;
    font-weight: 600;
    pointer-events: none;
  }

  .no-artwork {
    text-align: center;
    color: #999;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .hint {
    font-size: 12px;
  }
</style>
