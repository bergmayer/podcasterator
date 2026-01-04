<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { appState, setArtwork, deleteArtwork } from "../stores/app";

  let artworkUrl = $derived(
    $appState.artwork_path ? convertFileSrc($appState.artwork_path) : null
  );

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
</script>

<div class="artwork-container">
  <div class="artwork-display" role="button" tabindex="0" onclick={selectArtwork} onkeydown={(e) => e.key === 'Enter' && selectArtwork()}>
    {#if artworkUrl}
      <img src={artworkUrl} alt="Podcast artwork" />
    {:else}
      <div class="no-artwork">
        <span>No artwork set</span>
        <span class="hint">Click to add</span>
      </div>
    {/if}
  </div>

  {#if $appState.artwork_path}
    <button class="danger" onclick={deleteArtwork}>Delete Artwork</button>
  {/if}
</div>

<style>
  .artwork-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }

  .artwork-display {
    width: 150px;
    height: 150px;
    border: 2px solid #ddd;
    border-radius: 8px;
    overflow: hidden;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    background: white;
    transition: border-color 0.2s;
  }

  .artwork-display:hover {
    border-color: #4a90d9;
  }

  .artwork-display img {
    width: 100%;
    height: 100%;
    object-fit: cover;
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

  button {
    width: 150px;
  }
</style>
