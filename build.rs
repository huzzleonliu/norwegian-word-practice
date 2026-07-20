//! 编译期脚本：扫描 `data/lexicon-word-bank` 下的加密词库（`.nwpdict`）文件，
//! 生成 `LEXICON_WORD_BANK_FILES` 常量供运行时下拉框使用。

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be available"),
    );
    let lexicon_dir = manifest_dir.join("data").join("lexicon-word-bank");
    println!("cargo:rerun-if-changed={}", lexicon_dir.display());

    let mut files = collect_word_bank_files(&lexicon_dir);
    files.sort();

    // 生成 `LEXICON_WORD_BANK_FILES` 常量供 PracticeMode 下拉框使用。
    let output = render_catalog_source(&files);
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be available"));
    let output_path = out_dir.join("lexicon_word_bank_catalog.rs");
    fs::write(&output_path, output).expect("write generated word bank catalog");
}

fn collect_word_bank_files(dir: &PathBuf) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("nwpdict"))
        .filter_map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.to_string())
        })
        .collect()
}

fn render_catalog_source(files: &[String]) -> String {
    let mut source = String::from("pub const LEXICON_WORD_BANK_FILES: &[&str] = &[\n");
    for file in files {
        source.push_str("    ");
        source.push_str(&format!("{file:?}"));
        source.push_str(",\n");
    }
    source.push_str("];\n");
    source
}
