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

  useMemo(() => {
    if (selectedFunc && parsedData[selectedFunc]?.meta) {
      const patterns = parsedData[selectedFunc].meta!.patterns;
      if (patterns.length > 0) {
        setSelectedPattern(patterns[0].name);
      }
    }
  }, [selectedFunc]);

  const { chartData, lines, chartTitle, chartDesc, yAxisLabel } =
    useMemo(() => {
      if (!selectedFunc || !selectedPattern) {
        return {
          chartData: [],
          lines: [],
          chartTitle: "",
          chartDesc: "",
          yAxisLabel: "",
        };
      }

      const funcData = parsedData[selectedFunc];
      const targetRuns = funcData.history.slice(0, historyCount);

      const mergedMap = new Map<number, Record<string, number>>();
      const lineInfos: LineInfo[] = [];
      const addedLineKeys = new Set<string>();
      let currentPatternDesc = "";

      targetRuns.forEach((run, index) => {
        const runLabel =
          index === 0 ? "Latest" : `Run-${run.fileName.split("_")[0]}`;
        const patternData = run.data[selectedPattern];

        if (!patternData) return;
        if (index === 0) currentPatternDesc = patternData.description || "";

        Object.entries(patternData.results).forEach(([algoName, entries]) => {
          const lineKey =
            historyCount === 1 ? algoName : `${algoName} (${runLabel})`;

          // LineInfoを生成してリストに追加（重複登録防止）
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

      const finalData = Array.from(mergedMap.values()).sort(
        (a, b) => a.input - b.input,
      );
      const selectedMetricLabel =
        METRICS.find((m) => m.key === selectedMetric)?.label || selectedMetric;

      return {
        chartData: finalData,
        lines: lineInfos,
        chartTitle: `${selectedPattern} - ${selectedMetricLabel}`,
        chartDesc: currentPatternDesc,
        yAxisLabel: selectedMetricLabel,
      };
    }, [selectedFunc, selectedPattern, selectedMetric, historyCount]);

  if (functions.length === 0) {
    return (
      <div className={styles.page}>
        No benchmark data found. Run Tenbin tests first!
      </div>
    );
  }

  const currentMeta = parsedData[selectedFunc]?.meta;

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
