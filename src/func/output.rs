use crate::{Func, Measurement};
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
#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReportManifest {
    functions: Vec<FunctionDataManifest>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FunctionDataManifest {
    name: String,
    main_json_path: String,
    max_history_number: u32,
}

impl<I> Func<I>
where
    I: Clone + serde::Serialize + std::fmt::Display + 'static, // 表示用に Display を追加
{
    /// Executes the full matrix of benchmarks with a rich CLI progress animation and
    /// automatically saves the results as JSON files in the `target/memori` directory.
    /// Generates a master `report.html` that aggregates all benchmarks.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// 【公開API】リッチなCLIプログレスアニメーションと共にすべてのベンチマークを実行し、
    /// 結果を自動的に `target/memori` ディレクトリ以下にJSONファイルとして保存します。
    /// さらに、`target/memori/report.html` と `target/memori/report-manifest.json` を生成します。
    /// HTML本体には結果を埋め込まず、ビューアーはフォルダ内のJSONをmanifest経由で参照します。
    /// </details>
    pub fn run_and_save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let base_path = PathBuf::from("target/memori").join(&self.name);
        fs::create_dir_all(&base_path)?;
        self.update_main_json(&base_path)?;

        // アニメーションONでコア処理を実行
        let report_map = self.execute_core(true);

        // 1. 履歴JSONの保存
        let next_num = self.get_next_file_number(&base_path);
        let history_path = base_path.join(format!("{:03}.json", next_num));

        let json_data = serde_json::to_string_pretty(&report_map)?;
        fs::write(history_path, json_data)?;

        // 2. マスター report.html を生成（target/memori 直下）
        Self::generate_master_report()?;

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
            println!("Start: {}", self.name);
        }

        for pattern in &self.patterns {
            if show_progress {
                println!(" ├─ Bench: {} ({:?})", pattern.name, pattern.description);
            }

            let pattern_type = match &pattern.input {
                crate::Bench::Instant(_) => "instant",
                crate::Bench::Scaling(_) => "scaling",
            };

            let mut pattern_results = BTreeMap::new();

            for (func_name, func) in &mut self.functions {
                if show_progress {
                    println!(" │   ├─ Func: {}", func_name);
                }
                let mut data_entries = Vec::new();

                match &pattern.input {
                    crate::Bench::Instant(val) => {
                        if show_progress {
                            print!("\r │   │   Progress: [1/1] N={:<10}", val);
                            std::io::stdout().flush().unwrap();
                        }

                        let mut runner = crate::runner::Runner::new(val.clone(), &mut **func);
                        let m = runner.run();
                        data_entries.push(BenchJsonEntry {
                            input: val.clone(),
                            measurement: m,
                        });

                        if show_progress {
                            println!("\r │   │   OK: [1/1] Input={:<10}", val);
                        }
                    }
                    crate::Bench::Scaling(vals) => {
                        let total = vals.len();
                        for (i, val) in vals.iter().enumerate() {
                            if show_progress {
                                print!("\r │   │   Progress: [{}/{}] N={:<10}", i + 1, total, val);
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
                            println!("\r │   │   OK: [{}/{}] {:<15}", total, total, "");
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
                        let name = e.file_name();
                        let name = name.to_str()?;

                        // Support both legacy `001_YYYY-MM-DD.json` and current `001.json`.
                        let base = name.trim_end_matches(".json");
                        let prefix = base.split('_').next()?;
                        prefix.parse::<u32>().ok()
                    })
                    .max()
                    .unwrap_or(0)
                    + 1
            })
            .unwrap_or(1)
    }

    /// Generates `report.html` and `report-manifest.json` in `target/memori`.
    ///
    /// `report.html` keeps only the viewer shell, while `report-manifest.json` stores
    /// relative JSON paths that the viewer loads at runtime.
    ///
    /// <details>
    /// <summary>Japanese</summary>
    ///
    /// 【内部API】`target/memori` 直下に `report.html` と `report-manifest.json` を生成します。
    /// HTMLにはデータ本体を埋め込まず、manifestにある相対パス経由でフォルダ内JSONを読み込みます。
    /// </details>
    fn generate_master_report() -> Result<(), Box<dyn std::error::Error>> {
        let memori_root = PathBuf::from("target/memori");

        // target/memori が存在します確認
        if !memori_root.exists() {
            return Ok(());
        }

        let html_template = include_str!("../../../../../viewer/dist/index.html");
        let mut function_manifests = Vec::new();
        let mut embedded_data = BTreeMap::new();

        // target/memori 直下のすべてのディレクトリを走査
        if let Ok(entries) = fs::read_dir(&memori_root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }

                let func_name = match path.file_name() {
                    Some(name) => name.to_string_lossy().to_string(),
                    None => continue,
                };

                let main_json_path = path.join("main.json");
                let mut max_history_number = 0u32;
                let mut history_list = Vec::new();
                let meta = if main_json_path.exists() {
                    match fs::read_to_string(&main_json_path) {
                        Ok(content) => serde_json::from_str::<FuncMetadata>(&content).ok(),
                        Err(_) => None,
                    }
                } else {
                    None
                };

                // 該当ディレクトリ内のすべてのJSONファイルを読み込む
                if let Ok(files) = fs::read_dir(&path) {
                    let mut file_entries: Vec<_> = files.filter_map(|e| e.ok()).collect();
                    // 降順（新しい順）にソート
                    file_entries.sort_by_key(|a| std::cmp::Reverse(a.file_name()));

                    for entry in file_entries {
                        let file_name = entry.file_name().to_string_lossy().to_string();
                        if file_name == "main.json"
                            || file_name == "report.html"
                            || file_name == "report-manifest.json"
                            || !file_name.ends_with(".json")
                        {
                            continue;
                        }

                        let prefix = file_name
                            .trim_end_matches(".json")
                            .split('_')
                            .next()
                            .unwrap_or("");
                        if let Ok(num) = prefix.parse::<u32>() {
                            if num > max_history_number {
                                max_history_number = num;
                            }

                            if let Ok(content) = fs::read_to_string(entry.path()) {
                                if let Ok(parsed) =
                                    serde_json::from_str::<serde_json::Value>(&content)
                                {
                                    history_list.push(serde_json::json!({
                                        "fileName": file_name,
                                        "data": parsed
                                    }));
                                }
                            }
                        }
                    }
                }

                if !main_json_path.exists() && max_history_number == 0 {
                    continue;
                }

                function_manifests.push(FunctionDataManifest {
                    name: func_name.clone(),
                    main_json_path: format!("{}/main.json", func_name),
                    max_history_number,
                });

                embedded_data.insert(
                    func_name,
                    serde_json::json!({
                        "meta": meta,
                        "history": history_list
                    }),
                );
            }
        }

        function_manifests.sort_by(|a, b| a.name.cmp(&b.name));

        let manifest = ReportManifest {
            functions: function_manifests,
        };

        let injected_json = serde_json::to_string(&embedded_data)?;
        let safe_json = injected_json.replace("</", "<\\/");
        let injection_script = format!(
            "<script>window.__memori_DATA__ = {};</script>\n</head>",
            safe_json
        );
        let final_html = html_template.replace("</head>", &injection_script);

        fs::write(memori_root.join("report.html"), final_html)?;
        fs::write(
            memori_root.join("report-manifest.json"),
            serde_json::to_string_pretty(&manifest)?,
        )?;

        println!("Output report.html");
        if let Ok(cwd) = std::env::current_dir() {
            let abs_report = cwd.join(&memori_root).join("report.html");
            // let abs_manifest = cwd.join(&memori_root).join("report-manifest.json");

            // Windows環境でもブラウザで開きやすいように \ を / に置換
            let report_url = abs_report.to_string_lossy().replace('\\', "/");
            // let manifest_url = abs_manifest.to_string_lossy().replace('\\', "/");

            #[cfg(target_os = "windows")]
            {
                println!("   file:///{}", report_url);
                // println!("   file:///{}", manifest_url);
            }
            #[cfg(not(target_os = "windows"))]
            {
                println!("   file://{}", report_url);
                // println!("   file://{}", manifest_url);
            }
        }

        Ok(())
    }
}
