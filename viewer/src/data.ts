import type { BenchmarkDataMap, FuncMetadata, BenchJsonReport } from "./types";

export function loadBenchmarkData(): BenchmarkDataMap {
  // Viteの機能で target ディレクトリ以下のJSONを同期的に一括取得
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
