use crate::{Func, Measurement};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

// --- JSON出力用の構造体 ---

/// A structured report for a single benchmark pattern, containing results for all registered functions.
/// Designed for JSON serialization to be consumed by the viewer.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchJsonReport<I> {
    pub pattern_type: String, // "instant" または "scaling"
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    // 関数名をキーにして、その計測結果の配列を保持する
    pub results: BTreeMap<String, Vec<BenchJsonEntry<I>>>,
}

/// A single data point representing the measurement result for a specific input size.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchJsonEntry<I> {
    pub input: I,
    pub measurement: Measurement,
}

/// Metadata for the entire benchmark suite.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FuncMetadata {
    pub name: String,
    pub description: Option<String>,
    pub functions: Vec<String>,
    pub patterns: Vec<PatternMetadata>,
}

/// Metadata describing a specific benchmark pattern.
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
    I: Clone + serde::Serialize + std::fmt::Display + 'static,
{
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

        // 2. マスター report.html を生成
        Self::generate_master_report()?;

        Ok(())
    }

    fn execute_core(&mut self, show_progress: bool) -> BTreeMap<String, BenchJsonReport<I>> {
        let mut report_map = BTreeMap::new();
        let comment = std::env::var("MEMORI_COMMENT").ok();

        if show_progress {
            println!("Start: {}", self.name);
            if let Some(ref c) = comment {
                println!(" ├─ Comment: {}", c);
            }
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
                    comment: comment.clone(),
                    results: pattern_results,
                },
            );
        }
        report_map
    }

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

    fn get_next_file_number(&self, path: &Path) -> u32 {
        fs::read_dir(path)
            .map(|dir| {
                dir.flatten()
                    .filter_map(|e| {
                        let name = e.file_name();
                        let name = name.to_str()?;
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

    fn generate_master_report() -> Result<(), Box<dyn std::error::Error>> {
        let memori_root = PathBuf::from("target/memori");

        if !memori_root.exists() {
            return Ok(());
        }

        let html_template = include_str!("../../viewer/dist/index.html");
        let mut function_manifests = Vec::new();
        let mut embedded_data = BTreeMap::new();

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

                if let Ok(files) = fs::read_dir(&path) {
                    let mut file_entries: Vec<_> = files.filter_map(|e| e.ok()).collect();
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
            let report_url = abs_report.to_string_lossy().replace('\\', "/");

            #[cfg(target_os = "windows")]
            {
                println!("   file:///{}", report_url);
            }
            #[cfg(not(target_os = "windows"))]
            {
                println!("   file://{}", report_url);
            }
        }

        Ok(())
    }
}
