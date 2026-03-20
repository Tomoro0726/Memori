import { useState } from "react";
import styles from "./ParameterDescription.module.css";

const PARAMETERS = [
  {
    key: "CPU Cycles",
    en: "The pure execution cost. Linux: Uses perf_event. x86_64 (Intel Mac/Windows): Uses RDTSC. aarch64 (Apple Silicon): Uses CNTVCT_EL0.",
    ja: "純粋な実行コスト。Linux: perf_eventで厳密計測。x86_64 (Intel Mac/Windows): RDTSCを利用。aarch64 (Apple Silicon): CNTVCT_EL0（仮想タイマーtick）を利用。",
  },
  {
    key: "Time (ns)",
    en: "Real wall-clock execution time in nanoseconds. Only available when the `real_time` feature is enabled. Note: Enabling this adds a small timing overhead.",
    ja: "実実行時間（ナノ秒）。`real_time` フィーチャーが有効な場合のみ取得可能です。※注意: 有効にするとタイマー取得のわずかなオーバーヘッドが追加されます。",
  },
  {
    key: "Allocations (Count)",
    en: "The number of times heap memory allocation was requested.",
    ja: "関数実行中のヒープメモリ確保（Alloc）の要求回数。",
  },
  {
    key: "Allocated Memory (Bytes)",
    en: "The total amount of heap memory requested, in bytes.",
    ja: "要求されたヒープメモリの総確保量（バイト数）。",
  },
  {
    key: "Deallocated (Count)",
    en: "The number of times heap memory deallocation was requested.",
    ja: "関数実行中のヒープメモリ解放（Dealloc）の要求回数。",
  },
  {
    key: "Deallocated (Bytes)",
    en: "The total amount of heap memory deallocated, in bytes.",
    ja: "解放されたヒープメモリの総量（バイト数）。",
  },
  {
    key: "Net Memory Increase",
    en: "Net memory increase (Allocated Bytes - Deallocated Bytes).",
    ja: "正味のメモリ増加量（確保バイト数 - 解放バイト数）。",
  },
];

export function ParameterDescription() {
  const [open, setOpen] = useState(false);

  return (
    <div className={styles.container}>
      <button
        className={styles.toggle}
        onClick={() => setOpen((v) => !v)}
        aria-expanded={open}
      >
        <span className={styles.icon}>{open ? "−" : "+"}</span>
        Metrics & Environment Details (パラメーターと計測環境の詳細)
      </button>

      {open && (
        <div className={styles.content}>
          <section className={styles.section}>
            <h4 className={styles.sectionTitle}>Metrics Definition</h4>
            <dl className={styles.list}>
              {PARAMETERS.map((p) => (
                <div
                  key={p.key}
                  className={styles.item}
                  style={{ marginBottom: "12px" }}
                >
                  <dt className={styles.term}>{p.key}</dt>
                  <dd className={styles.description}>
                    <div style={{ lineHeight: "1.4" }}>{p.en}</div>
                    <div
                      style={{
                        fontSize: "0.85em",
                        opacity: 0.8,
                        marginTop: "4px",
                        lineHeight: "1.4",
                      }}
                    >
                      {p.ja}
                    </div>
                  </dd>
                </div>
              ))}
            </dl>
          </section>

          <p
            className={styles.footerNote}
            style={{
              marginTop: "24px",
              paddingTop: "12px",
              borderTop: "1px solid var(--border-color, #eee)",
            }}
          >
            <span style={{ display: "block", marginBottom: "4px" }}>
              * Memory statistics are tracked accurately regardless of the OS or
              architecture, using a custom allocator that wraps GlobalAlloc.
            </span>
            <span style={{ fontSize: "0.85em", opacity: 0.8 }}>
              ※ メモリ統計は GlobalAlloc
              をラップしたカスタムアロケーターにより、OSやアーキテクチャを問わず正確に追跡されます。
            </span>
            <br />
            <span style={{ display: "block", marginBottom: "4px" }}>
              * Instructions count is omitted from this list because it is
              fundamentally impossible to track from user-space on macOS/Windows
              without kernel extensions.
            </span>
            <span style={{ fontSize: "0.85em", opacity: 0.8 }}>
              ※ 命令数 (Instructions) は macOS や Windows
              のユーザー空間からは根本的に取得不可能なため、このリストからは省略されています
              (Linux環境でのみ内部的に記録されます)。
            </span>
          </p>
        </div>
      )}
    </div>
  );
}
