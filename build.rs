use std::path::Path;
use std::process::Command;

fn main() {
    // 1. Cargoに「どのファイルが変更されたらこのスクリプトを再実行するか」を教える
    // これにより、Rust側の変更だけでReact側が毎回ビルドされるのを防ぎ、コンパイルを高速に保ちます。
    println!("cargo:rerun-if-changed=viewer/src");
    println!("cargo:rerun-if-changed=viewer/index.html");
    println!("cargo:rerun-if-changed=viewer/vite.config.ts");
    println!("cargo:rerun-if-changed=viewer/package.json");

    let viewer_dir = Path::new("viewer");

    // 2. viewerディレクトリが存在しない場合は何もしない（超重要）
    // ※クレートとして crates.io に公開した際、エンドユーザーの環境で
    // bun のインストールを要求してエラーになるのを防ぐための安全対策です。
    if !viewer_dir.join("package.json").exists() {
        return;
    }

    // bun のパスを複数試す
    let bun_executables = if cfg!(target_os = "windows") {
        vec![
            "bun.cmd".to_string(),
            "bun".to_string(),
            format!(
                "{}/.bun/bin/bun.exe",
                std::env::var("USERPROFILE").unwrap_or_default()
            ),
        ]
    } else {
        vec!["bun".to_string()]
    };

    let mut bun_found = false;
    let mut bun_cmd = "bun";

    // 最初に利用可能な bun を探す
    for candidate in &bun_executables {
        let status = Command::new(candidate).arg("--version").status();
        if status.is_ok() {
            bun_found = true;
            bun_cmd = candidate;
            break;
        }
    }

    if !bun_found {
        println!("⚠️  Warning: 'bun' command not found. Skipping viewer build.");
        println!("    Install bun from https://bun.sh to build the viewer.");
        return;
    }

    // 3. 実際に `bun run build` を実行する
    let status = Command::new(bun_cmd)
        .current_dir(viewer_dir)
        .args(&["run", "build"])
        .status()
        .expect("bunコマンドの実行に失敗しました。bunがインストールされているか確認してください。");

    // ビルドに失敗した場合はRustのコンパイルも止める
    if !status.success() {
        panic!("❌ フロントエンド (viewer) のビルドに失敗しました。");
    }
}
