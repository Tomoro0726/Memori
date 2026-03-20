import type { BenchJsonReport, BenchmarkDataMap, FuncMetadata } from "./types";

// 本番環境でRustから注入されるグローバル変数の型定義
declare global {
  interface Window {
    __TENBIN_DATA__?: Record<string, unknown> | null;
  }
}

function validateBenchmarkData(data: BenchmarkDataMap): boolean {
  for (const [, funcData] of Object.entries(data)) {
    if (
      funcData &&
      typeof funcData === "object" &&
      "meta" in funcData &&
      "history" in funcData
    ) {
      return true;
    }
  }
  return false;
}

function loadProductionData(): BenchmarkDataMap {
  try {
    if (window.__TENBIN_DATA__ && typeof window.__TENBIN_DATA__ === "object") {
      const data = window.__TENBIN_DATA__ as BenchmarkDataMap;
      if (validateBenchmarkData(data)) {
        return data;
      }
      console.warn("Injected data format is incorrect.");
    } else {
      console.warn("No benchmark data found in window.__TENBIN_DATA__.");
    }
  } catch (err) {
    console.error("Failed to load injected benchmark data:", err);
  }
  return {};
}

function loadDevelopmentData(): BenchmarkDataMap {
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

  for (const funcName in parsedData) {
    parsedData[funcName].history.sort((a, b) =>
      b.fileName.localeCompare(a.fileName),
    );
  }

  return parsedData;
}

export function loadBenchmarkData(): BenchmarkDataMap {
  if (import.meta.env.PROD) {
    return loadProductionData();
  }
  return loadDevelopmentData();
}
