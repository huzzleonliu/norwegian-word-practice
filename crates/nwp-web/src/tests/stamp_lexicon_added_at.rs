//! 一次性迁移：给 data/ 下所有 .nwpdict 词条写入当前添加时间后重新加密。
//! 运行：cargo test -p nwp-web --bin nwp-web stamp_all_nwpdict_added_at -- --exact --nocapture

use std::fs;
use std::path::PathBuf;

use crate::utils::csv_schema::{parse_word_bank_csv, serialize_word_bank_csv};
use crate::utils::dictionary::current_added_at_timestamp;
use crate::utils::dictionary_crypto::{
    parse_lexicon_import_content, serialize_encrypted_lexicon_export,
};

#[test]
#[ignore = "一次性迁移：给所有内置 .nwpdict 写入添加时间"]
fn stamp_all_nwpdict_added_at() {
    let data_roots = [
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/lexicon-word-bank"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/series-word-bank"),
    ];

    let now = current_added_at_timestamp();
    let mut stamped_files = 0_usize;
    let mut stamped_entries = 0_usize;

    for root in data_roots {
        let entries = fs::read_dir(&root).unwrap_or_else(|err| {
            panic!("无法读取目录 {}: {err}", root.display());
        });
        for entry in entries {
            let entry = entry.expect("读取目录项失败");
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("nwpdict") {
                continue;
            }

            let raw = fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("读取失败 {}: {err}", path.display()));
            let csv = parse_lexicon_import_content(&raw)
                .unwrap_or_else(|err| panic!("解密失败 {}: {err}", path.display()));
            let mut words = parse_word_bank_csv(&csv)
                .unwrap_or_else(|err| panic!("CSV 解析失败 {}: {err}", path.display()));

            for word in &mut words {
                word.added_at = now.clone();
            }
            stamped_entries += words.len();

            let csv_out = serialize_word_bank_csv(&words)
                .unwrap_or_else(|err| panic!("CSV 序列化失败 {}: {err}", path.display()));
            let encrypted = serialize_encrypted_lexicon_export(&csv_out)
                .unwrap_or_else(|err| panic!("加密失败 {}: {err}", path.display()));
            fs::write(&path, encrypted)
                .unwrap_or_else(|err| panic!("写入失败 {}: {err}", path.display()));

            stamped_files += 1;
            println!(
                "stamped {} entries in {}",
                words.len(),
                path.file_name().unwrap().to_string_lossy()
            );
        }
    }

    println!("done: {stamped_files} files, {stamped_entries} entries, added_at={now}");
    assert!(stamped_files > 0, "未找到任何 .nwpdict 文件");
}
