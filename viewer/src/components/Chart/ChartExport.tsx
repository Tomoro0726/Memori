/**
 * ChartExport component
 *
 * グラフをCSV/PNG形式でエクスポートする機能
 */

import { toPng } from "html-to-image";
import { Download, FileSpreadsheet } from "lucide-react";
import type React from "react";
import { useCallback } from "react";
import type { LineInfo } from "../../types";
import styles from "./BenchmarkChart.module.css";

interface ChartExportProps {
  /** グラフのタイトル */
  title: string;
  /** グラフデータ */
  data: Record<string, unknown>[];
  /** グラフラインリスト */
  lines: LineInfo[];
  /** X軸キー */
  xAxisKey: string;
  /** グラフ参照ノード */
  chartRef: React.RefObject<HTMLDivElement | null>;
}

/**
 * グラフエクスポートコンポーネント
 * CSV形式でのエクスポート、PNG形式での保存に対応
 */
export const ChartExport: React.FC<ChartExportProps> = ({
  title,
  data,
  lines,
  xAxisKey,
  chartRef,
}) => {
  /**
   * グラフをPNG画像として保存
   */
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
  }, [title, chartRef]);

  /**
   * グラフをCSV形式でダウンロード
   */
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
  );
};
