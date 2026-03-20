# Tenbin Viewer - React + TypeScript + Vite

## ⚡ Quick Start

```bash
# Install dependencies
bun install

# Development server
bun run dev

# Build for production
bun run build

# Run all quality checks (format + lint + typecheck)
bun run ci
```

## 🔍 Code Quality Check System

このプロジェクトでは **Biome** を使用した包括的なコード品質チェックを導入しています。

### GitHub Push 前のチェック手順

GitHub にコミットを push する前に、以下のコマンドですべてのチェックをクリアしてください：

```bash
bun run ci
```

このコマンドは以下を実行します：

1. **Format** - コードを自動フォーマット + 不要なコード削除

   ```bash
   bun run format
   ```

2. **Check** - リント + 複雑度チェック + 警告検出

   ```bash
   bun run check
   ```

3. **Type Check** - TypeScript 型チェック
   ```bash
   bun run typecheck
   ```

### チェック内容

- **フォーマッティング**: コードスタイルの統一（インデント、セミコロン等）
- **リント**: 一般的なコード問題の検出（未使用変数、セキュリティ等）
- **複雑度分析**: 認知複雑度が高い関数を警告（閾値: 15）
- **型安全**: TypeScript の型エラー検出

### Biome について

詳細は [BIOME_SETUP.md](./BIOME_SETUP.md) を参照してください。

---

# React + TypeScript + Vite

This template provides a minimal setup to get React working in Vite with HMR and some ESLint rules.

Currently, two official plugins are available:

- [@vitejs/plugin-react](https://github.com/vitejs/vite-plugin-react/blob/main/packages/plugin-react) uses [Oxc](https://oxc.rs)
- [@vitejs/plugin-react-swc](https://github.com/vitejs/vite-plugin-react/blob/main/packages/plugin-react-swc) uses [SWC](https://swc.rs/)

## React Compiler

The React Compiler is enabled on this template. See [this documentation](https://react.dev/learn/react-compiler) for more information.

Note: This will impact Vite dev & build performances.

## Expanding the ESLint configuration

If you are developing a production application, we recommend updating the configuration to enable type-aware lint rules:

```js
export default defineConfig([
  globalIgnores(["dist"]),
  {
    files: ["**/*.{ts,tsx}"],
    extends: [
      // Other configs...

      // Remove tseslint.configs.recommended and replace with this
      tseslint.configs.recommendedTypeChecked,
      // Alternatively, use this for stricter rules
      tseslint.configs.strictTypeChecked,
      // Optionally, add this for stylistic rules
      tseslint.configs.stylisticTypeChecked,

      // Other configs...
    ],
    languageOptions: {
      parserOptions: {
        project: ["./tsconfig.node.json", "./tsconfig.app.json"],
        tsconfigRootDir: import.meta.dirname,
      },
      // other options...
    },
  },
]);
```

You can also install [eslint-plugin-react-x](https://github.com/Rel1cx/eslint-react/tree/main/packages/plugins/eslint-plugin-react-x) and [eslint-plugin-react-dom](https://github.com/Rel1cx/eslint-react/tree/main/packages/plugins/eslint-plugin-react-dom) for React-specific lint rules:

```js
// eslint.config.js
import reactX from "eslint-plugin-react-x";
import reactDom from "eslint-plugin-react-dom";

export default defineConfig([
  globalIgnores(["dist"]),
  {
    files: ["**/*.{ts,tsx}"],
    extends: [
      // Other configs...
      // Enable lint rules for React
      reactX.configs["recommended-typescript"],
      // Enable lint rules for React DOM
      reactDom.configs.recommended,
    ],
    languageOptions: {
      parserOptions: {
        project: ["./tsconfig.node.json", "./tsconfig.app.json"],
        tsconfigRootDir: import.meta.dirname,
      },
      // other options...
    },
  },
]);
```
