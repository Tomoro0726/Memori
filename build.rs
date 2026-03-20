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

    // Windows環境ではコマンド名が異なるための対応
    let bun_cmd = if cfg!(target_os = "windows") {
        "bun.cmd"
    } else {
        "bun"
    };

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
