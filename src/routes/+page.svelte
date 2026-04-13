<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  interface AppConfig {
    session_cookie: string;
    poll_interval_secs: number;
    org_id: string;
  }

  interface UsageData {
    session_percent: number;
    session_reset_minutes: number;
    weekly_percent: number;
    weekly_reset_minutes: number;
    plan_type: string;
  }

  let config = $state<AppConfig>({
    session_cookie: "",
    poll_interval_secs: 120,
    org_id: "",
  });

  let usage = $state<UsageData | null>(null);
  let error = $state<string>("");
  let saving = $state(false);
  let refreshing = $state(false);

  onMount(async () => {
    await loadConfig();
    await loadUsage();
  });

  async function loadConfig() {
    try {
      const loaded = await invoke<AppConfig>("get_config");
      config = loaded;
    } catch (e) {
      console.error("Failed to load config:", e);
    }
  }

  async function loadUsage() {
    try {
      const data = await invoke<UsageData | null>("get_usage");
      usage = data;
    } catch (e) {
      console.error("Failed to load usage:", e);
    }
  }

  async function saveConfig() {
    saving = true;
    error = "";
    try {
      await invoke("save_config", { config });
      error = "Settings saved!";
      setTimeout(() => (error = ""), 3000);
    } catch (e) {
      error = `Error saving settings: ${e}`;
    } finally {
      saving = false;
    }
  }

  async function refreshNow() {
    refreshing = true;
    error = "";
    try {
      const data = await invoke<UsageData>("refresh_usage");
      usage = data;
    } catch (e) {
      error = `Error refreshing: ${e}`;
    } finally {
      refreshing = false;
    }
  }

  function formatTime(minutes: number): string {
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    return `${hours}h ${mins}m`;
  }
</script>

<main class="settings">
  <h1>Claude Pro Usage Tracker</h1>

  <section class="status">
    {#if usage}
      <div class="usage-card">
        <h2>{usage.plan_type} Account</h2>
        <div class="metrics">
          <div class="metric">
            <span class="label">Session Usage:</span>
            <span class="value">{usage.session_percent.toFixed(1)}%</span>
            <span class="reset">Resets in {formatTime(usage.session_reset_minutes)}</span>
          </div>
          <div class="metric">
            <span class="label">Weekly Usage:</span>
            <span class="value">{usage.weekly_percent.toFixed(1)}%</span>
            <span class="reset">Resets in {formatTime(usage.weekly_reset_minutes)}</span>
          </div>
        </div>
      </div>
    {:else}
      <p>Configure your settings below to see usage stats.</p>
    {/if}
  </section>

  <section class="settings-form">
    <h2>Settings</h2>

    <div class="form-group">
      <label for="org-id">Organization ID</label>
      <input
        id="org-id"
        type="text"
        placeholder="Your Claude.ai organization ID"
        bind:value={config.org_id}
      />
      <small>Found in claude.ai/settings/usage API calls</small>
    </div>

    <div class="form-group">
      <label for="cookie">Session Cookie</label>
      <input
        id="cookie"
        type="password"
        placeholder="sessionKey=sk-ant-..."
        bind:value={config.session_cookie}
      />
      <small>
        Get this from DevTools → Application tab → Cookies → sessionKey on claude.ai
      </small>
    </div>

    <div class="form-group">
      <label for="interval">Poll Interval (seconds)</label>
      <select id="interval" bind:value={config.poll_interval_secs}>
        <option value={60}>1 minute</option>
        <option value={120}>2 minutes</option>
        <option value={300}>5 minutes</option>
        <option value={600}>10 minutes</option>
      </select>
    </div>

    {#if error}
      <div class="message" class:error={error.includes("Error")}>
        {error}
      </div>
    {/if}

    <div class="button-group">
      <button onclick={saveConfig} disabled={saving}>
        {saving ? "Saving..." : "Save Settings"}
      </button>
      <button onclick={refreshNow} disabled={refreshing || !usage}>
        {refreshing ? "Refreshing..." : "Refresh Now"}
      </button>
    </div>
  </section>
</main>

<style>
  :root {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen,
      Ubuntu, Cantarell, sans-serif;
    font-size: 14px;
    color: #0f0f0f;
    background-color: #f6f6f6;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #1e1e1e;
    }
  }

  main {
    max-width: 500px;
    margin: 0 auto;
    padding: 20px;
  }

  h1 {
    text-align: center;
    margin-bottom: 30px;
    font-size: 24px;
  }

  h2 {
    font-size: 18px;
    margin-bottom: 15px;
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
    padding-bottom: 10px;
  }

  section {
    margin-bottom: 30px;
    padding: 15px;
    border-radius: 8px;
    background-color: #ffffff;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  @media (prefers-color-scheme: dark) {
    section {
      background-color: #2a2a2a;
    }
  }

  .usage-card {
    text-align: center;
  }

  .metrics {
    display: grid;
    gap: 15px;
    margin-top: 15px;
  }

  .metric {
    display: flex;
    flex-direction: column;
    gap: 5px;
    padding: 10px;
    background-color: #f0f0f0;
    border-radius: 6px;
  }

  @media (prefers-color-scheme: dark) {
    .metric {
      background-color: #333333;
    }
  }

  .label {
    font-weight: 500;
    font-size: 12px;
    text-transform: uppercase;
    opacity: 0.7;
  }

  .value {
    font-size: 28px;
    font-weight: bold;
    color: #396cd8;
  }

  .reset {
    font-size: 12px;
    opacity: 0.6;
  }

  .form-group {
    margin-bottom: 20px;
  }

  label {
    display: block;
    margin-bottom: 6px;
    font-weight: 500;
  }

  input,
  select {
    width: 100%;
    padding: 8px 12px;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 14px;
    box-sizing: border-box;
    background-color: #f9f9f9;
    color: #0f0f0f;
  }

  @media (prefers-color-scheme: dark) {
    input,
    select {
      background-color: #333333;
      color: #f6f6f6;
      border-color: #444444;
    }
  }

  input:focus,
  select:focus {
    outline: none;
    border-color: #396cd8;
    box-shadow: 0 0 0 3px rgba(57, 108, 216, 0.1);
  }

  small {
    display: block;
    margin-top: 4px;
    font-size: 12px;
    opacity: 0.6;
  }

  .message {
    padding: 10px 12px;
    border-radius: 6px;
    margin-bottom: 15px;
    background-color: #d4edda;
    color: #155724;
    font-size: 13px;
  }

  .message.error {
    background-color: #f8d7da;
    color: #721c24;
  }

  @media (prefers-color-scheme: dark) {
    .message {
      background-color: #2d5a3d;
      color: #a8d5a8;
    }

    .message.error {
      background-color: #5a2d2d;
      color: #d5a8a8;
    }
  }

  .button-group {
    display: flex;
    gap: 10px;
  }

  button {
    flex: 1;
    padding: 10px 16px;
    border: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    background-color: #396cd8;
    color: white;
    transition: background-color 0.2s;
  }

  button:hover:not(:disabled) {
    background-color: #2d57b8;
  }

  button:active:not(:disabled) {
    background-color: #1e3a7a;
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
