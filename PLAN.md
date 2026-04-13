# Claude Usage Tracker — Windows System Tray App

## Context

The user wants a lightweight Windows app that lives in the system tray and shows their **Claude Pro/Max subscription usage** in real-time — specifically the session usage %, weekly usage %, and reset timers (as shown on claude.ai/settings/usage). The app should later expand to Mac, Android, and iOS.

**Chosen stack**: Tauri v2 (Rust backend + Svelte frontend)  
**Auth method**: Manual session cookie paste  
**Display**: Color-coded tray icon + tooltip with usage stats

---

## Step 1: Scaffold Tauri v2 Project

Command: `npm create tauri-app@latest claude-usage -- --template svelte-ts`

Create the project structure using Tauri v2 with a Svelte frontend.

```
claude-usage/
├── src-tauri/                 # Rust backend
│   ├── src/
│   │   ├── lib.rs             # Tauri app setup, commands, tray
│   │   ├── api.rs             # Claude.ai API client (fetch usage)
│   │   └── config.rs          # Config persistence (cookie, settings)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json       # Tauri permissions
│   └── icons/                 # Tray icons (green/yellow/red SVGs)
│       ├── icon-green.png
│       ├── icon-yellow.png
│       └── icon-red.png
├── src/                       # Svelte frontend
│   ├── App.svelte             # Main app (settings UI)
│   ├── main.ts                # Entry point
│   └── lib/
│       └── types.ts           # Shared TypeScript types
├── static/
├── package.json
├── svelte.config.js
├── tsconfig.json
├── vite.config.ts
└── .gitignore
```

**Key Rust dependencies** (`Cargo.toml`):
- `tauri` v2 — app framework + system tray
- `tauri-plugin-store` — persist cookie securely on disk
- `reqwest` — HTTP client for claude.ai API
- `serde` / `serde_json` — JSON serialization
- `tokio` — async runtime (included via Tauri)

**Key frontend dependencies** (`package.json`):
- `@tauri-apps/api` v2 — invoke Rust commands from JS
- `svelte` — lightweight UI framework
- `vite` — build tool
- `typescript`

---

## Step 2: API Discovery & Client (`src-tauri/src/api.rs`)

### 2a: Discover the claude.ai usage API endpoint

The usage data (session %, weekly %, reset timers) is fetched by the claude.ai web app from an internal API. We need to identify the exact endpoint. The likely endpoint pattern is:

```
GET https://api.claude.ai/api/organizations/{org_id}/usage
```

With headers:
```
Cookie: sessionKey=sk-ant-sid01-...
Content-Type: application/json
```

**Discovery approach**: The user opens DevTools on claude.ai/settings/usage, checks the Network tab, and provides us the exact endpoint + response shape. We'll code the client to match.

### 2b: Implement the API client

```rust
// api.rs — key struct
pub struct UsageData {
    pub session_percent: f32,        // e.g. 91.0
    pub session_reset_minutes: u32,  // e.g. 157 (2h 37m)
    pub weekly_percent: f32,         // e.g. 51.0
    pub weekly_reset_minutes: u32,   // e.g. 1057 (17h 37m)
    pub plan_type: String,           // e.g. "Pro"
}

// Fetch usage from claude.ai API using stored session cookie
pub async fn fetch_usage(cookie: &str) -> Result<UsageData, Error> { ... }
```

- Poll every **2 minutes** by default (configurable)
- Handle errors gracefully (expired cookie → notify user)
- Return structured data to the tray manager

---

## Step 3: Configuration & Persistence (`src-tauri/src/config.rs`)

Use `tauri-plugin-store` to persist settings to disk:

```rust
pub struct AppConfig {
    pub session_cookie: String,      // The sessionKey cookie value
    pub poll_interval_secs: u64,     // Default: 120 (2 min)
    pub org_id: String,              // Claude.ai organization ID
}
```

- Store config in Tauri's app data directory
- Cookie stored locally (not transmitted anywhere except to claude.ai)

---

## Step 4: System Tray (`src-tauri/src/lib.rs`)

### Tauri v2 Tray API (confirmed):
- `TrayIconBuilder::with_id()` — create tray with ID
- `tray.set_tooltip()` — dynamic tooltip text
- `tray.set_icon()` — swap icon dynamically (for color changes)
- `Menu::with_items()` + `MenuItem::with_id()` — right-click context menu
- `menu_item.set_text()` — update menu item text dynamically
- Left-click event via `on_tray_icon_event` handler
- Tauri v2 requires `tray-icon` feature enabled in `Cargo.toml`

### Tray icon behavior:
| Usage Level | Icon Color | Meaning |
|---|---|---|
| 0-50% | Green | Plenty of usage left |
| 51-80% | Yellow | Moderate usage |
| 81-100% | Red | Running low |

### Tooltip format:
```
Claude Pro Usage
Session: 91% used (resets in 2h 37m)
Weekly: 51% used (resets in 17h 37m)
```

### Tray menu (right-click):
- **Refresh Now** — force an immediate poll
- **Settings** — open the settings window
- **Quit** — exit the app

### Implementation:
- Use Tauri v2's `tray::TrayIconBuilder` for the system tray
- Spawn a background `tokio` task that polls the API on interval
- On each poll, update the tray icon color + tooltip text
- Use Tauri's event system to communicate between backend and frontend

---

## Step 5: Settings UI (`src/App.svelte`)

Minimal settings window (opens when clicking tray icon or "Settings" menu):

- **Session Cookie** — text input to paste the `sessionKey` cookie value
- **Organization ID** — text input (or auto-detect from cookie)
- **Poll Interval** — dropdown (1 min / 2 min / 5 min)
- **Current Status** — show live usage stats in the window
- **Save** button — persists config and starts/restarts polling

Instructions in the UI explaining how to get the session cookie from the browser (DevTools → Application → Cookies → sessionKey).

---

## Step 6: Cookie Expiry Notifications

- When the API returns 401/403 → show a Windows notification:  
  `"Claude Usage: Session expired. Please update your cookie in Settings."`
- When usage exceeds 80% → optional notification:  
  `"Claude Usage: Session usage at 91%. Resets in 2h 37m."`

Use Tauri's notification plugin or native Windows toast notifications.

---

## Build Order

1. **Scaffold** — `npm create tauri-app` with Svelte + TypeScript template
2. **Tray icon** — get a basic system tray working with a static tooltip
3. **Settings UI** — cookie input + save to store
4. **API client** — fetch usage data from claude.ai
5. **Wire it up** — polling loop → updates tray icon/tooltip
6. **Polish** — error handling, notifications, icon colors
7. **Package** — build Windows `.msi` / `.exe` installer via Tauri

---

## Verification

1. Build and run the app with `cargo tauri dev`
2. Paste a valid session cookie in Settings
3. Verify the tray icon appears and tooltip shows correct usage %s
4. Wait for a poll cycle and confirm the data refreshes
5. Test with an expired cookie — verify the error notification appears
6. Build a release with `cargo tauri build` and test the installer

---

## Important Note: API Endpoint Discovery

There is **no public Anthropic API** for querying Claude Pro/Max subscription usage percentages. The data shown on `claude.ai/settings/usage` is served by claude.ai's internal API. Before coding the API client, we need to:

1. User opens `claude.ai/settings/usage` in Chrome
2. Opens DevTools → Network tab
3. Refreshes the page and identifies the XHR request(s) that return usage data
4. We note the exact endpoint URL, request headers, and response JSON shape
5. Code the Rust client to match

This is the **critical dependency** for the whole app — we'll tackle it first in Step 2.

---

## Future Expansion (not in scope now)

- **Mac**: Tauri v2 supports macOS natively — same codebase, menu bar app
- **Android/iOS**: Tauri v2 has mobile support — widget or notification-based display
- **Browser extension**: Auto-sync session cookie to eliminate manual paste
- **Usage history**: SQLite database to track usage over time with charts
- **Claude Code log parsing**: Parse `~/.claude/projects/*/*.jsonl` for local token usage tracking (zero config, no cookie needed)
