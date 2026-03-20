import { toPng } from "html-to-image";
import { Download, FileSpreadsheet, Settings } from "lucide-react";
import type React from "react";
import { useCallback, useEffect, useRef, useState } from "react";
import {
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { LineInfo } from "../../types";
import styles from "./BenchmarkChart.module.css";

interface ChartProps {
  title: string;
  description?: string;
  data: Record<string, unknown>[];
  lines: LineInfo[];
  yAxisLabel: string;
  xAxisKey: string;
  xAxisLabel: string;
}

export const BenchmarkChart: React.FC<ChartProps> = ({
  title,
  description,
  data,
  lines,
  yAxisLabel,
  xAxisKey,
  xAxisLabel,
}) => {
  const chartRef = useRef<HTMLDivElement>(null);
  const [height, setHeight] = useState(400);

  const algoNames = Array.from(new Set(lines.map((l) => l.algoName)));
  const [algoColors, setAlgoColors] = useState<Record<string, string>>({});

  useEffect(() => {
    setAlgoColors((prev) => {
      const newColors = { ...prev };
      for (const [i, name] of algoNames.entries()) {
        if (!newColors[name]) {
          newColors[name] = `hsl(${(i * 137.5) % 360}, 70%, 50%)`;
        }
      }
      return newColors;
    });
  }, [algoNames]);

  const handleAlgoColorChange = (algoName: string, color: string) => {
    setAlgoColors((prev) => ({ ...prev, [algoName]: color }));
  };

  const maxRunIndex = Math.max(0, ...lines.map((l) => l.runIndex));

  const getOpacity = (runIndex: number) => {
    if (maxRunIndex === 0) return 1;
    return 1 - (runIndex / maxRunIndex) * 0.8;
  };

  const downloadChart = useCallback(() => {
    if (chartRef.current === null) return;
    toPng(chartRef.current, {
      cacheBust: true,
      pixelRatio: 2,
      backgroundColor: "#ffffff",
    })
      .then((dataUrl) => {
        const link = document.createElement("a");
        link.download = `${title.replace(/\s+/g, "_")}.png`;
        link.href = dataUrl;
        link.click();
      })
      .catch((err) => console.error("PNG保存に失敗しました", err));
  }, [title]);

  const downloadCSV = () => {
    if (data.length === 0) return;
    const headers = [xAxisKey, ...lines.map((l) => l.key)].join(",");
    const rows = data.map(
      (row) =>
        `${String(row[xAxisKey])},${lines.map((line) => row[line.key] ?? "").join(",")}`,
    );
    const csvContent = `data:text/csv;charset=utf-8,${[headers, ...rows].join("\n")}`;
    const encodedUri = encodeURI(csvContent);
    const link = document.createElement("a");
    link.setAttribute("href", encodedUri);
    link.setAttribute("download", `${title.replace(/\s+/g, "_")}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
  };

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <div>
          <h3 className={styles.title}>{title}</h3>
          {description && <p className={styles.description}>{description}</p>}
        </div>
        <div className={styles.actions}>
          <button
            type="button"
            onClick={downloadCSV}
            className={`${styles.btn} ${styles.btnCsv}`}
          >
            <FileSpreadsheet size={16} /> CSV
          </button>
          <button
            type="button"
            onClick={downloadChart}
            className={`${styles.btn} ${styles.btnImage}`}
          >
            <Download size={16} /> Image
          </button>
        </div>
      </div>

      <div
        ref={chartRef}
        className={styles.chartArea}
        style={{ height: `${height}px` }}
      >
        <ResponsiveContainer width="100%" height="100%">
          <LineChart
            data={data}
            margin={{ top: 10, right: 30, left: 60, bottom: 20 }}
          >
            <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />

            <XAxis
              dataKey={xAxisKey}
              label={{
                value: xAxisLabel,
                position: "insideBottomRight",
                offset: -10,
              }}
              padding={{ left: 20, right: 20 }}
            />

            <YAxis
              label={{
                value: yAxisLabel,
                angle: -90,
                position: "insideLeft",
                style: { textAnchor: "middle" },
                offset: -10,
              }}
            />

            <Tooltip
              formatter={(
                value:
                  | number
                  | string
                  | readonly (number | string)[]
                  | undefined,
              ) => {
                if (typeof value === "number")
                  return new Intl.NumberFormat().format(value);
                if (Array.isArray(value)) return value.join(", ");
                return value ?? "";
              }}
            />
            <Legend wrapperStyle={{ paddingTop: "20px" }} />

            {lines.map((line) => (
              <Line
                key={line.key}
                type="monotone"
                dataKey={line.key}
                stroke={algoColors[line.algoName] || "#000"}
                strokeWidth={line.runIndex === 0 ? 3 : 2}
                strokeOpacity={getOpacity(line.runIndex)}
                dot={{ r: line.runIndex === 0 ? 4 : 2 }}
                activeDot={{ r: 6 }}
              />
            ))}
          </LineChart>
        </ResponsiveContainer>
      </div>

      <div className={styles.controls}>
        <div className={styles.controlGroup}>
          <span className={styles.label}>Height:</span>
          <input
            type="range"
            min="300"
            max="800"
            step="50"
            value={height}
            onChange={(e) => setHeight(Number(e.target.value))}
            className={styles.rangeInput}
          />
          <span className={styles.rangeValue}>{height}px</span>
        </div>

        <div className={styles.colorPickerGroup}>
          <span className={styles.label}>
            <Settings size={14} /> Colors:
          </span>
          {algoNames.map((algoName) => (
            <div key={algoName} className={styles.colorItem}>
              <input
                type="color"
                value={algoColors[algoName] || "#000000"}
                onChange={(e) =>
                  handleAlgoColorChange(algoName, e.target.value)
                }
                className={styles.colorInput}
              />
              <span className={styles.colorName} title={algoName}>
                {algoName}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
