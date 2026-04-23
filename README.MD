![Cyberhack Preview](./images/preview.png)

````markdown
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
````

## 🚀 Usage (React / Next.js)

The library provides a ready-to-use React component. **Important:** Don't forget to import the compiled CSS file for the neon styling to work properly.

```tsx
import React from "react";
import { Cyberhack, CyberhackResult } from "@nineap/cyberhack";
import "@nineap/cyberhack/cyberhack.css"; // Required for styling

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

## ⚙️ Component Props

| Prop          | Type                                | Default      | Description                                                            |
| :------------ | :---------------------------------- | :----------- | :--------------------------------------------------------------------- |
| `redirectUrl` | `string`                            | **Required** | The URL to redirect the user to upon completion (passes `?coins=...`). |
| `baseValue`   | `number`                            | **Required** | Base reward value. Multiplied by daemon level (1x, 2x, 3x).            |
| `timeLimit`   | `number`                            | **Required** | Time limit in seconds. The timer starts on the first cell click.       |
| `locale`      | `"ru" \| "en"`                      | `"ru"`       | UI Language.                                                           |
| `theme`       | `CyberhackTheme`                    | `undefined`  | Optional custom color overrides (see below).                           |
| `onComplete`  | `(result: CyberhackResult) => void` | `undefined`  | Callback fired when the game ends (win or lose).                       |

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

## 🛠 Local Development & Building

If you want to clone this repository and build the module yourself, you will need **Rust**, `wasm-pack`, and **Node.js** installed.

1. **Clone the repo and navigate to it:**

   ```bash
   git clone [https://github.com/nineap/cyberhack.git](https://github.com/nineap/cyberhack.git)
   cd cyberhack
   ```

2. **Install dependencies:**

   ```bash
   npm install
   ```

3. **Build the WASM binary and CSS:**
   This command compiles the Rust code to WASM, generates the Tailwind CSS output, and copies the TypeScript wrappers into the `pkg/` directory.
   ```bash
   npm run build
   ```
