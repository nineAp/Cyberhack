![Cyberhack Preview](./images/preview.png)

# Cyberhack: Breach Protocol 👾

A high-performance, React-ready WebAssembly port of the iconic "Breach Protocol" hacking minigame from Cyberpunk 2077.

Core logic is written in **Rust (Yew)** and compiled to WebAssembly for lightning-fast execution. The UI is heavily customized using **Tailwind CSS v4** with CSS-based neon glow and flicker animations.

## 📦 Installation

Install the package via npm:

```bash
npm install cyberhack
yarn add cyberhack
pnpm add cyberhack
```

## 🚀 Usage (React / Next.js)

The library provides a ready-to-use React component. **Important:** Don't forget to import the compiled CSS file for the neon styling to work properly.

```tsx
import React from "react";
import { Cyberhack, CyberhackResult } from "cyberhack";
import "cyberhack/cyberhack.css"; // Required for styling

const App = () => {
  const handleComplete = (result: CyberhackResult) => {
    console.log("Hacking finished!");
    console.log("Earned coins:", result.total_coins);
    console.log("Buffer sequence:", result.buffer);
  };

  return (
    <div style={{ width: "100vw", height: "100vh" }}>
      <Cyberhack
        timeLimit={30}
        baseValue={10}
        redirectUrl="/success"
        locale="ru"
        onComplete={handleComplete}
      />
    </div>
  );
};

export default App;
```

The component dynamically imports the WASM module on mount, so it must run client-side (mark the page/component `"use client"` in Next.js App Router, which `Cyberhack.tsx` already does).

## ⚙️ Component Props

| Prop          | Type                                | Default      | Description                                                            |
| :------------ | :----------------------------------- | :----------- | :--------------------------------------------------------------------- |
| `redirectUrl` | `string`                            | **Required** | The URL to redirect the user to upon completion (passes `?coins=...`). An **empty string** means "don't redirect" — the integration handles the result itself via `onComplete`/`postMessage` (used by the server-authoritative SPA flow). |
| `baseValue`   | `number`                            | **Required** | Base reward value. Multiplied by daemon level (1x, 2x, 3x).            |
| `timeLimit`   | `number`                            | **Required** | Time limit in seconds. The timer starts on the first cell click.       |
| `locale`      | `"ru" \| "en"`                      | `"ru"`       | UI Language.                                                           |
| `theme`       | `CyberhackTheme`                    | `undefined`  | Optional custom color overrides (see below).                           |
| `matrix`      | `string[][]`                       | `undefined`  | Server-authoritative board (see below). Must be paired with `targets`. |
| `targets`     | `string[][]`                       | `undefined`  | Server-authoritative daemon targets. Must be paired with `matrix`.     |
| `sessionId`   | `string`                            | `undefined`  | Opaque session id from the backend; echoed back unchanged in `onComplete`'s `result.session_id`. |
| `onComplete`  | `(result: CyberhackResult) => void` | `undefined`  | Callback fired when the game ends (win or lose).                       |

### Server-authoritative mode (`matrix` / `targets` / `sessionId`)

By default the board (5×5 token matrix + 3 daemon targets) is generated **client-side** in Rust/WASM (`generate_solvable_board` in `src/lib.rs`). This is fine for a purely cosmetic minigame, but if `total_coins` ever drives a real payout, a client can just fabricate any `CyberhackResult` via `postMessage` — the WASM module has no way to prove it actually ran the game.

To make the reward trustworthy, the backend can generate the board itself and pass it in:

```tsx
<Cyberhack
  timeLimit={45}
  baseValue={50}
  redirectUrl=""            // empty: this integration handles the result itself, no location.assign
  matrix={serverMatrix}      // 5x5 string[][], tokens must be one of "1C" "55" "BD" "E9" "7A"
  targets={serverTargets}    // 3 target sequences (string[][])
  sessionId={session.id}
  onComplete={(result) => {
    // Send session_id + result.path (the actual sequence of (row, col) clicks)
    // to the backend. The backend must replay `path` against its own copy of
    // matrix/targets and compute the reward itself — do NOT trust
    // result.total_coins, it is only meaningful in the local/offline mode.
    submitHackResult(result.session_id, result.path);
  }}
/>
```

When `matrix`/`targets` are provided, the WASM side skips its own board generation and plays exactly the board it was given. `result.path` (the ordered list of accepted `(row, col)` clicks) is what a server should replay to independently verify which daemons were actually completed — `result.total_coins` in the payload is client-computed and only meant for local display in the debug/offline build.

### Customizing the Theme

You can override the default Cyberpunk colors (Purple and Cyan) by passing a `theme` object. You can use standard CSS colors, Hex, RGB, or OKLCH.

```tsx
<Cyberhack
  // ...other props
  theme={{
    primary: "#ff003c", // Red active state
    secondary: "#00f0ff", // Blue active state
    background: "#0a0a0a", // Deep dark background
    foreground: "#ffffff", // White text
  }}
/>
```

## 🎮 How it works

- The core game (board generation, breach rules, timer, scoring) is implemented once in Rust as a Yew component (`src/lib.rs`) and compiled to WebAssembly.
- `Cyberhack.tsx` is a thin React client component: it lazy-loads the WASM module (`import("cyberhack/wasm")`), mounts it into an internal `<div>`, and bridges results back via `window.postMessage` — there is no direct JS↔Rust callback binding.
- Without `matrix`/`targets` props, the 5×5 board is generated **client-side** as a *solvable* puzzle: a random zig-zag path (alternating row/column moves, exactly like the real buffer rules) is walked across the matrix first, and the three daemon targets are cut out of that same path, so completing them all in one sequence is always possible. With `matrix`/`targets` supplied, this generation step is skipped entirely and the server-provided board is used instead (see "Server-authoritative mode" above).
- Reward for a completed daemon is `baseValue * (daemon index + 1)` — later/longer daemons are worth more. This is only used for the number shown in the end-of-game overlay; a real payout must be computed server-side from `result.path`.

## 🛠 Local Development & Building

If you want to clone this repository and build the module yourself, you will need **Rust**, `wasm-pack`, and **Node.js** installed.

1. **Clone the repo and navigate to it:**

   ```bash
   git clone https://github.com/nineap/cyberhack.git
   cd cyberhack
   ```

2. **Install dependencies:**

   This project uses **pnpm** (`pnpm-lock.yaml` / `pnpm-workspace.yaml` are the source of truth; `npm install` also works but will not respect the pnpm workspace settings).

   ```bash
   pnpm install
   ```

3. **Build the WASM binary and CSS:**
   This command compiles the Rust code to WASM, generates the Tailwind CSS output, and copies the TypeScript wrappers into the `pkg/` directory.
   ```bash
   pnpm run build
   ```

4. **Run the standalone debug page (optional):**
   Uses [Trunk](https://trunkrs.dev/) to serve `index.html`, which auto-mounts a debug instance of the game (see `run_app()` in `src/lib.rs`) with hot CSS reloading. Trunk is not installed by `pnpm install` — install it separately (`cargo install trunk`) if you need this step.
   ```bash
   pnpm run dev
   ```

### Available scripts

| Script                | Description                                                                 |
| :-------------------- | :---------------------------------------------------------------------------- |
| `pnpm run dev`         | Watches Tailwind CSS and serves the debug page via Trunk on port 8080.        |
| `pnpm run build:wasm`  | Compiles `src/lib.rs` to WebAssembly with `wasm-pack` (`--no-default-features`, i.e. the standalone debug mount is compiled out). |
| `pnpm run build:css`   | Builds the Tailwind CSS bundle into `pkg/cyberhack.css`.                      |
| `pnpm run build`       | Full production build: wasm + css + copies TS wrappers (`prep:pkg`) + syncs `pkg/package.json` from `package.publish.json` (`sync:config`). |
| `pnpm run release`     | Bumps the patch version (`npm version patch`) and runs a full build.          |

Note: `pkg/` is entirely generated output (its own `pkg/.gitignore` ignores everything in it) — nothing under `pkg/` is committed to git. `pnpm run build` must be run at least once before importing the package locally or running anything that does `import("cyberhack/wasm")`.

Rust unit tests (board generation, reward math, i18n fallback) live in `src/lib.rs` and run with:

```bash
cargo test
```
