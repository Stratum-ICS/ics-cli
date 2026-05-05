# Obsidian ICS plugin — P0 implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create the **sibling** git repository for the Obsidian plugin and ship **P0**: settings (`ics` binary path, optional Stratum base URL), **ICS output** sidebar view, **ribbon** focus, and **command palette** actions that spawn the host **`ics`** with vault root as `cwd` for `status`, `commit`, `log`, and `diff` — matching [`2026-05-05-obsidian-ics-plugin-design.md`](../specs/2026-05-05-obsidian-ics-plugin-design.md) and UX §11.

**Architecture:** TypeScript plugin built with **esbuild**; **`IcsRunner`** wraps `child_process.spawn` (serialized queue); **`IcsOutputView`** extends **`ItemView`** for streaming log text; **`IcsPlugin`** registers commands and a **`PluginSettingTab`**. No Stratum HTTP in the plugin.

**Tech Stack:** `obsidian` (peer), `typescript`, `esbuild`, `@types/node`; optional later **Vitest** for pure helpers only.

**Normative CLI interface (this repo’s `ics` binary):** `ics status`, `ics commit -m "…"`, `ics log`, `ics diff [PATH …]` (see `ics-cli` `src/main.rs`). Plugin passes **vault-relative** paths for “diff current file”.

---

### File map (sibling repo root)

Assume repository name **`ics-obsidian`** (rename if you prefer). All paths below are relative to that repo root after `git init`.

| Path | Responsibility |
|------|----------------|
| `package.json` | Scripts: `build`, `version`; deps |
| `manifest.json` | Obsidian plugin manifest (`id`, `minAppVersion`, …) |
| `versions.json` | Obsidian community version map |
| `tsconfig.json` | `strict`, `moduleResolution: bundler`, `target ES2020` |
| `esbuild.config.mjs` | Bundle `main.js` from `src/main.ts` |
| `src/main.ts` | `IcsPlugin`: ribbon, commands, view registration |
| `src/settings.ts` | `IcsSettings` interface + defaults + `SETTINGS_TAB` strings |
| `src/runner.ts` | `IcsRunner`: spawn, env merge, queue, stream callbacks |
| `src/views/IcsOutputView.ts` | `ItemView` subclass; append / clear buffer |
| `README.md` | Install from BRAT / manual; Flatpak `ics` path; manual QA checklist |

---

### Task 1: Create sibling repository and toolchain

**Files:**
- Create: `package.json`, `tsconfig.json`, `esbuild.config.mjs`, `.gitignore`

- [ ] **Step 1: Create directory and git**

```bash
mkdir -p ics-obsidian && cd ics-obsidian && git init
```

- [ ] **Step 2: `.gitignore`**

```
node_modules/
main.js
.DS_Store
```

- [ ] **Step 3: `package.json`**

```json
{
  "name": "ics-obsidian",
  "version": "0.1.0",
  "description": "Obsidian UI for the ics CLI (local history + Stratum ICS)",
  "main": "main.js",
  "scripts": {
    "build": "node esbuild.config.mjs",
    "dev": "node esbuild.config.mjs --watch"
  },
  "devDependencies": {
    "@types/node": "^20.10.0",
    "builtin-modules": "^4.0.0",
    "esbuild": "^0.24.0",
    "obsidian": "^1.6.0",
    "typescript": "^5.6.0"
  }
}
```

- [ ] **Step 4: `tsconfig.json`**

```json
{
  "compilerOptions": {
    "baseUrl": ".",
    "inlineSourceMap": true,
    "inlineSources": true,
    "module": "ESNext",
    "target": "ES2020",
    "allowJs": true,
    "noImplicitAny": true,
    "moduleResolution": "bundler",
    "strictNullChecks": true,
    "importHelpers": true,
    "isolatedModules": true,
    "verbatimModuleSyntax": false,
    "lib": ["DOM", "ES2020"]
  },
  "include": ["src/**/*.ts"]
}
```

- [ ] **Step 5: `esbuild.config.mjs`**

```javascript
import esbuild from "esbuild";
import process from "process";
import builtins from "builtin-modules";

const banner = `/*
THIS IS A GENERATED FILE
*/
`;

const prod = process.argv[2] === "production";

const context = await esbuild.context({
  banner: { js: banner },
  entryPoints: ["src/main.ts"],
  bundle: true,
  external: [
    "obsidian",
    "electron",
    "@codemirror/autocomplete",
    "@codemirror/collab",
    "@codemirror/commands",
    "@codemirror/language",
    "@codemirror/lint",
    "@codemirror/search",
    "@codemirror/state",
    "@codemirror/view",
    "@lezer/common",
    "@lezer/highlight",
    "@lezer/lr",
    ...builtins,
  ],
  format: "cjs",
  target: "es2020",
  logLevel: "info",
  sourcemap: prod ? false : "inline",
  treeShaking: true,
  outfile: "main.js",
});

if (prod) await context.rebuild();
else await context.watch();

process.exit(prod ? 0 : undefined);
```

- [ ] **Step 6: Install and verify build (empty entry for now)**

```bash
npm install && printf '// placeholder\n' > src/main.ts && npm run build && test -f main.js
```

Expected: `main.js` exists (will be replaced in Task 4).

- [ ] **Step 7: Commit**

```bash
git add package.json tsconfig.json esbuild.config.mjs .gitignore src/main.ts main.js
git commit -m "chore: scaffold obsidian plugin toolchain"
```

---

### Task 2: `manifest.json` and `versions.json`

**Files:**
- Create: `manifest.json`, `versions.json`

- [ ] **Step 1: `manifest.json`**

```json
{
  "id": "ics-obsidian",
  "name": "ICS",
  "version": "0.1.0",
  "minAppVersion": "1.6.0",
  "description": "Run the ics CLI from your vault: status, commit, log, diff.",
  "author": "Your Name",
  "isDesktopOnly": true
}
```

- [ ] **Step 2: `versions.json`**

```json
{
  "0.1.0": "1.6.0"
}
```

- [ ] **Step 3: Commit**

```bash
git add manifest.json versions.json && git commit -m "chore: add obsidian manifest"
```

---

### Task 3: Settings types and defaults

**Files:**
- Create: `src/settings.ts`

- [ ] **Step 1: Create `src/settings.ts`**

```typescript
export interface IcsSettings {
  /** Executable or bare name resolved via PATH */
  icsBinaryPath: string;
  /** When non-empty, passed as STRATUM_BASE_URL to the child process */
  stratumBaseUrl: string;
}

export const DEFAULT_SETTINGS: IcsSettings = {
  icsBinaryPath: "ics",
  stratumBaseUrl: "",
};
```

- [ ] **Step 2: Commit**

```bash
git add src/settings.ts && git commit -m "feat: add plugin settings types"
```

---

### Task 4: `IcsRunner` (spawn, queue, streaming)

**Files:**
- Create: `src/runner.ts`

- [ ] **Step 1: Implement `src/runner.ts`**

```typescript
import { spawn, type ChildProcessWithoutNullStreams } from "child_process";

export type StreamHandler = (chunk: string, stream: "stdout" | "stderr") => void;

export interface RunOptions {
  cwd: string;
  env: NodeJS.ProcessEnv;
  onChunk: StreamHandler;
  onStart?: () => void;
}

/**
 * Serialize runs so two palette actions never interleave output.
 */
export class IcsRunner {
  private queue: Promise<void> = Promise.resolve();

  run(
    binary: string,
    args: string[],
    opts: RunOptions
  ): Promise<{ code: number | null; signal: NodeJS.Signals | null }> {
    const task = async () => {
      opts.onStart?.();
      return await this.spawnOnce(binary, args, opts);
    };
    const next = this.queue.then(task);
    this.queue = next.then(() => undefined);
    return next;
  }

  private spawnOnce(
    binary: string,
    args: string[],
    opts: RunOptions
  ): Promise<{ code: number | null; signal: NodeJS.Signals | null }> {
    return new Promise((resolve, reject) => {
      let child: ChildProcessWithoutNullStreams;
      try {
        child = spawn(binary, args, {
          cwd: opts.cwd,
          env: opts.env,
          shell: false,
        });
      } catch (e) {
        reject(e);
        return;
      }

      const pump = (stream: "stdout" | "stderr", data: Buffer) => {
        opts.onChunk(data.toString("utf8"), stream);
      };

      child.stdout.on("data", (d: Buffer) => pump("stdout", d));
      child.stderr.on("data", (d: Buffer) => pump("stderr", d));

      child.on("error", (err) => reject(err));
      child.on("close", (code, signal) => {
        resolve({ code, signal });
      });
    });
  }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/runner.ts && git commit -m "feat: add ics process runner with queue"
```

---

### Task 5: `IcsOutputView` (`ItemView`)

**Files:**
- Create: `src/views/IcsOutputView.ts`

- [ ] **Step 1: Create `src/views/IcsOutputView.ts`**

```typescript
import { ItemView, WorkspaceLeaf } from "obsidian";

export const ICS_OUTPUT_VIEW_TYPE = "ics-output";

export class IcsOutputView extends ItemView {
  private buffer = "";

  constructor(leaf: WorkspaceLeaf) {
    super(leaf);
  }

  getViewType(): string {
    return ICS_OUTPUT_VIEW_TYPE;
  }

  getDisplayText(): string {
    return "ICS output";
  }

  getIcon(): string {
    return "scroll-text";
  }

  async onOpen(): Promise<void> {
    const el = this.containerEl.children[1] as HTMLElement;
    el.empty();
    el.createEl("pre", {
      cls: "ics-output-pre",
      text: this.buffer || "(no output yet — run an ICS command)",
    });
  }

  async onClose(): Promise<void> {}

  clear(): void {
    this.buffer = "";
    this.render();
  }

  append(text: string): void {
    this.buffer += text;
    if (this.buffer.length > 500_000) {
      this.buffer = this.buffer.slice(-400_000);
    }
    this.render();
  }

  private render(): void {
    const el = this.containerEl.children[1] as HTMLElement;
    if (!el) return;
    el.empty();
    el.createEl("pre", {
      cls: "ics-output-pre",
      text: this.buffer || "(empty)",
    });
  }
}
```

- [ ] **Step 2: Add `styles.css` at repo root** (Obsidian loads it automatically when placed next to `main.js` in the plugin folder — no manifest key required)

```css
.ics-output-pre {
  margin: 0;
  padding: 0.75em;
  font-size: 0.85em;
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 100%;
  overflow-y: auto;
}
```

- [ ] **Step 3: Commit**

```bash
git add src/views/IcsOutputView.ts styles.css
git commit -m "feat: add ICS output sidebar view"
```

---

### Task 6: `main.ts` — plugin core, ribbon, commands, settings tab

**Files:**
- Create: `src/main.ts` (replace placeholder)

- [ ] **Step 1: Implement `src/main.ts`** (single file — includes `CommitModal`; **`TFile.path`** is already vault-relative in Obsidian, so pass it directly to `ics diff`)

```typescript
import {
  App,
  Modal,
  Notice,
  Plugin,
  PluginSettingTab,
  Setting,
} from "obsidian";
import { DEFAULT_SETTINGS, type IcsSettings } from "./settings";
import { IcsRunner } from "./runner";
import { ICS_OUTPUT_VIEW_TYPE, IcsOutputView } from "./views/IcsOutputView";

function vaultBasePath(app: App): string {
  const adapter = app.vault.adapter;
  if ("getBasePath" in adapter && typeof adapter.getBasePath === "function") {
    return adapter.getBasePath();
  }
  throw new Error("Vault has no filesystem base path (ICS requires a local vault)");
}

function childEnv(settings: IcsSettings): NodeJS.ProcessEnv {
  const env = { ...process.env };
  if (settings.stratumBaseUrl.trim()) {
    env.STRATUM_BASE_URL = settings.stratumBaseUrl.trim();
  }
  return env;
}

class CommitModal extends Modal {
  private input!: HTMLTextAreaElement;
  /** Set by buttons before `close()`; `onClose` forwards to callback */
  private result: string | null = null;

  constructor(
    app: App,
    private readonly onDone: (message: string | null) => void
  ) {
    super(app);
  }

  onOpen(): void {
    const { contentEl } = this;
    contentEl.createEl("h2", { text: "Commit message" });
    this.input = contentEl.createEl("textarea");
    this.input.rows = 4;
    new Setting(contentEl)
      .addButton((b) =>
        b.setButtonText("Commit").onClick(() => {
          const v = this.input.value.trim();
          this.result = v.length ? v : null;
          this.close();
        })
      )
      .addButton((b) =>
        b.setButtonText("Cancel").onClick(() => {
          this.result = null;
          this.close();
        })
      );
  }

  onClose(): void {
    this.onDone(this.result);
  }
}

export default class IcsPlugin extends Plugin {
  settings: IcsSettings = { ...DEFAULT_SETTINGS };
  runner = new IcsRunner();

  async onload(): Promise<void> {
    await this.loadSettings();

    this.registerView(ICS_OUTPUT_VIEW_TYPE, (leaf) => new IcsOutputView(leaf));

    this.addRibbonIcon("scroll-text", "Open ICS output", () => {
      void this.ensureOutputView();
    });

    this.addCommand({
      id: "ics-status",
      name: "Status",
      callback: () => void this.runIcs(["status"]),
    });

    this.addCommand({
      id: "ics-log",
      name: "Log",
      callback: () => void this.runIcs(["log"]),
    });

    this.addCommand({
      id: "ics-diff",
      name: "Diff (vault)",
      callback: () => void this.runIcs(["diff"]),
    });

    this.addCommand({
      id: "ics-diff-active",
      name: "Diff (active file)",
      callback: () => void this.diffActiveFile(),
    });

    this.addCommand({
      id: "ics-commit",
      name: "Commit…",
      callback: () => void this.promptCommit(),
    });

    this.addSettingTab(new IcsSettingTab(this.app, this));
  }

  async loadSettings(): Promise<void> {
    this.settings = Object.assign({}, DEFAULT_SETTINGS, await this.loadData());
  }

  async saveSettings(): Promise<void> {
    await this.saveData(this.settings);
  }

  async ensureOutputView(): Promise<IcsOutputView> {
    const { workspace } = this.app;
    const existing = workspace.getLeavesOfType(ICS_OUTPUT_VIEW_TYPE);
    if (existing.length > 0) {
      const leaf = existing[0]!;
      await workspace.revealLeaf(leaf);
      return leaf.view as IcsOutputView;
    }
    const leaf = workspace.getRightLeaf(false);
    if (!leaf) {
      new Notice("Could not open right sidebar leaf");
      throw new Error("no right leaf");
    }
    await leaf.setViewState({ type: ICS_OUTPUT_VIEW_TYPE, active: true });
    await workspace.revealLeaf(leaf);
    return leaf.view as IcsOutputView;
  }

  async runIcs(args: string[]): Promise<void> {
    const view = await this.ensureOutputView();
    view.clear();
    const cwd = vaultBasePath(this.app);
    const bin = this.settings.icsBinaryPath.trim() || "ics";
    const env = childEnv(this.settings);

    const append = (chunk: string, stream: "stdout" | "stderr") => {
      const prefix = stream === "stderr" ? "[stderr] " : "";
      view.append(prefix + chunk);
    };

    try {
      const { code } = await this.runner.run(bin, args, { cwd, env, onChunk: append });
      if (code === 0) {
        new Notice("ics: finished (0)");
      } else {
        new Notice(`ics: exited with code ${code ?? "?"}`);
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      view.append(`\n[plugin error] ${msg}\n`);
      new Notice(`ics spawn failed: ${msg}`);
    }
  }

  async diffActiveFile(): Promise<void> {
    const f = this.app.workspace.getActiveFile();
    if (!f) {
      new Notice("No active file");
      return;
    }
    await this.runIcs(["diff", f.path]);
  }

  async promptCommit(): Promise<void> {
    const message = await new Promise<string | null>((resolve) => {
      const m = new CommitModal(this.app, resolve);
      m.open();
    });
    if (!message) {
      new Notice("Commit cancelled");
      return;
    }
    await this.runIcs(["commit", "-m", message]);
  }
}

class IcsSettingTab extends PluginSettingTab {
  plugin: IcsPlugin;

  constructor(app: App, plugin: IcsPlugin) {
    super(app, plugin);
    this.plugin = plugin;
  }

  display(): void {
    const { containerEl } = this;
    containerEl.empty();
    containerEl.createEl("h2", { text: "ICS" });

    new Setting(containerEl)
      .setName("ics binary")
      .setDesc("Path or command name (e.g. ics, /usr/bin/ics, ~/.cargo/bin/ics)")
      .addText((t) =>
        t
          .setPlaceholder("ics")
          .setValue(this.plugin.settings.icsBinaryPath)
          .onChange(async (v) => {
            this.plugin.settings.icsBinaryPath = v;
            await this.plugin.saveSettings();
          })
      );

    new Setting(containerEl)
      .setName("Stratum base URL (optional)")
      .setDesc("Sets STRATUM_BASE_URL for the child process when non-empty.")
      .addText((t) =>
        t
          .setPlaceholder("https://…")
          .setValue(this.plugin.settings.stratumBaseUrl)
          .onChange(async (v) => {
            this.plugin.settings.stratumBaseUrl = v;
            await this.plugin.saveSettings();
          })
      );
  }
}
```

- [ ] **Step 2: Build**

```bash
npm run build
```

Expected: no TypeScript/esbuild errors; `main.js` updated.

- [ ] **Step 3: Commit**

```bash
git add src/main.ts main.js
git commit -m "feat: wire palette commands, settings, and ICS output view"
```

---

### Task 7: README and manual QA

**Files:**
- Create or replace: `README.md`

- [ ] **Step 1: `README.md` sections**

1. **Prerequisites:** `ics` installed (`cargo install --path …` or release binary); vault is a git-less folder with `ics init` already run in the vault root.  
2. **Development:** `npm install`, `npm run build`, copy `main.js`, `manifest.json`, and `styles.css` into `<Vault>/.obsidian/plugins/ics-obsidian/`, reload Obsidian, enable plugin.  
3. **Flatpak Obsidian:** set **ics binary** to full path; vault must live under paths Flatpak exposes (typically home).  
4. **Manual QA checklist:**  
   - Settings save/restore binary path.  
   - **ICS: Status** shows CLI output in sidebar.  
   - **ICS: Commit…** with message creates commit (verify with terminal `ics log`).  
   - **ICS: Log** prints history.  
   - **ICS: Diff (vault)** and **Diff (active file)** run without throw.  
   - Wrong binary path → Notice + error text in panel.

- [ ] **Step 2: Commit**

```bash
git add README.md && git commit -m "docs: add readme and manual QA for P0"
```

---

### Plan self-review

| Design / spec requirement | Task covering it |
|---------------------------|------------------|
| Sibling repo | Task 1 |
| Spawn `ics`, `cwd` = vault | `vaultBasePath`, `IcsRunner` Tasks 4, 6 |
| `STRATUM_BASE_URL` | `childEnv` Task 6 |
| Serialized runs | `IcsRunner.queue` Task 4 |
| Output leaf + ribbon + palette | Tasks 5–6 |
| ENOENT / spawn errors | `try/catch` in `runIcs` Task 6 |
| Exit code notices | `runIcs` Task 6 |
| Commit modal | `promptCommit` Task 6 |
| UX §11 first-run / daily | README + commands Task 7 |

**Gaps addressed in follow-up (not P0):** P1 debounced status / status bar; P2+ commands when CLI stabilizes argument names. **`CommitModal`** uses `onClose` to resolve exactly once (including overlay dismiss → `null`).

---

**Plan complete and saved to** `docs/superpowers/plans/2026-05-05-obsidian-ics-plugin-p0.md`.

**Execution options:**

1. **Subagent-driven (recommended)** — one subagent per task, review between tasks.  
2. **Inline execution** — run tasks in order in one workspace (sibling folder can live next to `ics-cli`).

Which approach do you want for implementing the sibling repo?
