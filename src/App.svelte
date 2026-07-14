<script lang="ts">
  import { onMount } from "svelte";
  import { appState, lastError, loadState, setPodcastName } from "./stores/app";
  import DropZone from "./lib/DropZone.svelte";
  import Artwork from "./lib/Artwork.svelte";
  import ServerControls from "./lib/ServerControls.svelte";
  import FileList from "./lib/FileList.svelte";

  let podcastName = $state("");
  let lastServerName = "";

  // Sync from backend state, but only when the backend value actually
  // changes — other commands (e.g. adding files) refresh the whole state
  // and must not clobber a name the user is still typing.
  $effect(() => {
    if ($appState.podcast_name !== lastServerName) {
      lastServerName = $appState.podcast_name;
      podcastName = $appState.podcast_name;
    }
  });

  onMount(() => {
    loadState();
  });
</script>

{#if $lastError}
  <div class="error-bar">{$lastError}</div>
{/if}

<main>
  <div class="left-panel">
    <h1>Podcasterator</h1>

    <DropZone />

    <Artwork />

    <div class="podcast-name">
      <label for="podcast-name">Podcast Name:</label>
      <input
        type="text"
        id="podcast-name"
        bind:value={podcastName}
        onchange={() => setPodcastName(podcastName)}
      />
    </div>

    <ServerControls />
  </div>

  <div class="right-panel">
    <FileList />
  </div>
</main>

<style>
  .error-bar {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    padding: 8px 16px;
    background: #e74c3c;
    color: white;
    font-size: 13px;
    text-align: center;
    z-index: 1000;
  }

  main {
    display: flex;
    height: 100vh;
    width: 100%;
  }

  .left-panel {
    width: 40%;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    border-right: 1px solid #ddd;
    background: #fafafa;
  }

  .right-panel {
    width: 60%;
    padding: 20px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  h1 {
    font-size: 24px;
    font-weight: bold;
    text-align: center;
    margin-bottom: 8px;
  }

  .podcast-name {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .podcast-name label {
    font-weight: 500;
    font-size: 13px;
  }
</style>
