/**
 * Data loading and validation utilities
 *
 * ベンチマークデータをmanifest経由でローカルJSONファイルから読み込む
 */

import type { BenchJsonReport, BenchmarkDataMap, FuncMetadata } from "../types";

interface ReportManifest {
  functions: ReportManifestEntry[];
}

interface ReportManifestEntry {
  name: string;
  mainJsonPath: string;
  historyJsonPaths: string[];
}

/**
 * ベンチマークデータの形式が正しいかどうかをチェック
 * @param data - チェック対象のデータ
 * @returns データが有効な形式であれば true
 */
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

/**
 * ブラウザのfetchで安全に読めるよう、URLセグメント単位でエンコードする
 */
function encodeRelativePath(path: string): string {
  return path
    .split("/")
    .filter((segment) => segment.length > 0)
    .map((segment) => encodeURIComponent(segment))
    .join("/");
}

/**
 * JSONファイルを読み込む。読み込み失敗時は null を返す
 */
async function fetchJson<T>(relativePath: string): Promise<T | null> {
  const encodedPath = encodeRelativePath(relativePath);

  try {
    const response = await fetch(`./${encodedPath}`, { cache: "no-store" });
    if (!response.ok) {
      console.warn(`Failed to fetch: ${relativePath} (${response.status})`);
      return null;
    }
    return (await response.json()) as T;
  } catch (err) {
    console.warn(`Failed to fetch: ${relativePath}`, err);
    return null;
  }
}

/**
 * 本番環境: manifestを起点にフォルダ内JSONを動的に読み込む
 * @returns パースされたベンチマークデータ、またはエラー時は空オブジェクト
 */
async function loadProductionData(): Promise<BenchmarkDataMap> {
  const manifest = await fetchJson<ReportManifest>("report-manifest.json");
  if (!manifest || !Array.isArray(manifest.functions)) {
    console.warn("report-manifest.json is missing or malformed.");
    return {};
  }

  const parsedData: BenchmarkDataMap = {};

  for (const entry of manifest.functions) {
    if (!entry || typeof entry.name !== "string") {
      continue;
    }

    const meta = await fetchJson<FuncMetadata>(entry.mainJsonPath);
    const history = [] as BenchmarkDataMap[string]["history"];

    for (const historyPath of entry.historyJsonPaths || []) {
      const historyData =
        await fetchJson<Record<string, BenchJsonReport>>(historyPath);
      if (!historyData) {
        continue;
      }

      const fileName = historyPath.split("/").pop() || historyPath;
      history.push({ fileName, data: historyData });
    }

    history.sort((a, b) => b.fileName.localeCompare(a.fileName));

    if (meta || history.length > 0) {
      parsedData[entry.name] = {
        meta: meta ?? null,
        history,
      };
    }
  }

  if (
    !validateBenchmarkData(parsedData) &&
    Object.keys(parsedData).length > 0
  ) {
    console.warn("Loaded benchmark data format may be incomplete.");
  }

  return parsedData;
}

/**
 * 開発環境: ローカルファイルシステムからデータを動的に読み込む
 * @returns パースされたベンチマークデータ
 */
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

  // UI表示用に、ファイル名（001_, 002_...）の降順（最新が先頭）でソート
  for (const funcName in parsedData) {
    parsedData[funcName].history.sort((a, b) =>
      b.fileName.localeCompare(a.fileName),
    );
  }

  return parsedData;
}

/**
 * 環境に応じて適切なデータロード方法でベンチマークデータを読み込む
 * @returns パースされたベンチマークデータ
 */
export async function loadBenchmarkData(): Promise<BenchmarkDataMap> {
  if (import.meta.env.PROD) {
    return await loadProductionData();
  }
  return loadDevelopmentData();
}
