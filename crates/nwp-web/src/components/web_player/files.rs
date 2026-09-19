use super::lyrics::{parse_lyrics, LyricsDoc};
use encoding_rs::{GB18030, UTF_16BE, UTF_16LE};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{File, FileList, Url};

pub(crate) const AUDIO_EXTS: &[&str] = &[
    "mp3", "wav", "flac", "ogg", "oga", "m4a", "aac", "opus", "webm",
];
pub(crate) const LYRIC_EXTS: &[&str] = &["lrc", "srt"];

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub id: usize,
    pub title: String,
    pub artist: Option<String>,
    pub folder: String,
    pub audio_name: String,
    pub audio_url: String,
    pub lyric_name: Option<String>,
    pub lyrics: Vec<super::lyrics::LyricLine>,
}

impl Track {
    pub fn has_lyrics(&self) -> bool {
        !self.lyrics.is_empty()
    }

    pub fn has_audio(&self) -> bool {
        !self.audio_url.is_empty()
    }

    pub fn display_title(&self) -> String {
        match &self.artist {
            Some(artist) if !artist.is_empty() => format!("{artist} · {}", self.title),
            _ => self.title.clone(),
        }
    }
}

pub struct CatalogFile {
    pub name: String,
    pub url: String,
    pub lyric_text: Option<String>,
}

struct PendingFile {
    name: String,
    relative_path: String,
    file: File,
}

pub async fn build_library(list: &FileList) -> Result<Vec<Track>, JsValue> {
    let files = file_list_to_vec(list);
    build_library_from_files(files).await
}

pub async fn build_library_from_files(files: Vec<File>) -> Result<Vec<Track>, JsValue> {
    let mut pending: Vec<PendingFile> = files
        .into_iter()
        .map(|file| {
            let relative_path = relative_path(&file);
            PendingFile {
                name: file_name(&relative_path),
                relative_path,
                file,
            }
        })
        .collect();
    pending.sort_by(|a, b| {
        a.relative_path
            .to_lowercase()
            .cmp(&b.relative_path.to_lowercase())
    });

    let mut audio = Vec::new();
    let mut lyrics = Vec::new();
    for item in pending {
        let ext = extension(&item.name);
        if AUDIO_EXTS.contains(&ext.as_str()) {
            audio.push(item);
        } else if LYRIC_EXTS.contains(&ext.as_str()) {
            lyrics.push(item);
        }
    }

    let mut tracks = Vec::new();
    let mut used_lyrics = vec![false; lyrics.len()];

    for (id, item) in audio.into_iter().enumerate() {
        let folder = parent_dir(&item.relative_path);
        let stem = file_stem(&item.name);
        let lyric_idx = find_lyric(&lyrics, &used_lyrics, folder, &stem);
        let mut doc = LyricsDoc::default();
        let mut lyric_name = None;
        if let Some(idx) = lyric_idx {
            used_lyrics[idx] = true;
            lyric_name = Some(lyrics[idx].name.clone());
            let content = read_text(&lyrics[idx].file).await?;
            doc = parse_lyrics(&lyrics[idx].name, &content);
        }

        let title = doc
            .title
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| pretty_title(&stem));
        let audio_url = Url::create_object_url_with_blob(&item.file)?;

        tracks.push(Track {
            id,
            title,
            artist: doc.artist.filter(|value| !value.is_empty()),
            folder: folder.to_string(),
            audio_name: item.name,
            audio_url,
            lyric_name,
            lyrics: doc.lines,
        });
    }

    Ok(tracks)
}

pub fn revoke_tracks(tracks: &[Track]) {
    for track in tracks {
        if track.audio_url.starts_with("blob:") {
            let _ = Url::revoke_object_url(&track.audio_url);
        }
    }
}

pub(crate) fn is_audio_name(name: &str) -> bool {
    AUDIO_EXTS.contains(&extension(name).as_str())
}

pub(crate) fn is_lyric_name(name: &str) -> bool {
    LYRIC_EXTS.contains(&extension(name).as_str())
}

pub fn assemble_library(files: Vec<CatalogFile>, folder: &str) -> Vec<Track> {
    let mut audio = Vec::new();
    let mut lyrics = Vec::new();
    for file in files {
        if is_audio_name(&file.name) {
            audio.push(file);
        } else if is_lyric_name(&file.name) {
            lyrics.push(file);
        }
    }
    audio.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    lyrics.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let mut used_lyrics = vec![false; lyrics.len()];
    let mut tracks = Vec::new();

    for item in audio {
        let stem = file_stem(&item.name);
        let lyric_idx = lyrics.iter().enumerate().find_map(|(idx, lyric)| {
            if used_lyrics[idx] {
                return None;
            }
            lyric_rank(&stem, &file_stem(&lyric.name), &extension(&lyric.name)).map(|_| idx)
        });
        let mut doc = LyricsDoc::default();
        let mut lyric_name = None;
        if let Some(idx) = lyric_idx {
            used_lyrics[idx] = true;
            lyric_name = Some(lyrics[idx].name.clone());
            if let Some(content) = &lyrics[idx].lyric_text {
                doc = parse_lyrics(&lyrics[idx].name, content);
            }
        }
        let title = doc
            .title
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| pretty_title(&stem));
        tracks.push(Track {
            id: tracks.len(),
            title,
            artist: doc.artist.filter(|value| !value.is_empty()),
            folder: folder.to_string(),
            audio_name: item.name,
            audio_url: item.url,
            lyric_name,
            lyrics: doc.lines,
        });
    }

    for (idx, item) in lyrics.into_iter().enumerate() {
        if used_lyrics[idx] {
            continue;
        }
        let stem = file_stem(&item.name);
        let content = item.lyric_text.as_deref().unwrap_or_default();
        let doc = parse_lyrics(&item.name, content);
        let title = doc
            .title
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| pretty_title(&stem));
        tracks.push(Track {
            id: tracks.len(),
            title,
            artist: doc.artist.filter(|value| !value.is_empty()),
            folder: folder.to_string(),
            audio_name: item.name.clone(),
            audio_url: String::new(),
            lyric_name: Some(item.name),
            lyrics: doc.lines,
        });
    }

    tracks
}

pub fn file_list_to_vec(list: &FileList) -> Vec<File> {
    (0..list.length()).filter_map(|i| list.item(i)).collect()
}

fn find_lyric(
    lyrics: &[PendingFile],
    used: &[bool],
    folder: &str,
    audio_stem: &str,
) -> Option<usize> {
    let mut ranked: Vec<(u8, usize)> = Vec::new();
    for (idx, item) in lyrics.iter().enumerate() {
        if used[idx] {
            continue;
        }
        if parent_dir(&item.relative_path) != folder {
            continue;
        }
        let stem = file_stem(&item.name);
        let rank = match lyric_rank(audio_stem, &stem, &extension(&item.name)) {
            Some(rank) => rank,
            None => continue,
        };
        ranked.push((rank, idx));
    }
    ranked.sort_by_key(|(rank, _)| *rank);
    ranked.first().map(|(_, idx)| *idx)
}

fn lyric_rank(audio_stem: &str, lyric_stem: &str, ext: &str) -> Option<u8> {
    let audio = audio_stem.to_ascii_lowercase();
    let lyric = lyric_stem.to_ascii_lowercase();
    let ext_score = if ext == "lrc" { 0 } else { 1 };
    if lyric == audio {
        return Some(ext_score);
    }
    if lyric.starts_with(&format!("{audio}.")) || lyric.starts_with(&format!("{audio}_")) {
        return Some(10 + ext_score);
    }
    None
}

fn relative_path(file: &File) -> String {
    js_sys::Reflect::get(file, &JsValue::from_str("webkitRelativePath"))
        .ok()
        .and_then(|value| value.as_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| file.name())
}

fn file_name(path: &str) -> String {
    path.rsplit(['/', '\\'])
        .next()
        .unwrap_or(path)
        .to_string()
}

fn parent_dir(path: &str) -> &str {
    path.rfind(['/', '\\']).map(|i| &path[..i]).unwrap_or("")
}

pub(crate) fn extension(name: &str) -> String {
    name.rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .unwrap_or_default()
}

pub(crate) fn file_stem(name: &str) -> String {
    match name.rfind('.') {
        Some(i) if i > 0 => name[..i].to_string(),
        _ => name.to_string(),
    }
}

pub(crate) fn pretty_title(stem: &str) -> String {
    stem.replace(['_', '-'], " ")
}

pub(crate) fn decode_bytes(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return UTF_16LE.decode(bytes).0.into_owned();
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return UTF_16BE.decode(bytes).0.into_owned();
    }
    let utf8_bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    if let Ok(text) = std::str::from_utf8(utf8_bytes) {
        return text.to_string();
    }
    GB18030.decode(utf8_bytes).0.into_owned()
}

async fn read_text(file: &File) -> Result<String, JsValue> {
    let buffer = JsFuture::from(file.array_buffer()).await?;
    let array = js_sys::Uint8Array::new(&buffer);
    let mut bytes = vec![0u8; array.length() as usize];
    array.copy_to(&mut bytes);
    Ok(decode_bytes(&bytes))
}

#[cfg(test)]
mod tests {
    use super::{file_stem, lyric_rank, parent_dir};

    #[test]
    fn ranks_exact_lrc_first() {
        assert_eq!(lyric_rank("hello", "hello", "lrc"), Some(0));
        assert_eq!(lyric_rank("hello", "hello.en", "lrc"), Some(10));
        assert_eq!(lyric_rank("hello", "other", "lrc"), None);
    }

    #[test]
    fn parses_paths() {
        assert_eq!(parent_dir("music/nested/song.mp3"), "music/nested");
        assert_eq!(file_stem("song.en.lrc"), "song.en");
    }
}
