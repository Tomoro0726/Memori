/**
 * Data loading and validation utilities
 *
 * ベンチマークデータをmanifest経由でローカルJSONファイルから読み込む
 */

import type { BenchJsonReport, BenchmarkDataMap, FuncMetadata } from "../types";

declare global {
  interface Window {
    __TENBIN_DATA__?: BenchmarkDataMap;
  }
}

interface ReportManifest {
  functions: ReportManifestEntry[];
}

interface ReportManifestEntry {
  name: string;
  mainJsonPath: string;
  maxHistoryNumber: number;
}

type JsonMap = Record<string, unknown>;

/**
 * ベンチマークデータの形式が正しいかどうかをチェック
 * @param data - チェック対象のデータ
 * @returns データが有効な形式であれば true
 */
function validateBenchmarkData(data: BenchmarkDataMap): boolean {
  for (const [, funcData] of Object.entries(data)) {
    if (funcData && typeof funcData === "object" && "meta" in funcData && "history" in funcData) {
      return true;
    }
  }
  return false;
}

function loadInjectedData(): BenchmarkDataMap | null {
  const injected = window.__TENBIN_DATA__;
  if (!injected || typeof injected !== "object") {
    return null;
  }

  const data = injected as BenchmarkDataMap;
  if (!validateBenchmarkData(data)) {
    return null;
  }

  return data;
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
  const resourcePath = `./${encodedPath}`;

  try {
    const response = await fetch(resourcePath, { cache: "no-store" });
    if (!response.ok) {
      console.warn(`Failed to fetch: ${relativePath} (${response.status})`);
    } else {
      return (await response.json()) as T;
    }
  } catch (err) {
    console.warn(`Fetch failed: ${relativePath}`, err);
  }

  // Some environments block fetch for file:// URLs. Try XHR as a fallback.
  try {
    return await new Promise<T | null>((resolve) => {
      const xhr = new XMLHttpRequest();
      xhr.open("GET", resourcePath, true);
      xhr.responseType = "text";

      xhr.onload = () => {
        if (xhr.status !== 0 && (xhr.status < 200 || xhr.status >= 300)) {
          console.warn(`Failed to load via XHR: ${relativePath} (${xhr.status})`);
          resolve(null);
          return;
        }

        try {
          resolve(JSON.parse(xhr.responseText) as T);
        } catch (parseErr) {
          console.warn(`Failed to parse JSON via XHR: ${relativePath}`, parseErr);
          resolve(null);
        }
      };

      xhr.onerror = () => {
        console.warn(`XHR failed: ${relativePath}`);
        resolve(null);
      };

      xhr.send();
    });
  } catch (err) {
    console.warn(`XHR fallback failed: ${relativePath}`, err);
  }

  return null;
}

function isValidManifestEntry(entry: unknown): entry is ReportManifestEntry {
  if (!entry || typeof entry !== "object") {
    return false;
  }

  const candidate = entry as Partial<ReportManifestEntry>;
  return (
    typeof candidate.name === "string" &&
    typeof candidate.mainJsonPath === "string" &&
    typeof candidate.maxHistoryNumber === "number"
  );
}

async function loadFunctionDataFromManifestEntry(
  entry: ReportManifestEntry
): Promise<[string, BenchmarkDataMap[string]] | null> {
  const meta = await fetchJson<FuncMetadata>(entry.mainJsonPath);
  const history = [] as BenchmarkDataMap[string]["history"];

  const maxHistoryNumber = Number(entry.maxHistoryNumber) || 0;
  for (let num = maxHistoryNumber; num >= 1; num--) {
    const historyPath = `${entry.name}/${String(num).padStart(3, "0")}.json`;
    const historyData = await fetchJson<Record<string, BenchJsonReport>>(historyPath);
    if (!historyData) {
      continue;
    }

    const fileName = historyPath.split("/").pop() || historyPath;
    history.push({ fileName, data: historyData });
  }

  history.sort((a, b) => b.fileName.localeCompare(a.fileName));

  if (!meta && history.length === 0) {
    return null;
  }

  return [
    entry.name,
    {
      meta: meta ?? null,
      history,
    },
  ];
}

/**
 * 本番環境: manifestを起点にフォルダ内JSONを動的に読み込む
 * @returns パースされたベンチマークデータ、またはエラー時は空オブジェクト
 */
async function loadProductionData(): Promise<BenchmarkDataMap> {
  const injected = loadInjectedData();
  if (injected) {
    return injected;
  }

  const manifest = await fetchJson<ReportManifest>("report-manifest.json");
  if (!manifest || !Array.isArray(manifest.functions)) {
    throw new Error(
      "Failed to load report-manifest.json. If you opened report.html directly, try serving target/tenbin over a local HTTP server."
    );
  }

  const validEntries = manifest.functions.filter(isValidManifestEntry);
  const loadedEntries = await Promise.all(
    validEntries.map((entry) => loadFunctionDataFromManifestEntry(entry))
  );
  const parsedData: BenchmarkDataMap = Object.fromEntries(
    loadedEntries.filter((entry): entry is [string, BenchmarkDataMap[string]] => entry !== null)
  );

  if (!validateBenchmarkData(parsedData) && Object.keys(parsedData).length > 0) {
    console.warn("Loaded benchmark data format may be incomplete.");
  }

  return parsedData;
}

function normalizeBrowserRelativePath(path: string): string {
  return path.replace(/\\/g, "/").replace(/^\.\//, "");
}

function isHistoryJsonFile(fileName: string): boolean {
  return /^\d+\.json$/i.test(fileName) || /^\d+_.*\.json$/i.test(fileName);
}

function ensureFunctionData(
  parsedData: BenchmarkDataMap,
  funcName: string
): BenchmarkDataMap[string] {
  if (!parsedData[funcName]) {
    parsedData[funcName] = { meta: null, history: [] };
  }

  return parsedData[funcName];
}

function toRelativePath(file: File): string {
  const webkitPath = "webkitRelativePath" in file ? file.webkitRelativePath : "";
  return normalizeBrowserRelativePath(webkitPath || file.name);
}

function parseJsonDescriptor(
  file: File
): { relPath: string; funcName: string; fileName: string } | null {
  if (!file.name.endsWith(".json")) {
    return null;
  }

  const relPath = toRelativePath(file);
  const parts = relPath.split("/");
  if (parts.length < 2) {
    return null;
  }

  const fileName = parts[parts.length - 1];
  const funcName = parts[parts.length - 2];
  if (!fileName || !funcName) {
    return null;
  }

  return { relPath, funcName, fileName };
}

async function assignLocalJsonToData(
  parsedData: BenchmarkDataMap,
  file: File,
  descriptor: { relPath: string; funcName: string; fileName: string }
): Promise<void> {
  const target = ensureFunctionData(parsedData, descriptor.funcName);
  const content = JSON.parse(await file.text()) as JsonMap;

  if (descriptor.fileName === "main.json") {
    target.meta = content as unknown as FuncMetadata;
    return;
  }

  if (descriptor.fileName === "report-manifest.json") {
    return;
  }

  if (isHistoryJsonFile(descriptor.fileName)) {
    target.history.push({
      fileName: descriptor.fileName,
      data: content as Record<string, BenchJsonReport>,
    });
  }
}

/**
 * file input (webkitdirectory) で選択したJSON群からデータを組み立てる
 */
export async function loadBenchmarkDataFromFileList(
  fileList: FileList | File[]
): Promise<BenchmarkDataMap> {
  const files = Array.from(fileList);
  const parsedData: BenchmarkDataMap = {};

  for (const file of files) {
    const descriptor = parseJsonDescriptor(file);
    if (!descriptor) {
      continue;
    }

    try {
      await assignLocalJsonToData(parsedData, file, descriptor);
    } catch (err) {
      console.warn(`Failed to parse local JSON: ${descriptor.relPath}`, err);
    }
  }

  for (const funcName in parsedData) {
    parsedData[funcName].history.sort((a, b) => b.fileName.localeCompare(a.fileName));
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
    const parentDir = parts.pop();

    if (!funcName || !fileName) continue;
    if (parentDir !== "tenbin") continue;

    if (!parsedData[funcName]) {
      parsedData[funcName] = { meta: null, history: [] };
    }

    if (fileName === "main.json") {
      parsedData[funcName].meta = content as FuncMetadata;
    } else if (fileName !== "report-manifest.json") {
      parsedData[funcName].history.push({
        fileName,
        data: content as Record<string, BenchJsonReport>,
      });
    }
  }

  // UI表示用に、ファイル名（001_, 002_...）の降順（最新が先頭）でソート
  for (const funcName in parsedData) {
    parsedData[funcName].history.sort((a, b) => b.fileName.localeCompare(a.fileName));
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
