use super::files::{
    assemble_library, decode_bytes, is_audio_name, is_lyric_name, CatalogFile, Track,
};
use gloo_net::http::Request;

include!(concat!(env!("OUT_DIR"), "/audio_book_catalog.rs"));

#[derive(Clone, Debug, PartialEq)]
pub struct Book {
    pub slug: String,
    pub title: String,
    pub files: Vec<BookFile>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BookFile {
    pub name: String,
}

pub fn catalog_books() -> Vec<Book> {
    AUDIO_BOOKS
        .iter()
        .map(|book| Book {
            slug: book.slug.to_string(),
            title: book.title.to_string(),
            files: book
                .files
                .iter()
                .copied()
                .map(|name| BookFile {
                    name: name.to_string(),
                })
                .collect(),
        })
        .collect()
}

pub async fn load_book(book: &Book) -> Result<Vec<Track>, String> {
    let mut files = Vec::new();
    for file in &book.files {
        let url = format!("/data/audio/{}/{}", book.slug, encode_path(&file.name));
        if is_lyric_name(&file.name) {
            files.push(CatalogFile {
                name: file.name.clone(),
                url: url.clone(),
                lyric_text: Some(fetch_text(&url).await?),
            });
            continue;
        }
        if is_audio_name(&file.name) {
            files.push(CatalogFile {
                name: file.name.clone(),
                url,
                lyric_text: None,
            });
        }
    }

    Ok(assemble_library(files, &book.slug))
}

fn encode_path(name: &str) -> String {
    name.split('/')
        .map(|part| {
            let mut encoded = String::new();
            for ch in part.chars() {
                match ch {
                    'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => encoded.push(ch),
                    ch => encoded.push_str(&format!("%{:02X}", ch as u32)),
                }
            }
            encoded
        })
        .collect::<Vec<_>>()
        .join("/")
}

async fn fetch_text(url: &str) -> Result<String, String> {
    let bytes = fetch_bytes(url).await?;
    Ok(decode_bytes(&bytes))
}

async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    let response = Request::get(url)
        .send()
        .await
        .map_err(|err| err.to_string())?;
    if !response.ok() {
        return Err(format!("HTTP {} for {url}", response.status()));
    }
    response.binary().await.map_err(|err| err.to_string())
}
