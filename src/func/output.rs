use crate::{Func, Measurement};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

// --- JSON出力用の構造体 ---

/// A structured report for a single benchmark pattern, containing results for all registered functions.
/// Designed for JSON serialization to be consumed by the viewer.
///
/// <details>
/// <summary>Japanese</summary>
///
/// 単一のベンチマークパターンに対する構造化されたレポートです。
/// 登録された全関数の結果が含まれており、ビューワーで読み込むためのJSONシリアライズに使用されます。
/// </details>
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchJsonReport<I> {
    pub pattern_type: String, // "instant" または "scaling"
    pub description: String,
    // 関数名をキーにして、その計測結果の配列を保持する
    pub results: BTreeMap<String, Vec<BenchJsonEntry<I>>>,
}

/// A single data point representing the measurement result for a specific input size.
///
/// <details>
/// <summary>Japanese</summary>
///
/// 特定の入力サイズに対する計測結果を表す単一のデータポイントです。
/// </details>
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchJsonEntry<I> {
    pub input: I,
    pub measurement: Measurement,
}

/// Metadata for the entire benchmark suite.
/// Saved as `main.json` to act as an index/manifest for the frontend viewer.
///
/// <details>
/// <summary>Japanese</summary>
///
/// ベンチマークスイート全体のメタデータです。
/// フロントエンドのビューワー用の目次（マニフェスト）として `main.json` に保存されます。
/// </details>
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FuncMetadata {
    pub name: String,
    pub description: Option<String>,
    pub functions: Vec<String>,
    pub patterns: Vec<PatternMetadata>,
}

/// Metadata describing a specific benchmark pattern.
///
/// <details>
/// <summary>Japanese</summary>
///
/// 特定のベンチマークパターンを説明するメタデータです。
/// </details>
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PatternMetadata {
    pub name: String,
    pub description: String,
    pub pattern_type: String,
}

impl<I> Func<I>
where
    I: Clone + serde::Serialize + std::fmt::Display + 'static, // 表示用に Display を追加
{
    /// Executes the full matrix of benchmarks purely in memory and returns the structured results.
    ///
    /// CLI progress animations are disabled. This is highly useful for programmatic access,
    /// testing, or server-side execution where standard output manipulation is undesirable.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// 【公開API】すべてのベンチマークをメモリ上でのみ実行し、構造化された結果を返します。
    ///
    /// CLIのプログレスアニメーションは無効化されます。標準出力の書き換えを避けたい場合や、
    /// プログラムからの直接アクセス、自動テスト、サーバーサイドでの実行に非常に便利です。
    /// </details>
    pub fn run_all(&mut self) -> BTreeMap<String, BenchJsonReport<I>> {
        self.execute_core(false) // アニメーションOFF
    }

    /// Executes the full matrix of benchmarks with a rich CLI progress animation and
    /// automatically saves the results as JSON files in the `target/tenbin` directory.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// 【公開API】リッチなCLIプログレスアニメーションと共にすべてのベンチマークを実行し、
    /// 結果を自動的に `target/tenbin` ディレクトリ以下にJSONファイルとして保存します。
    /// </details>
    pub fn run_and_save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let base_path = PathBuf::from("target/tenbin").join(&self.name);
        fs::create_dir_all(&base_path)?;
        self.update_main_json(&base_path)?;

        // アニメーションONでコア処理を実行
        let report_map = self.execute_core(true);

        let next_num = self.get_next_file_number(&base_path);
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let history_path = base_path.join(format!("{:03}_{}.json", next_num, date_str));

        let json_data = serde_json::to_string_pretty(&report_map)?;
        fs::write(history_path, json_data)?;

        println!("✨ すべての計測が完了し、{} に保存されました。", self.name);
        Ok(())
    }

    /// The core execution loop for benchmarks. Handles both silent and animated CLI execution.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// 【内部API】ベンチマークのコア実行ループです。
    /// `show_progress` フラグによって、静かな実行とアニメーション付きのCLI実行を切り替えます。
    /// </details>
    fn execute_core(&mut self, show_progress: bool) -> BTreeMap<String, BenchJsonReport<I>> {
        let mut report_map = BTreeMap::new();

        if show_progress {
            println!("🚀 実行開始: {}", self.name);
        }

        for pattern in &self.patterns {
            if show_progress {
                println!(" ├─ パターン: {} ({:?})", pattern.name, pattern.description);
            }

            let pattern_type = match &pattern.input {
                crate::Bench::Instant(_) => "instant",
                crate::Bench::Scaling(_) => "scaling",
            };

            let mut pattern_results = BTreeMap::new();

            for (func_name, func) in &mut self.functions {
                if show_progress {
                    println!(" │   ├─ 関数: {}", func_name);
                }
                let mut data_entries = Vec::new();

                match &pattern.input {
                    crate::Bench::Instant(val) => {
                        if show_progress {
                            print!("\r │   │   ⏳ 計測中: [1/1] N={:<10}", val);
                            std::io::stdout().flush().unwrap();
                        }

                        let mut runner = crate::runner::Runner::new(val.clone(), &mut **func);
                        let m = runner.run();
                        data_entries.push(BenchJsonEntry {
                            input: val.clone(),
                            measurement: m,
                        });

                        if show_progress {
                            println!("\r │   │   ✅ 計測完了: [1/1] Input={:<10}", val);
                        }
                    }
                    crate::Bench::Scaling(vals) => {
                        let total = vals.len();
                        for (i, val) in vals.iter().enumerate() {
                            if show_progress {
                                print!("\r │   │   ⏳ 計測中: [{}/{}] N={:<10}", i + 1, total, val);
                                std::io::stdout().flush().unwrap();
                            }

                            let mut runner = crate::runner::Runner::new(val.clone(), &mut **func);
                            let m = runner.run();
                            data_entries.push(BenchJsonEntry {
                                input: val.clone(),
                                measurement: m,
                            });
                        }
                        if show_progress {
                            println!("\r │   │   ✅ 計測完了: [{}/{}] {:<15}", total, total, "");
                        }
                    }
                }
                pattern_results.insert(func_name.clone(), data_entries);
            }

            report_map.insert(
                pattern.name.clone(),
                BenchJsonReport {
                    pattern_type: pattern_type.to_string(),
                    description: pattern.description.clone(),
                    results: pattern_results,
                },
            );
        }
        report_map
    }

    /// Updates the `main.json` metadata file if the suite configuration has changed.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// スイートの構成（関数やパターンの追加など）が変更されている場合、
    /// `main.json` メタデータファイルを最新化します。
    /// </details>
    fn update_main_json(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let main_path = path.join("main.json");
        let current_meta = FuncMetadata {
            name: self.name.clone(),
            description: self.description.clone(),
            functions: self
                .functions
                .iter()
                .map(|(name, _)| name.clone())
                .collect(),
            patterns: self
                .patterns
                .iter()
                .map(|p| PatternMetadata {
                    name: p.name.clone(),
                    description: p.description.clone(),
                    pattern_type: match p.input {
                        crate::Bench::Instant(_) => "instant".to_string(),
                        crate::Bench::Scaling(_) => "scaling".to_string(),
                    },
                })
                .collect(),
        };

        let should_write = if main_path.exists() {
            let content = fs::read_to_string(&main_path)?;
            if let Ok(existing) = serde_json::from_str::<FuncMetadata>(&content) {
                existing != current_meta
            } else {
                true
            }
        } else {
            true
        };

        if should_write {
            fs::write(main_path, serde_json::to_string_pretty(&current_meta)?)?;
        }
        Ok(())
    }

    /// Determines the next available sequential number for saving history files.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// 履歴ファイルを保存するための、次に利用可能な連番（例: 001, 002）を決定します。
    /// </details>
    fn get_next_file_number(&self, path: &Path) -> u32 {
        fs::read_dir(path)
            .map(|dir| {
                dir.flatten()
                    .filter_map(|e| {
                        e.file_name()
                            .to_str()?
                            .split('_')
                            .next()?
                            .parse::<u32>()
                            .ok()
                    })
                    .max()
                    .unwrap_or(0)
                    + 1
            })
            .unwrap_or(1)
    }
}
