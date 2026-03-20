import { useState, useMemo } from "react";
import { loadBenchmarkData } from "./data";
import type { MetricKey, LineInfo } from "./types";
import styles from "./App.module.css";
import { BenchmarkChart } from "./components/BenchmarkChart/BenchmarkChart";

const parsedData = loadBenchmarkData();

const METRICS: { key: MetricKey; label: string }[] = [
  { key: "cycles", label: "CPU Cycles" },
  { key: "timeNs", label: "Time (ns)" },
  { key: "allocCount", label: "Allocations (Count)" },
  { key: "allocBytes", label: "Allocated Memory (Bytes)" },
];

export default function App() {
  const functions = Object.keys(parsedData);

  const [selectedFunc, setSelectedFunc] = useState<string>(functions[0] || "");
  const [selectedPattern, setSelectedPattern] = useState<string>("");
  const [selectedMetric, setSelectedMetric] = useState<MetricKey>("timeNs");
  const [historyCount, setHistoryCount] = useState<number>(1);

  const currentMeta = parsedData[selectedFunc]?.meta;
  const currentPatternMeta = currentMeta?.patterns.find(
    (p) => p.name === selectedPattern,
  );
  const isInstant = currentPatternMeta?.patternType === "instant";

  useMemo(() => {
    if (selectedFunc && parsedData[selectedFunc]?.meta) {
      const patterns = parsedData[selectedFunc].meta!.patterns;
      if (patterns.length > 0) {
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
    let finalData: any[] = [];
    const lineInfos: LineInfo[] = [];
    const addedLineKeys = new Set<string>();
    let currentPatternDesc = "";

    if (isInstant) {
      // ＝＝＝ Instantパターンの場合：CodSpeed風トレンドグラフ ＝＝＝
      const targetRuns = [...funcData.history].reverse();
      const trendMap = new Map<number, any>();

      targetRuns.forEach((run, index) => {
        const runNum = run.fileName.split("_")[0];
        const isLatest = index === targetRuns.length - 1;
        const runLabel = isLatest ? "Latest" : `Run-${runNum}`;

        const patternData = run.data[selectedPattern];
        if (!patternData) return;
        if (isLatest) currentPatternDesc = patternData.description || "";

        const dataPoint: any = { run: runLabel };

        Object.entries(patternData.results).forEach(([algoName, entries]) => {
          if (!addedLineKeys.has(algoName)) {
            addedLineKeys.add(algoName);
            lineInfos.push({
              key: algoName,
              algoName,
              runLabel: "",
              runIndex: 0,
            });
          }

          if (entries.length > 0) {
            const metricValue = entries[0].measurement[selectedMetric];
            if (metricValue !== null && metricValue !== undefined) {
              dataPoint[algoName] = metricValue;
            }
          }
        });

        if (Object.keys(dataPoint).length > 1) {
          trendMap.set(index, dataPoint);
        }
      });

      finalData = Array.from(trendMap.values());

      return {
        chartData: finalData,
        lines: lineInfos,
        chartTitle: `${selectedPattern} - Trend History`,
        chartDesc: currentPatternDesc || "CodSpeed style performance history.",
        yAxisLabel:
          METRICS.find((m) => m.key === selectedMetric)?.label ||
          selectedMetric,
        xAxisKey: "run",
        xAxisLabel: "History (Runs)",
      };
    } else {
      // ＝＝＝ Scalingパターンの場合：Nに依存するグラフ ＝＝＝
      const targetRuns = funcData.history.slice(0, historyCount);
      const mergedMap = new Map<number, Record<string, number>>();

      targetRuns.forEach((run, index) => {
        const runLabel =
          index === 0 ? "Latest" : `Run-${run.fileName.split("_")[0]}`;
        const patternData = run.data[selectedPattern];

        if (!patternData) return;
        if (index === 0) currentPatternDesc = patternData.description || "";

        Object.entries(patternData.results).forEach(([algoName, entries]) => {
          const lineKey =
            historyCount === 1 ? algoName : `${algoName} (${runLabel})`;

          if (!addedLineKeys.has(lineKey)) {
            addedLineKeys.add(lineKey);
            lineInfos.push({
              key: lineKey,
              algoName,
              runLabel,
              runIndex: index,
            });
          }

          entries.forEach((entry) => {
            const inputSize = entry.input;
            const metricValue = entry.measurement[selectedMetric];

            if (metricValue === null || metricValue === undefined) return;

            if (!mergedMap.has(inputSize)) {
              mergedMap.set(inputSize, { input: inputSize });
            }
            mergedMap.get(inputSize)![lineKey] = metricValue;
          });
        });
      });

      finalData = Array.from(mergedMap.values()).sort(
        (a, b) => a.input - b.input,
      );

      return {
        chartData: finalData,
        lines: lineInfos,
        chartTitle: `${selectedPattern} - ${METRICS.find((m) => m.key === selectedMetric)?.label}`,
        chartDesc: currentPatternDesc,
        yAxisLabel:
          METRICS.find((m) => m.key === selectedMetric)?.label ||
          selectedMetric,
        xAxisKey: "input",
        xAxisLabel: "N (Input Size)",
      };
    }
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
          <div className={styles.spacer}></div>

          <div className={styles.filters}>
            {/* Target Function */}
            <div className={styles.filterGroup}>
              <label className={styles.filterLabel}>Target Function</label>
              <select
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
              <label className={styles.filterLabel}>Benchmark Pattern</label>
              <select
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
              <label className={styles.filterLabel}>Metric to Graph</label>
              <select
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
                <label className={styles.filterLabel}>Compare History</label>
                <select
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
