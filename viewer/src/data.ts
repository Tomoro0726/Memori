import type { BenchmarkDataMap, FuncMetadata, BenchJsonReport } from "./types";

// 本番環境でRustから注入されるグローバル変数の型定義
declare global {
  interface Window {
    __TENBIN_DATA__?: BenchmarkDataMap | null;
  }
}

export function loadBenchmarkData(): BenchmarkDataMap {
  // ＝＝＝ 本番環境（ビルド後）の処理 ＝＝＝
  // Rustの `run_and_save` によって HTML の <head> に注入されたデータを読み取る
  if (import.meta.env.PROD) {
    if (window.__TENBIN_DATA__) {
      return window.__TENBIN_DATA__;
    }
    console.warn("No benchmark data found in window.__TENBIN_DATA__.");
    return {};
  }

  // ＝＝＝ 開発環境（npm run dev）の処理 ＝＝＝
  // 開発中はローカルのファイルを監視して動的に読み込む
  const rawFiles = import.meta.glob("../../target/tenbin/**/*.json", {
    eager: true,
    import: "default",
  });

  const parsedData: BenchmarkDataMap = {};

  for (const [path, content] of Object.entries(rawFiles)) {
    const parts = path.split("/");
    const fileName = parts.pop();
    const funcName = parts.pop();

    if (!funcName || !fileName) continue;

    if (!parsedData[funcName]) {
      parsedData[funcName] = { meta: null, history: [] };
    }

    if (fileName === "main.json") {
      parsedData[funcName].meta = content as FuncMetadata;
    } else {
      parsedData[funcName].history.push({
        fileName,
        data: content as Record<string, BenchJsonReport>,
      });
    }
  }

  // UI表示用に、ファイル名（001_, 002_...）の降順（最新が先頭）でソート
  for (const funcName in parsedData) {
    parsedData[funcName].history.sort((a, b) =>
      b.fileName.localeCompare(a.fileName),
    );
  }

  return parsedData;
}
