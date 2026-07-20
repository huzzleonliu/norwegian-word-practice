//! CSV schema 测试：覆盖版本前缀、兼容读取与高版本拦截。

use crate::utils::csv_schema::{parse_word_bank_csv, serialize_word_bank_csv};

#[test]
fn serialize_prefixes_schema_version() {
    let csv = serialize_word_bank_csv(&[]).expect("serialize empty csv should succeed");
    assert!(csv.starts_with("#schema_version=1\nid,selected,part_of_speech,"));
}

#[test]
fn parse_accepts_legacy_csv_without_schema_line() {
    let csv_with_schema = serialize_word_bank_csv(&[]).expect("serialize empty csv should succeed");
    let legacy_csv = csv_with_schema
        .lines()
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n");
    let parsed = parse_word_bank_csv(&legacy_csv).expect("legacy csv should parse");
    assert!(parsed.is_empty());
}

#[test]
fn parse_rejects_higher_schema_version() {
    let csv_with_schema = serialize_word_bank_csv(&[]).expect("serialize empty csv should succeed");
    let unsupported = csv_with_schema.replacen("#schema_version=1", "#schema_version=999", 1);
    let err = parse_word_bank_csv(&unsupported).expect_err("higher schema should be rejected");
    assert!(err.contains("版本过高"));
}
