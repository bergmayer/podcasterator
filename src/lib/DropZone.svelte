<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { onMount } from "svelte";
  import { addFiles } from "../stores/app";

  let isDragOver = $state(false);

  onMount(() => {
    const unlisten = getCurrentWebview().onDragDropEvent(async (event) => {
      if (event.payload.type === "enter" || event.payload.type === "over") {
        isDragOver = true;
      } else if (event.payload.type === "leave") {
        isDragOver = false;
      } else if (event.payload.type === "drop") {
        isDragOver = false;
        const paths = event.payload.paths;
        if (paths && paths.length > 0) {
          await addFiles(paths);
        }
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  });

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    isDragOver = true;
  }

  function handleDragLeave() {
    isDragOver = false;
  }

  function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragOver = false;
  }

  async function selectFiles() {
    const selected = await open({
      multiple: true,
      directory: false,
      filters: [
        {
          name: "Audio Files",
          extensions: ["mp3", "m4a", "mp4", "m4b"],
        },
      ],
    });

    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      if (paths.length > 0) {
        await addFiles(paths);
      }
    }
  }

  async function selectFolder() {
    const selected = await open({
      multiple: false,
      directory: true,
    });

    if (selected && !Array.isArray(selected)) {
      await addFiles([selected]);
    }
  }
</script>

<div
  class="drop-zone"
  class:dragover={isDragOver}
  role="button"
  tabindex="0"
  ondragover={handleDragOver}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
>
  <div class="drop-content">
    <span class="icon">📁</span>
    <p>Drag audio files or folders here</p>
    <p class="subtext">Or click a button below</p>
  </div>

  <div class="buttons">
    <button onclick={selectFiles}>Select Files</button>
    <button onclick={selectFolder}>Select Folder</button>
  </div>
</div>

<style>
  .drop-zone {
    border: 2px dashed #ccc;
    border-radius: 8px;
    padding: 20px;
    text-align: center;
    transition: all 0.2s;
    background: white;
  }

  .drop-zone.dragover {
    border-color: #4a90d9;
    background: #f0f7ff;
  }

  .drop-content {
    margin-bottom: 16px;
  }

  .icon {
    font-size: 32px;
    display: block;
    margin-bottom: 8px;
  }

  p {
    margin: 4px 0;
    color: #666;
  }

  .subtext {
    font-size: 12px;
    color: #999;
  }

  .buttons {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .buttons button {
    width: 100%;
  }
</style>
