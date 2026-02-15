<script lang="ts">
  import {
    appState,
    deleteFile,
    renameFile,
    moveFile,
    alphabetize,
    reverse,
    clearAll,
  } from "../stores/app";

  let editingIndex = $state<number | null>(null);
  let editingName = $state("");

  function startRename(index: number, currentName: string) {
    editingIndex = index;
    // Remove extension for editing
    const lastDot = currentName.lastIndexOf(".");
    editingName = lastDot > 0 ? currentName.substring(0, lastDot) : currentName;
  }

  async function saveRename(index: number) {
    if (editingIndex !== index) return; // Already cancelled
    if (editingName.trim()) {
      await renameFile(index, editingName.trim());
    }
    editingIndex = null;
    editingName = "";
  }

  function cancelRename() {
    editingIndex = null;
    editingName = "";
  }

  function truncateName(name: string, maxLength: number = 40): string {
    if (name.length <= maxLength) return name;
    return name.substring(0, maxLength - 3) + "...";
  }
</script>

<div class="file-list-container">
  <div class="header">
    <span class="file-count">{$appState.files.length} files</span>
    <div class="actions">
      <button onclick={clearAll} disabled={$appState.files.length === 0}>
        Clear All
      </button>
      <button onclick={alphabetize} disabled={$appState.files.length === 0}>
        Alphabetize
      </button>
      <button onclick={reverse} disabled={$appState.files.length === 0}>
        Reverse
      </button>
    </div>
  </div>

  <div class="file-list">
    {#each $appState.files as file, index (file.id)}
      <div class="file-item">
        <div class="file-controls">
          <button
            class="icon-button"
            onclick={() => moveFile(index, "up")}
            disabled={index === 0}
            title="Move up"
          >
            ↑
          </button>
          <button
            class="icon-button"
            onclick={() => moveFile(index, "down")}
            disabled={index === $appState.files.length - 1}
            title="Move down"
          >
            ↓
          </button>
        </div>

        {#if editingIndex === index}
          <input
            type="text"
            class="rename-input"
            bind:value={editingName}
            onkeydown={(e) => {
              if (e.key === "Enter") saveRename(index);
              if (e.key === "Escape") cancelRename();
            }}
            onblur={() => saveRename(index)}
          />
        {:else}
          <span class="file-name" title={file.display_name}>
            {truncateName(file.display_name)}
          </span>
        {/if}

        <div class="file-actions">
          <button
            class="icon-button"
            onclick={() => startRename(index, file.display_name)}
            title="Rename"
          >
            ✏️
          </button>
          <button
            class="icon-button"
            onclick={() => deleteFile(index)}
            title="Delete"
          >
            ×
          </button>
        </div>
      </div>
    {/each}

    {#if $appState.files.length === 0}
      <div class="empty-state">
        <p>No files added yet</p>
        <p class="hint">Drag files here or use the buttons on the left</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .file-list-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
    padding-bottom: 12px;
    border-bottom: 1px solid #eee;
  }

  .file-count {
    font-weight: 500;
    color: #666;
  }

  .actions {
    display: flex;
    gap: 8px;
  }

  .file-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px;
    background: white;
    border: 1px solid #eee;
    border-radius: 4px;
  }

  .file-item:hover {
    border-color: #ddd;
  }

  .file-controls {
    display: flex;
    gap: 2px;
  }

  .file-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .rename-input {
    flex: 1;
    padding: 4px 8px;
  }

  .file-actions {
    display: flex;
    gap: 2px;
  }

  .icon-button {
    padding: 4px 8px;
    min-width: 28px;
    font-size: 14px;
  }

  .empty-state {
    text-align: center;
    padding: 40px 20px;
    color: #999;
  }

  .empty-state p {
    margin: 4px 0;
  }

  .hint {
    font-size: 12px;
  }
</style>
