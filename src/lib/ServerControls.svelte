<script lang="ts">
  import { writeText } from "@tauri-apps/plugin-clipboard-manager";
  import { serverUrl, isServerRunning, startServer, stopServer } from "../stores/app";

  let copied = $state(false);

  async function copyUrl() {
    if ($serverUrl) {
      await writeText($serverUrl);
      copied = true;
      setTimeout(() => (copied = false), 2000);
    }
  }
</script>

<div class="server-controls">
  {#if $isServerRunning}
    <button class="danger" onclick={stopServer}>Stop Server</button>

    <div class="url-container">
      <span class="url">{$serverUrl}</span>
      <button onclick={copyUrl}>{copied ? "Copied!" : "Copy URL"}</button>
    </div>
  {:else}
    <button class="primary" onclick={startServer}>
      Launch Local Podcast Server
    </button>
  {/if}
</div>

<style>
  .server-controls {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-top: auto;
    padding-top: 16px;
    border-top: 1px solid #eee;
  }

  .url-container {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .url {
    flex: 1;
    font-size: 12px;
    color: #4a90d9;
    word-break: break-all;
    padding: 6px 10px;
    background: #f0f7ff;
    border-radius: 4px;
  }
</style>
