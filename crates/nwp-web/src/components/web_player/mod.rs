//! 跟读播放器：本地文件 / 内置音频书 + 歌词对照。

pub mod books;
pub mod files;
pub mod lyrics;

use crate::structures::word_bank_entry::UiLanguage;
use crate::app_state::UiState;
use crate::utils::i18n::tr;
use books::{catalog_books, load_book, Book};
use files::{
    build_library, build_library_from_files, file_list_to_vec, revoke_tracks, Track,
};
use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use lyrics::{active_line_index, should_repeat_line, LyricLine};
use std::time::Duration;
use wasm_bindgen::JsCast;
use web_sys::{
    HtmlAudioElement, HtmlInputElement, ScrollBehavior, ScrollIntoViewOptions,
    ScrollLogicalPosition,
};

const SPEEDS: [f64; 6] = [0.5, 0.75, 0.9, 1.0, 1.1, 1.25];

#[component]
pub fn WebPlayer() -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
    let file_input: NodeRef<html::Input> = NodeRef::new();
    let files_input: NodeRef<html::Input> = NodeRef::new();
    let audio_ref: NodeRef<html::Audio> = NodeRef::new();
    let lyrics_ref: NodeRef<html::Div> = NodeRef::new();

    let (tracks, set_tracks) = signal(Vec::<Track>::new());
    let (current_id, set_current_id) = signal(Option::<usize>::None);
    let (status, set_status) = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (playing, set_playing) = signal(false);
    let (current_time, set_current_time) = signal(0.0);
    let (duration, set_duration) = signal(0.0);
    let (speed, set_speed) = signal(1.0);
    let (loop_line, set_loop_line) = signal(false);
    let (loop_idx, set_loop_idx) = signal(Option::<usize>::None);
    let (dragging, set_dragging) = signal(false);
    let (books, set_books) = signal(Vec::<Book>::new());
    let (books_open, set_books_open) = signal(false);
    let (books_error, set_books_error) = signal(Option::<String>::None);
    let (active_book, set_active_book) = signal(Option::<String>::None);

    let current_track = Memo::new(move |_| {
        let id = current_id.get()?;
        tracks.get().into_iter().find(|track| track.id == id)
    });

    let lyrics = Memo::new(move |_| {
        current_track
            .get()
            .map(|track| track.lyrics)
            .unwrap_or_default()
    });

    let active_idx = Memo::new(move |_| active_line_index(&lyrics.get(), current_time.get()));

    let load_files = move |files: web_sys::FileList| {
        set_loading.set(true);
        set_status.set(tr(lang.get_untracked(), "正在匹配音频与歌词…", "Matching audio and lyrics…").into());
        spawn_local(async move {
            match build_library(&files).await {
                Ok(library) => apply_library(
                    library,
                    tracks,
                    set_tracks,
                    set_current_id,
                    set_status,
                    set_playing,
                    set_current_time,
                    set_duration,
                    set_active_book,
                    tr(
                        lang.get_untracked(),
                        "这个文件夹里没有音频文件。",
                        "No audio files found in this folder.",
                    ),
                    lang.get_untracked(),
                ),
                Err(_) => set_status.set(
                    tr(
                        lang.get_untracked(),
                        "无法读取该文件夹，请重试。",
                        "Could not read this folder. Please try again.",
                    )
                    .into(),
                ),
            }
            set_loading.set(false);
        });
    };

    let on_folder = move |_| {
        let Some(input) = file_input.get() else {
            return;
        };
        let Some(files) = input.files() else {
            return;
        };
        if files.length() == 0 {
            return;
        }
        load_files(files);
    };

    let on_files = move |_| {
        let Some(input) = files_input.get() else {
            return;
        };
        let Some(files) = input.files() else {
            return;
        };
        if files.length() == 0 {
            return;
        }
        load_files(files);
    };

    let pick_folder = move |_| {
        if let Some(input) = file_input.get() {
            let _ = input.set_attribute("webkitdirectory", "");
            let _ = input.set_attribute("directory", "");
            input.set_multiple(true);
            input.click();
        }
    };

    let pick_files = move |_| {
        if let Some(input) = files_input.get() {
            input.click();
        }
    };

    let toggle_books = move |_| {
        set_books_open.set(!books_open.get_untracked());
    };

    let open_book = move |book: Book| {
        set_books_open.set(false);
        set_loading.set(true);
        set_status.set(format!(
            "{} {}…",
            tr(lang.get_untracked(), "正在加载", "Loading"),
            book.title
        ));
        spawn_local(async move {
            match load_book(&book).await {
                Ok(library) => {
                    let count = library.len();
                    let with_lyrics = library.iter().filter(|track| track.has_lyrics()).count();
                    let with_audio = library.iter().filter(|track| track.has_audio()).count();
                    apply_library(
                        library,
                        tracks,
                        set_tracks,
                        set_current_id,
                        set_status,
                        set_playing,
                        set_current_time,
                        set_duration,
                        set_active_book,
                        tr(
                            lang.get_untracked(),
                            "这本书里没有音频或歌词。",
                            "No audio or lyrics found in this book.",
                        ),
                        lang.get_untracked(),
                    );
                    if count > 0 {
                        let mut msg = format!(
                            "{} {} · {count} {}，{with_lyrics} {}。",
                            tr(lang.get_untracked(), "已加载", "Loaded"),
                            book.title,
                            tr(lang.get_untracked(), "首", "tracks"),
                            tr(lang.get_untracked(), "首有歌词", "with lyrics")
                        );
                        if with_audio == 0 {
                            msg.push_str(tr(
                                lang.get_untracked(),
                                " 音频文件尚未放到服务器。",
                                " Audio files are not on the server yet.",
                            ));
                        }
                        set_status.set(msg);
                        set_active_book.set(Some(book.slug));
                    }
                }
                Err(err) => set_status.set(err),
            }
            set_loading.set(false);
        });
    };

    let play_track = move |id: usize| {
        set_current_id.set(Some(id));
        set_current_time.set(0.0);
        set_playing.set(true);
    };

    let toggle_play = move || {
        let Some(audio) = audio_ref.get() else {
            return;
        };
        if audio.paused() {
            let _ = audio.play();
            set_playing.set(true);
        } else {
            let _ = audio.pause();
            set_playing.set(false);
        }
    };

    let sync_clock = move |audio: HtmlAudioElement| {
        if audio.seeking() {
            return;
        }
        let total = audio.duration();
        if total.is_finite() {
            set_duration.set(total);
        }
        if apply_line_loop(
            &audio,
            &lyrics.get_untracked(),
            loop_line.get_untracked(),
            loop_idx.get_untracked(),
            set_current_time,
            set_playing,
        ) {
            return;
        }
        set_current_time.set(audio.current_time());
    };

    let begin_seek = move |time: f64, line: Option<usize>| {
        let time = time.max(0.0);
        if loop_line.get_untracked() {
            if let Some(index) = line {
                set_loop_idx.set(Some(index));
            } else {
                set_loop_idx.set(active_line_index(&lyrics.get_untracked(), time));
            }
        }
        if let Some(audio) = audio_ref.get() {
            audio.set_current_time(time);
            if !audio.seeking() {
                set_current_time.set(audio.current_time());
            }
            if playing.get_untracked() {
                let _ = audio.play();
            }
        }
    };

    let seek_to = move |time: f64| {
        begin_seek(time, None);
    };

    let jump_line = move |index: usize| {
        let lines = lyrics.get_untracked();
        if let Some(line) = lines.get(index) {
            set_playing.set(true);
            begin_seek(line.time, Some(index));
        }
    };

    let shift_line = move |delta: isize| {
        let lines = lyrics.get_untracked();
        if lines.is_empty() {
            return;
        }
        let current = active_idx.get_untracked().unwrap_or(0) as isize;
        let next = (current + delta).clamp(0, lines.len() as isize - 1) as usize;
        jump_line(next);
    };

    let toggle_loop = move || {
        let next = !loop_line.get_untracked();
        set_loop_line.set(next);
        if next {
            set_loop_idx.set(active_idx.get_untracked());
        } else {
            set_loop_idx.set(None);
        }
    };

    Effect::new(move |_| {
        let catalog = catalog_books();
        if catalog.is_empty() {
            set_books_error.set(Some(
                tr(
                    lang.get(),
                    "未找到内置音频书，请检查 data/audio。",
                    "No built-in books found. Check data/audio.",
                )
                .to_string(),
            ));
        } else {
            set_books_error.set(None);
        }
        set_books.set(catalog);
    });

    Effect::new(move |_| {
        if let Some(input) = file_input.get() {
            let _ = input.set_attribute("webkitdirectory", "");
            let _ = input.set_attribute("directory", "");
            input.set_multiple(true);
        }
    });

    Effect::new(move |_| {
        let rate = speed.get();
        if let Some(audio) = audio_ref.get() {
            audio.set_playback_rate(rate);
        }
    });

    Effect::new(move |_| {
        let _ = current_id.get();
        if loop_line.get_untracked() {
            set_loop_idx.set(None);
        }
    });

    Effect::new(move |_| {
        if loop_line.get() && loop_idx.get().is_none() {
            if let Some(index) = active_idx.get() {
                set_loop_idx.set(Some(index));
            }
        }
    });

    Effect::new(move |_| {
        if !playing.get() {
            return;
        }
        if let Ok(handle) = set_interval_with_handle(
            move || {
                if let Some(audio) = audio_ref.get() {
                    sync_clock(audio);
                }
            },
            Duration::from_millis(40),
        ) {
            on_cleanup(move || handle.clear());
        }
    });

    Effect::new(move |_| {
        let Some(track_id) = current_id.get() else {
            return;
        };
        let Some(index) = active_idx.get() else {
            return;
        };
        let Some(el) = document().get_element_by_id(&format!("lyric-{track_id}-{index}")) else {
            return;
        };
        let opts = ScrollIntoViewOptions::new();
        opts.set_behavior(ScrollBehavior::Smooth);
        opts.set_block(ScrollLogicalPosition::Center);
        el.scroll_into_view_with_scroll_into_view_options(&opts);
        let _ = lyrics_ref.get();
    });

    let _close_books = window_event_listener(ev::click, move |event| {
        if !books_open.get_untracked() {
            return;
        }
        let Some(target) = event.target() else {
            return;
        };
        let Ok(element) = target.dyn_into::<web_sys::Element>() else {
            set_books_open.set(false);
            return;
        };
        if element.closest(".web-player-books").ok().flatten().is_none() {
            set_books_open.set(false);
        }
    });

    on_cleanup(move || {
        if let Some(audio) = audio_ref.get() {
            let _ = audio.pause();
        }
        revoke_tracks(&tracks.get_untracked());
    });

    view! {
        <div
            class="web-player"
            class:is-dragging=move || dragging.get()
            on:dragenter=move |event: web_sys::DragEvent| {
                event.prevent_default();
                set_dragging.set(true);
            }
            on:dragover=move |event: web_sys::DragEvent| event.prevent_default()
            on:dragleave=move |event: web_sys::DragEvent| {
                event.prevent_default();
                set_dragging.set(false);
            }
            on:drop=move |event: web_sys::DragEvent| {
                event.prevent_default();
                set_dragging.set(false);
                let Some(transfer) = event.data_transfer() else {
                    return;
                };
                let Some(files) = transfer.files() else {
                    return;
                };
                if files.length() == 0 {
                    return;
                }
                set_loading.set(true);
                set_status.set(
                    tr(lang.get_untracked(), "正在匹配音频与歌词…", "Matching audio and lyrics…").into(),
                );
                let files = file_list_to_vec(&files);
                spawn_local(async move {
                    match build_library_from_files(files).await {
                        Ok(library) => apply_library(
                            library,
                            tracks,
                            set_tracks,
                            set_current_id,
                            set_status,
                            set_playing,
                            set_current_time,
                            set_duration,
                            set_active_book,
                            tr(
                                lang.get_untracked(),
                                "这些文件里没有音频。",
                                "No audio files found in this folder.",
                            ),
                            lang.get_untracked(),
                        ),
                        Err(_) => set_status.set(
                            tr(
                                lang.get_untracked(),
                                "无法读取这些文件，请重试。",
                                "Could not read these files. Please try again.",
                            )
                            .into(),
                        ),
                    }
                    set_loading.set(false);
                });
            }
        >
            <header class="web-player-topbar">
                <div class="web-player-actions">
                    <input
                        node_ref=file_input
                        class="hidden-input"
                        type="file"
                        multiple=true
                        on:change=on_folder
                    />
                    <input
                        node_ref=files_input
                        id="file-picker"
                        class="hidden-input"
                        type="file"
                        multiple=true
                        accept=".mp3,.wav,.flac,.ogg,.oga,.m4a,.aac,.opus,.webm,.lrc,.srt,audio/*"
                        on:change=on_files
                    />
                    <button class="ui-btn-ghost" on:click=pick_files disabled=move || loading.get()>
                        <PlayerIcon icon="lucide:file-audio" />
                        {move || tr(lang.get(), "选择文件", "Choose files")}
                    </button>
                    <button class="ui-btn-ghost" on:click=pick_folder disabled=move || loading.get()>
                        <Show
                            when=move || loading.get()
                            fallback=|| view! { <PlayerIcon icon="lucide:folder-open" /> }
                        >
                            <PlayerIcon icon="lucide:loader-circle" />
                        </Show>
                        {move || {
                            if loading.get() {
                                tr(lang.get(), "读取中…", "Reading…")
                            } else {
                                tr(lang.get(), "打开文件夹", "Open folder")
                            }
                        }}
                    </button>
                    <div class="web-player-books" on:click=move |event| event.stop_propagation()>
                        <button
                            class="ui-btn-primary"
                            class:active=move || books_open.get()
                            on:click=toggle_books
                            disabled=move || loading.get()
                        >
                            <PlayerIcon icon="lucide:book-open" />
                            {move || tr(lang.get(), "音频书", "Books")}
                        </button>
                        <Show when=move || books_open.get()>
                            <div class="web-player-book-menu" role="menu">
                                <Show
                                    when=move || !books.get().is_empty()
                                    fallback=move || view! {
                                        <p class="web-player-muted">
                                            {move || books_error.get().unwrap_or_else(|| {
                                                tr(lang.get(), "正在加载音频书…", "Loading books…").to_string()
                                            })}
                                        </p>
                                    }
                                >
                                    <For
                                        each=move || books.get()
                                        key=|book| book.slug.clone()
                                        children=move |book| {
                                            let slug = book.slug.clone();
                                            let title = book.title.clone();
                                            let selected = slug.clone();
                                            view! {
                                                <button
                                                    class="web-player-book-item"
                                                    class:active=move || active_book.get().as_deref() == Some(selected.as_str())
                                                    role="menuitem"
                                                    on:click=move |_| open_book(book.clone())
                                                >
                                                    <span class="web-player-book-title">{title}</span>
                                                    <span class="web-player-book-slug">{slug}</span>
                                                </button>
                                            }
                                        }
                                    />
                                </Show>
                            </div>
                        </Show>
                    </div>
                </div>
            </header>

            <div class="web-player-workspace">
                <TrackList
                    tracks=tracks
                    current_id=current_id
                    status=status
                    on_select=play_track
                />
                <LyricsStage
                    track=current_track
                    lyrics=lyrics
                    active_idx=active_idx
                    lyrics_ref=lyrics_ref
                    on_jump=jump_line
                    has_library=Signal::derive(move || !tracks.get().is_empty())
                    status=status
                />
            </div>

            <PlayerBar
                audio_ref=audio_ref
                track=current_track
                playing=playing
                current_time=current_time
                duration=duration
                speed=speed
                loop_line=loop_line
                on_toggle=toggle_play
                on_seek=seek_to
                on_speed=set_speed
                on_loop=toggle_loop
                on_prev=move || shift_line(-1)
                on_next=move || shift_line(1)
                on_time=sync_clock
                on_seeked=move |audio: HtmlAudioElement| {
                    if playing.get_untracked() {
                        let _ = audio.play();
                    }
                    sync_clock(audio);
                }
                on_can_play=move |audio: HtmlAudioElement| {
                    audio.set_playback_rate(speed.get_untracked());
                    let total = audio.duration();
                    if total.is_finite() {
                        set_duration.set(total);
                    }
                    if playing.get_untracked() && !audio.seeking() {
                        let _ = audio.play();
                    }
                }
                on_ended=move || {
                    if loop_line.get_untracked() {
                        if let Some(audio) = audio_ref.get() {
                            if !apply_line_loop(
                                &audio,
                                &lyrics.get_untracked(),
                                true,
                                loop_idx.get_untracked(),
                                set_current_time,
                                set_playing,
                            ) {
                                if let Some(index) = loop_idx.get_untracked()
                                    && let Some(line) = lyrics.get_untracked().get(index)
                                {
                                    audio.set_current_time(line.time);
                                    set_current_time.set(audio.current_time());
                                }
                                let _ = audio.play();
                                set_playing.set(true);
                            }
                        }
                    } else {
                        set_playing.set(false);
                    }
                }
            />
        </div>
    }
}

fn apply_library(
    library: Vec<Track>,
    tracks: ReadSignal<Vec<Track>>,
    set_tracks: WriteSignal<Vec<Track>>,
    set_current_id: WriteSignal<Option<usize>>,
    set_status: WriteSignal<String>,
    set_playing: WriteSignal<bool>,
    set_current_time: WriteSignal<f64>,
    set_duration: WriteSignal<f64>,
    set_active_book: WriteSignal<Option<String>>,
    empty_msg: &str,
    language: UiLanguage,
) {
    revoke_tracks(&tracks.get_untracked());
    set_playing.set(false);
    set_current_time.set(0.0);
    set_duration.set(0.0);
    set_active_book.set(None);
    let count = library.len();
    let with_lyrics = library.iter().filter(|track| track.has_lyrics()).count();
    if count == 0 {
        set_tracks.set(Vec::new());
        set_current_id.set(None);
        set_status.set(empty_msg.into());
        return;
    }
    let first_id = library.first().map(|track| track.id);
    set_tracks.set(library);
    set_current_id.set(first_id);
    set_status.set(format!(
        "{} {count} {}，{with_lyrics} {}。",
        tr(language, "已加载", "Loaded"),
        tr(language, "首", "tracks"),
        tr(language, "首有歌词", "with lyrics")
    ));
}

fn format_time(time: f64) -> String {
    if !time.is_finite() || time < 0.0 {
        return "0:00".into();
    }
    let total = time.round() as u64;
    format!("{}:{:02}", total / 60, total % 60)
}

fn apply_line_loop(
    audio: &HtmlAudioElement,
    lines: &[LyricLine],
    looping: bool,
    loop_idx: Option<usize>,
    set_current_time: WriteSignal<f64>,
    set_playing: WriteSignal<bool>,
) -> bool {
    if !looping {
        return false;
    }
    let Some(index) = loop_idx else {
        return false;
    };
    let Some(line) = lines.get(index) else {
        return false;
    };
    let duration = audio.duration();
    let total = if duration.is_finite() {
        duration
    } else {
        f64::MAX
    };
    if should_repeat_line(lines, index, audio.current_time(), total) || audio.ended() {
        audio.set_current_time(line.time);
        set_current_time.set(audio.current_time());
        let _ = audio.play();
        set_playing.set(true);
        true
    } else {
        false
    }
}

#[component]
fn TrackList(
    tracks: ReadSignal<Vec<Track>>,
    current_id: ReadSignal<Option<usize>>,
    status: ReadSignal<String>,
    on_select: impl Fn(usize) + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
    view! {
        <aside class="web-player-library">
            <div class="web-player-library-head">
                <h2>{move || tr(lang.get(), "曲目", "Tracks")}</h2>
                <span>
                    {move || {
                        let total = tracks.get().len();
                        if total == 0 {
                            tr(lang.get(), "空", "Empty").to_string()
                        } else {
                            format!("{total}")
                        }
                    }}
                </span>
            </div>
            <p class="web-player-status">{move || status.get()}</p>
            <Show
                when=move || !tracks.get().is_empty()
                fallback=move || view! {
                    <p class="web-player-muted">
                        {move || tr(
                            lang.get(),
                            "打开文件夹，或从服务器选择一本音频书。",
                            "Open a folder, or choose a book from the server.",
                        )}
                    </p>
                }
            >
                <ul class="web-player-track-list">
                    <For
                        each=move || tracks.get()
                        key=|track| track.id
                        children=move |track| {
                            let id = track.id;
                            let title = track.display_title();
                            let folder = track.folder.clone();
                            let has_lyrics = track.has_lyrics();
                            let has_audio = track.has_audio();
                            let lang = lang;
                            view! {
                                <li>
                                    <button
                                        class="web-player-track"
                                        class:active=move || current_id.get() == Some(id)
                                        on:click=move |_| on_select(id)
                                    >
                                        <span class="web-player-track-title">{title}</span>
                                        <span class="web-player-track-meta">
                                            {if folder.is_empty() {
                                                tr(lang.get_untracked(), "根目录", "Root").to_string()
                                            } else {
                                                folder
                                            }}
                                            {if has_lyrics {
                                                format!(" · {}", tr(lang.get_untracked(), "有歌词", "Lyrics"))
                                            } else {
                                                format!(" · {}", tr(lang.get_untracked(), "无歌词", "No lyrics"))
                                            }}
                                            {if has_audio {
                                                String::new()
                                            } else {
                                                format!(" · {}", tr(lang.get_untracked(), "无音频", "No audio"))
                                            }}
                                        </span>
                                    </button>
                                </li>
                            }
                        }
                    />
                </ul>
            </Show>
        </aside>
    }
}

#[component]
fn LyricsStage(
    track: Memo<Option<Track>>,
    lyrics: Memo<Vec<LyricLine>>,
    active_idx: Memo<Option<usize>>,
    lyrics_ref: NodeRef<html::Div>,
    on_jump: impl Fn(usize) + Copy + Send + Sync + 'static,
    has_library: Signal<bool>,
    status: ReadSignal<String>,
) -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
    let current_line = Memo::new(move |_| {
        let lines = lyrics.get();
        active_idx.get().and_then(|index| lines.get(index).cloned())
    });
    let next_line = Memo::new(move |_| {
        current_line.get().and_then(|line| {
            lyrics
                .get()
                .into_iter()
                .find(|item| item.id == line.id + 1)
        })
    });
    let lyric_rows = Memo::new(move |_| {
        let track_id = track.get().map(|item| item.id).unwrap_or(usize::MAX);
        lyrics
            .get()
            .into_iter()
            .map(|line| (track_id, line))
            .collect::<Vec<_>>()
    });

    view! {
        <section class="web-player-stage">
            <Show
                when=move || track.get().is_some()
                fallback=move || view! {
                    <EmptyState has_library=has_library status=status />
                }
            >
                <div class="web-player-now">
                    <p class="web-player-eyebrow">{move || track.get().map(|item| item.audio_name).unwrap_or_default()}</p>
                    <h2>{move || track.get().map(|item| item.display_title()).unwrap_or_default()}</h2>
                    <div class="web-player-focus">
                        <p class="web-player-focus-text">
                            {move || current_line.get().map(|line| line.text).unwrap_or_else(|| {
                                tr(lang.get(), "就绪", "Ready").to_string()
                            })}
                        </p>
                        <p class="web-player-focus-tr">
                            {move || current_line.get().and_then(|line| line.translation).unwrap_or_default()}
                        </p>
                        <p class="web-player-focus-next">
                            {move || next_line.get().map(|line| {
                                format!("{}  {}", tr(lang.get(), "下一句", "Next"), line.text)
                            }).unwrap_or_default()}
                        </p>
                    </div>
                </div>
                <Show
                    when=move || !lyrics.get().is_empty()
                    fallback=move || view! {
                        <p class="web-player-muted web-player-stage-empty">
                            {move || tr(
                                lang.get(),
                                "没有找到歌词。请把同名的 .lrc 或 .srt 放在音频旁边。",
                                "No lyrics found. Put a same-named .lrc or .srt next to the audio file.",
                            )}
                        </p>
                    }
                >
                    <div class="web-player-lyrics" node_ref=lyrics_ref>
                        <For
                            each=move || lyric_rows.get()
                            key=|(track_id, line)| (*track_id, line.id)
                            children=move |(track_id, line)| {
                                let line_id = line.id;
                                let time_label = format_time(line.time);
                                view! {
                                    <button
                                        id=format!("lyric-{track_id}-{line_id}")
                                        class="web-player-lyric"
                                        class:active=move || current_line.get().map(|item| item.id) == Some(line_id)
                                        on:click=move |_| on_jump(line_id)
                                    >
                                        <span class="web-player-lyric-time">{time_label}</span>
                                        <span class="web-player-lyric-body">
                                            <span class="web-player-lyric-text">{line.text}</span>
                                            {line.translation.map(|text| view! { <span class="web-player-lyric-tr">{text}</span> })}
                                        </span>
                                    </button>
                                }
                            }
                        />
                    </div>
                </Show>
            </Show>
        </section>
    }
}

#[component]
fn EmptyState(has_library: Signal<bool>, status: ReadSignal<String>) -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
    view! {
        <div class="web-player-empty">
            <p class="web-player-eyebrow">{move || tr(lang.get(), "音频 + 歌词", "audio + lyrics")}</p>
            <h2>{move || tr(lang.get(), "跟读", "read along")}</h2>
            <p>
                {move || tr(
                    lang.get(),
                    "打开本地文件夹或选择一本音频书。播放器按文件名匹配音频与歌词，播放时歌词会滚动，点击某行即可跳转。",
                    "Open a local folder or choose a book. The player matches audio with lyrics by filename. Lyrics scroll as the track plays. Click a line to jump there.",
                )}
            </p>
            <ul>
                <li>{move || tr(lang.get(), "支持 mp3 / flac / wav / m4a / ogg 以及 .lrc 或 .srt", "Supports mp3 / flac / wav / m4a / ogg and .lrc or .srt")}</li>
                <li>{move || tr(lang.get(), "同一时间戳的两行会成为原文与译文", "Two lines with the same timestamp become source and translation")}</li>
            </ul>
            <p class="web-player-status">{move || {
                if has_library.get() {
                    String::new()
                } else {
                    status.get()
                }
            }}</p>
        </div>
    }
}

#[component]
fn PlayerBar(
    audio_ref: NodeRef<html::Audio>,
    track: Memo<Option<Track>>,
    playing: ReadSignal<bool>,
    current_time: ReadSignal<f64>,
    duration: ReadSignal<f64>,
    speed: ReadSignal<f64>,
    loop_line: ReadSignal<bool>,
    on_toggle: impl Fn() + Copy + Send + Sync + 'static,
    on_seek: impl Fn(f64) + Copy + Send + Sync + 'static,
    on_speed: WriteSignal<f64>,
    on_loop: impl Fn() + Copy + Send + Sync + 'static,
    on_prev: impl Fn() + Copy + Send + Sync + 'static,
    on_next: impl Fn() + Copy + Send + Sync + 'static,
    on_time: impl Fn(HtmlAudioElement) + Copy + Send + Sync + 'static,
    on_seeked: impl Fn(HtmlAudioElement) + Copy + Send + Sync + 'static,
    on_can_play: impl Fn(HtmlAudioElement) + Copy + Send + Sync + 'static,
    on_ended: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    let lang = expect_context::<UiState>().ui_language;
    let progress = move || {
        let total = duration.get();
        if total > 0.0 {
            (current_time.get() / total * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        }
    };

    view! {
        <footer class="web-player-bar">
            <audio
                node_ref=audio_ref
                src=move || {
                    track
                        .get()
                        .map(|item| item.audio_url)
                        .filter(|url| !url.is_empty())
                        .unwrap_or_default()
                }
                preload="auto"
                on:timeupdate=move |_| {
                    if let Some(audio) = audio_ref.get() {
                        on_time(audio);
                    }
                }
                on:seeked=move |_| {
                    if let Some(audio) = audio_ref.get() {
                        on_seeked(audio);
                    }
                }
                on:canplay=move |_| {
                    if let Some(audio) = audio_ref.get() {
                        on_can_play(audio);
                    }
                }
                on:ended=move |_| on_ended()
            />
            <div class="web-player-transport">
                <button
                    class="ui-btn-ghost web-player-icon-btn"
                    title=move || tr(lang.get(), "上一句", "Previous line")
                    aria-label=move || tr(lang.get(), "上一句", "Previous line")
                    on:click=move |_| on_prev()
                >
                    <PlayerIcon icon="lucide:arrow-big-left-dash" size=20 />
                </button>
                <button
                    class="ui-btn-primary web-player-icon-btn"
                    title=move || if playing.get() { tr(lang.get(), "暂停", "Pause") } else { tr(lang.get(), "播放", "Play") }
                    aria-label=move || if playing.get() { tr(lang.get(), "暂停", "Pause") } else { tr(lang.get(), "播放", "Play") }
                    on:click=move |_| on_toggle()
                    disabled=move || track.get().map(|item| item.audio_url.is_empty()).unwrap_or(true)
                >
                    {move || {
                        if playing.get() {
                            view! { <PlayerIcon icon="lucide:pause" size=22 /> }
                        } else {
                            view! { <PlayerIcon icon="lucide:play" size=22 /> }
                        }
                    }}
                </button>
                <button
                    class="ui-btn-ghost web-player-icon-btn"
                    title=move || tr(lang.get(), "下一句", "Next line")
                    aria-label=move || tr(lang.get(), "下一句", "Next line")
                    on:click=move |_| on_next()
                >
                    <PlayerIcon icon="lucide:arrow-big-right-dash" size=20 />
                </button>
            </div>
            <div class="web-player-scrubber">
                <span>{move || format_time(current_time.get())}</span>
                <input
                    type="range"
                    min="0"
                    max="1000"
                    step="1"
                    prop:value=move || (progress() * 10.0).round()
                    disabled=move || {
                        track
                            .get()
                            .map(|item| item.audio_url.is_empty())
                            .unwrap_or(true)
                    }
                    on:input=move |event| {
                        let value = event_target::<HtmlInputElement>(&event).value_as_number();
                        let time = duration.get_untracked() * value / 1000.0;
                        on_seek(time);
                    }
                />
                <span>{move || format_time(duration.get())}</span>
            </div>
            <div class="web-player-study">
                <div class="web-player-speeds">
                    {SPEEDS.into_iter().map(|value| {
                        let label = match value {
                            0.5 => "0.5x",
                            0.75 => "0.75x",
                            0.9 => "0.9x",
                            1.0 => "1x",
                            1.1 => "1.1x",
                            _ => "1.25x",
                        };
                        view! {
                            <button
                                class="web-player-speed"
                                class:active=move || (speed.get() - value).abs() < f64::EPSILON
                                on:click=move |_| on_speed.set(value)
                            >
                                {label}
                            </button>
                        }
                    }).collect_view()}
                </div>
                <button
                    id="loop-line"
                    class="web-player-loop"
                    class:active=move || loop_line.get()
                    title=move || tr(lang.get(), "循环本句", "Loop line")
                    on:click=move |_| on_loop()
                >
                    <PlayerIcon icon="lucide:repeat-1" />
                    {move || tr(lang.get(), "循环", "Loop")}
                </button>
            </div>
        </footer>
    }
}

#[component]
fn PlayerIcon(icon: &'static str, #[prop(default = 18)] size: i32) -> impl IntoView {
    let size = size.to_string();
    view! {
        <iconify-icon class="web-player-icon" icon=icon width=size.clone() height=size></iconify-icon>
    }
}
