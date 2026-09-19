//! 编译期脚本：扫描内置词库与跟读音频书目，生成运行时常量。

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be available"),
    );
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be available"));

    let lexicon_dir = manifest_dir.join("data").join("lexicon-word-bank");
    println!("cargo:rerun-if-changed={}", lexicon_dir.display());
    let mut files = collect_word_bank_files(&lexicon_dir);
    files.sort();
    fs::write(
        out_dir.join("lexicon_word_bank_catalog.rs"),
        render_lexicon_catalog(&files),
    )
    .expect("write generated word bank catalog");

    let audio_dir = manifest_dir.join("data").join("audio");
    println!("cargo:rerun-if-changed={}", audio_dir.display());
    fs::write(
        out_dir.join("audio_book_catalog.rs"),
        render_audio_catalog(&collect_audio_books(&audio_dir)),
    )
    .expect("write generated audio book catalog");
}

fn collect_word_bank_files(dir: &Path) -> Vec<String> {
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

fn render_lexicon_catalog(files: &[String]) -> String {
    let mut source = String::from("pub const LEXICON_WORD_BANK_FILES: &[&str] = &[\n");
    for file in files {
        source.push_str("    ");
        source.push_str(&format!("{file:?}"));
        source.push_str(",\n");
    }
    source.push_str("];\n");
    source
}

struct AudioBook {
    slug: String,
    title: String,
    files: Vec<String>,
}

fn collect_audio_books(root: &Path) -> Vec<AudioBook> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut dirs: Vec<_> = entries.filter_map(|entry| entry.ok()).collect();
    dirs.sort_by_key(|entry| entry.file_name());

    let mut books = Vec::new();
    for entry in dirs {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(slug) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if slug.starts_with('.') {
            continue;
        }
        let mut files = Vec::new();
        if let Ok(children) = fs::read_dir(&path) {
            let mut names: Vec<_> = children
                .filter_map(|child| child.ok())
                .filter(|child| child.path().is_file())
                .filter_map(|child| child.file_name().into_string().ok())
                .filter(|name| !name.starts_with('.'))
                .collect();
            names.sort();
            files = names;
        }
        books.push(AudioBook {
            slug: slug.to_string(),
            title: title_from_slug(slug),
            files,
        });
    }
    books
}

fn title_from_slug(slug: &str) -> String {
    slug.replace(['-', '_'], " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_audio_catalog(books: &[AudioBook]) -> String {
    let mut source = String::from(
        "#[derive(Clone, Copy)]\n\
pub struct AudioBookDef {\n\
    pub slug: &'static str,\n\
    pub title: &'static str,\n\
    pub files: &'static [&'static str],\n\
}\n\n\
pub const AUDIO_BOOKS: &[AudioBookDef] = &[\n",
    );
    for book in books {
        source.push_str("    AudioBookDef {\n");
        source.push_str(&format!("        slug: {:?},\n", book.slug));
        source.push_str(&format!("        title: {:?},\n", book.title));
        source.push_str("        files: &[\n");
        for file in &book.files {
            source.push_str(&format!("            {file:?},\n"));
        }
        source.push_str("        ],\n");
        source.push_str("    },\n");
    }
    source.push_str("];\n");
    source
}
