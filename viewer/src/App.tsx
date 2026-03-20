import { useMemo, useState } from "react";
import styles from "./App.module.css";
import { BenchmarkChart } from "./components/BenchmarkChart/BenchmarkChart";
import { loadBenchmarkData } from "./data";
import type { BenchJsonEntry, LineInfo, MetricKey } from "./types";

const parsedData = loadBenchmarkData();

const METRICS: { key: MetricKey; label: string }[] = [
  { key: "cycles", label: "CPU Cycles" },
  { key: "timeNs", label: "Time (ns)" },
  { key: "allocCount", label: "Allocations (Count)" },
  { key: "allocBytes", label: "Allocated Memory (Bytes)" },
];

type ChartDataPoint = Record<string, unknown>;

interface ChartState {
  chartData: ChartDataPoint[];
  lines: LineInfo[];
  chartTitle: string;
  chartDesc: string;
  yAxisLabel: string;
  xAxisKey: string;
  xAxisLabel: string;
}

function addLineInfoIfNew(
  lineInfos: LineInfo[],
  addedLineKeys: Set<string>,
  key: string,
  algoName: string,
  runLabel: string,
  runIndex: number,
): void {
  if (!addedLineKeys.has(key)) {
    addedLineKeys.add(key);
    lineInfos.push({ key, algoName, runLabel, runIndex });
  }
}

function addInstantMetricToPoint(
  algoName: string,
  entries: BenchJsonEntry[],
  selectedMetric: MetricKey,
  dataPoint: ChartDataPoint,
): void {
  if (entries.length === 0) return;
  const value = entries[0].measurement[selectedMetric];
  if (value !== null && value !== undefined) {
    dataPoint[algoName] = value;
  }
}

function processInstantPatternRun(
  run: (typeof parsedData)[string]["history"][number],
  index: number,
  targetRunsLength: number,
  selectedPattern: string,
  selectedMetric: MetricKey,
  dataMap: Map<number, ChartDataPoint>,
  lineInfos: LineInfo[],
  addedLineKeys: Set<string>,
): string {
  const runNum = run.fileName.split("_")[0];
  const isLatest = index === targetRunsLength - 1;
  const runLabel = isLatest ? "Latest" : `Run-${runNum}`;

  const patternData = run.data[selectedPattern];
  if (!patternData) return "";

  const dataPoint: ChartDataPoint = { run: runLabel };

  for (const [algoName, entries] of Object.entries(patternData.results)) {
    addLineInfoIfNew(lineInfos, addedLineKeys, algoName, algoName, "", 0);
    addInstantMetricToPoint(algoName, entries, selectedMetric, dataPoint);
  }

  if (Object.keys(dataPoint).length > 1) {
    dataMap.set(index, dataPoint);
  }

  return isLatest ? patternData.description || "" : "";
}

function processInstantPattern(
  funcData: (typeof parsedData)[string],
  selectedPattern: string,
  selectedMetric: MetricKey,
): ChartState {
  const targetRuns = [...funcData.history].reverse();
  const trendMap = new Map<number, ChartDataPoint>();
  const lineInfos: LineInfo[] = [];
  const addedLineKeys = new Set<string>();
  let currentPatternDesc = "";

  for (const [index, run] of targetRuns.entries()) {
    const desc = processInstantPatternRun(
      run,
      index,
      targetRuns.length,
      selectedPattern,
      selectedMetric,
      trendMap,
      lineInfos,
      addedLineKeys,
    );
    if (desc) currentPatternDesc = desc;
  }

  const finalData = Array.from(trendMap.values());

  return {
    chartData: finalData,
    lines: lineInfos,
    chartTitle: `${selectedPattern} - Trend History`,
    chartDesc: currentPatternDesc || "CodSpeed style performance history.",
    yAxisLabel:
      METRICS.find((m) => m.key === selectedMetric)?.label || selectedMetric,
    xAxisKey: "run",
    xAxisLabel: "History (Runs)",
  };
}

function addScalingMetricsToMap(
  entries: BenchJsonEntry[],
  selectedMetric: MetricKey,
  lineKey: string,
  mergedMap: Map<number, Record<string, number>>,
): void {
  for (const entry of entries) {
    const value = entry.measurement[selectedMetric];
    if (value === null || value === undefined) continue;

    if (!mergedMap.has(entry.input)) {
      mergedMap.set(entry.input, { input: entry.input });
    }
    const existing = mergedMap.get(entry.input);
    if (existing) {
      existing[lineKey] = value;
    }
  }
}

function processScalingPatternRun(
  run: (typeof parsedData)[string]["history"][number],
  index: number,
  selectedPattern: string,
  selectedMetric: MetricKey,
  historyCount: number,
  mergedMap: Map<number, Record<string, number>>,
  lineInfos: LineInfo[],
  addedLineKeys: Set<string>,
): string {
  const runLabel = index === 0 ? "Latest" : `Run-${run.fileName.split("_")[0]}`;
  const patternData = run.data[selectedPattern];

  if (!patternData) return "";

  for (const [algoName, entries] of Object.entries(patternData.results)) {
    const lineKey = historyCount === 1 ? algoName : `${algoName} (${runLabel})`;
    addLineInfoIfNew(
      lineInfos,
      addedLineKeys,
      lineKey,
      algoName,
      runLabel,
      index,
    );
    addScalingMetricsToMap(entries, selectedMetric, lineKey, mergedMap);
  }

  return index === 0 ? patternData.description || "" : "";
}

function processScalingPattern(
  funcData: (typeof parsedData)[string],
  selectedPattern: string,
  selectedMetric: MetricKey,
  historyCount: number,
): ChartState {
  const targetRuns = funcData.history.slice(0, historyCount);
  const mergedMap = new Map<number, Record<string, number>>();
  const lineInfos: LineInfo[] = [];
  const addedLineKeys = new Set<string>();
  let currentPatternDesc = "";

  for (const [index, run] of targetRuns.entries()) {
    const desc = processScalingPatternRun(
      run,
      index,
      selectedPattern,
      selectedMetric,
      historyCount,
      mergedMap,
      lineInfos,
      addedLineKeys,
    );
    if (desc) currentPatternDesc = desc;
  }

  const finalData = Array.from(mergedMap.values()).sort(
    (a, b) => (a.input as number) - (b.input as number),
  );

  return {
    chartData: finalData,
    lines: lineInfos,
    chartTitle: `${selectedPattern} - ${METRICS.find((m) => m.key === selectedMetric)?.label}`,
    chartDesc: currentPatternDesc,
    yAxisLabel:
      METRICS.find((m) => m.key === selectedMetric)?.label || selectedMetric,
    xAxisKey: "input",
    xAxisLabel: "N (Input Size)",
  };
}

export default function App() {
  const functions = Object.keys(parsedData);

  const [selectedFunc, setSelectedFunc] = useState<string>(functions[0] || "");
  const [selectedPattern, setSelectedPattern] = useState<string>("");
  const [selectedMetric, setSelectedMetric] = useState<MetricKey>("cycles");
  const [historyCount, setHistoryCount] = useState<number>(1);

  const currentMeta = parsedData[selectedFunc]?.meta;
  const currentPatternMeta = currentMeta?.patterns.find(
    (p) => p.name === selectedPattern,
  );
  const isInstant = currentPatternMeta?.patternType === "instant";

  useMemo(() => {
    if (selectedFunc && parsedData[selectedFunc]?.meta) {
      const patterns = parsedData[selectedFunc].meta?.patterns;
      if (patterns && patterns.length > 0) {
        setSelectedPattern(patterns[0].name);
      }
    }
  }, [selectedFunc]);

  const {
    chartData,
    lines,
    chartTitle,
    chartDesc,
    yAxisLabel,
    xAxisKey,
    xAxisLabel,
  } = useMemo(() => {
    if (!selectedFunc || !selectedPattern) {
      return {
        chartData: [],
        lines: [],
        chartTitle: "",
        chartDesc: "",
        yAxisLabel: "",
        xAxisKey: "",
        xAxisLabel: "",
      };
    }

    const funcData = parsedData[selectedFunc];
    if (isInstant) {
      return processInstantPattern(funcData, selectedPattern, selectedMetric);
    }
    return processScalingPattern(
      funcData,
      selectedPattern,
      selectedMetric,
      historyCount,
    );
  }, [selectedFunc, selectedPattern, selectedMetric, historyCount, isInstant]);

  if (functions.length === 0) {
    return (
      <div className={styles.page}>
        No benchmark data found. Run Tenbin tests first!
      </div>
    );
  }

  return (
    <div className={styles.page}>
      <div className={styles.layout}>
        <header className={styles.header}>
          <div>
            <h1 className={styles.pageTitle}>Tenbin Viewer</h1>
            <p className={styles.pageSubtitle}>Rust Performance Benchmarks</p>
          </div>
          <div className={styles.spacer} />

          <div className={styles.filters}>
            {/* Target Function */}
            <div className={styles.filterGroup}>
              <label className={styles.filterLabel} htmlFor="target-func">
                Target Function
              </label>
              <select
                id="target-func"
                value={selectedFunc}
                onChange={(e) => setSelectedFunc(e.target.value)}
                className={styles.selectBox}
              >
                {functions.map((f) => (
                  <option key={f} value={f}>
                    {f}
                  </option>
                ))}
              </select>
            </div>

            {/* Benchmark Pattern */}
            <div className={styles.filterGroup}>
              <label className={styles.filterLabel} htmlFor="bench-pattern">
                Benchmark Pattern
              </label>
              <select
                id="bench-pattern"
                value={selectedPattern}
                onChange={(e) => setSelectedPattern(e.target.value)}
                className={styles.selectBox}
              >
                {currentMeta?.patterns.map((p) => (
                  <option key={p.name} value={p.name}>
                    {p.name}
                  </option>
                ))}
              </select>
            </div>

            {/* Metric to Graph (常に表示！) */}
            <div className={styles.filterGroup}>
              <label className={styles.filterLabel} htmlFor="metric-graph">
                Metric to Graph
              </label>
              <select
                id="metric-graph"
                value={selectedMetric}
                onChange={(e) => setSelectedMetric(e.target.value as MetricKey)}
                className={styles.selectBox}
              >
                {METRICS.map((m) => (
                  <option key={m.key} value={m.key}>
                    {m.label}
                  </option>
                ))}
              </select>
            </div>

            {/* Compare History (Scalingのときだけ表示する) */}
            {!isInstant && (
              <div className={styles.filterGroup}>
                <label className={styles.filterLabel} htmlFor="compare-history">
                  Compare History
                </label>
                <select
                  id="compare-history"
                  value={historyCount}
                  onChange={(e) => setHistoryCount(Number(e.target.value))}
                  className={styles.selectBox}
                >
                  <option value={1}>Latest only</option>
                  <option value={2}>Last 2 runs</option>
                  <option value={3}>Last 3 runs</option>
                  <option value={5}>Last 5 runs</option>
                </select>
              </div>
            )}
          </div>
        </header>

        <main>
          {chartData.length > 0 ? (
            <BenchmarkChart
              title={chartTitle}
              description={chartDesc}
              data={chartData}
              lines={lines}
              yAxisLabel={yAxisLabel}
              xAxisKey={xAxisKey}
              xAxisLabel={xAxisLabel}
            />
          ) : (
            <div className="text-gray-500 text-center py-10">
              Select a pattern to view data.
            </div>
          )}
        </main>
      </div>
    </div>
  );
}
