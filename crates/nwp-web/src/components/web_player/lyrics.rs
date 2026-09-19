#[derive(Clone, Debug, PartialEq)]
pub struct LyricLine {
    pub id: usize,
    pub time: f64,
    pub text: String,
    pub translation: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LyricsDoc {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub lines: Vec<LyricLine>,
}

pub fn parse_lyrics(filename: &str, content: &str) -> LyricsDoc {
    let lower = filename.to_ascii_lowercase();
    if lower.ends_with(".srt") {
        parse_srt(content)
    } else {
        parse_lrc(content)
    }
}

pub fn active_line_index(lines: &[LyricLine], time: f64) -> Option<usize> {
    lines.iter().rposition(|line| line.time <= time)
}

pub fn line_end(lines: &[LyricLine], index: usize, duration: f64) -> f64 {
    lines
        .get(index + 1)
        .map(|line| line.time)
        .unwrap_or(duration)
}

pub fn should_repeat_line(lines: &[LyricLine], index: usize, time: f64, duration: f64) -> bool {
    let Some(line) = lines.get(index) else {
        return false;
    };
    let end = line_end(lines, index, duration);
    let held_long_enough = time - line.time >= 0.12;
    let reached_end = time + 0.02 >= end;
    let reached_file_end = duration.is_finite() && duration > 0.0 && time + 0.05 >= duration;
    held_long_enough && (reached_end || reached_file_end)
}

fn parse_lrc(content: &str) -> LyricsDoc {
    let mut title = None;
    let mut artist = None;
    let mut offset_secs = 0.0;
    let mut pairs: Vec<(f64, String)> = Vec::new();

    for raw in content.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(value) = meta_tag(line, "ti") {
            title = Some(value);
            continue;
        }
        if let Some(value) = meta_tag(line, "ar") {
            artist = Some(value);
            continue;
        }
        if let Some(value) = meta_tag(line, "offset") {
            if let Ok(ms) = value.parse::<f64>() {
                offset_secs = ms / 1000.0;
            }
            continue;
        }
        if meta_tag(line, "al").is_some()
            || meta_tag(line, "by").is_some()
            || meta_tag(line, "re").is_some()
            || meta_tag(line, "ve").is_some()
        {
            continue;
        }

        let mut stamps = Vec::new();
        let mut rest = line;
        while let Some(parsed) = take_timestamp(rest) {
            stamps.push(parsed.0);
            rest = parsed.1;
        }

        let text = strip_word_tags(rest).trim().to_string();
        if stamps.is_empty() || text.is_empty() {
            continue;
        }
        for stamp in stamps {
            pairs.push(((stamp - offset_secs).max(0.0), text.clone()));
        }
    }

    pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    LyricsDoc {
        title,
        artist,
        lines: merge_pairs(pairs),
    }
}

fn parse_srt(content: &str) -> LyricsDoc {
    let mut pairs: Vec<(f64, String)> = Vec::new();
    let blocks = content.replace('\r', "");
    for block in blocks.split("\n\n") {
        let mut lines = block.lines().filter(|line| !line.trim().is_empty());
        let Some(first) = lines.next() else {
            continue;
        };
        let time_line = if first.contains("-->") {
            first
        } else {
            match lines.next() {
                Some(line) if line.contains("-->") => line,
                _ => continue,
            }
        };
        let Some(start) = time_line.split("-->").next() else {
            continue;
        };
        let Some(time) = parse_srt_time(start.trim()) else {
            continue;
        };
        let text_lines: Vec<String> = lines.map(|line| line.trim().to_string()).collect();
        if text_lines.is_empty() {
            continue;
        }
        if text_lines.len() == 1 {
            pairs.push((time, text_lines[0].clone()));
        } else {
            pairs.push((
                time,
                format!("{}\n{}", text_lines[0], text_lines[1..].join(" ")),
            ));
        }
    }

    let mut lines = merge_pairs(pairs);
    for line in &mut lines {
        if line.translation.is_none()
            && let Some((text, translation)) = line.text.split_once('\n')
        {
            line.translation = Some(translation.trim().to_string());
            line.text = text.trim().to_string();
        }
    }

    LyricsDoc {
        title: None,
        artist: None,
        lines,
    }
}

fn merge_pairs(pairs: Vec<(f64, String)>) -> Vec<LyricLine> {
    let mut lines: Vec<LyricLine> = Vec::new();
    for (time, text) in pairs {
        if let Some(last) = lines.last_mut()
            && (last.time - time).abs() < 0.001
            && last.translation.is_none()
            && last.text != text
        {
            last.translation = Some(text);
            continue;
        }
        lines.push(LyricLine {
            id: 0,
            time: time.max(0.0),
            text,
            translation: None,
        });
    }
    for (index, line) in lines.iter_mut().enumerate() {
        line.id = index;
    }
    lines
}

fn meta_tag(line: &str, key: &str) -> Option<String> {
    let prefix = format!("[{key}:");
    let lower = line.to_ascii_lowercase();
    if !lower.starts_with(&prefix.to_ascii_lowercase()) || !line.ends_with(']') {
        return None;
    }
    Some(line[prefix.len()..line.len() - 1].trim().to_string())
}

fn take_timestamp(input: &str) -> Option<(f64, &str)> {
    let input = input.trim_start();
    if !input.starts_with('[') {
        return None;
    }
    let close = input.find(']')?;
    let time = parse_lrc_time(&input[1..close])?;
    Some((time, &input[close + 1..]))
}

fn parse_lrc_time(value: &str) -> Option<f64> {
    let value = value.trim().replace(',', ".");
    let parts: Vec<&str> = value.split(':').map(str::trim).collect();
    match parts.as_slice() {
        [minutes, seconds] => {
            let minutes: f64 = minutes.parse().ok()?;
            let seconds: f64 = seconds.parse().ok()?;
            Some(minutes * 60.0 + seconds)
        }
        [first, second, third] => {
            let a: f64 = first.parse().ok()?;
            let b: f64 = second.parse().ok()?;
            let c: f64 = third.parse().ok()?;
            if third.contains('.') || c >= 100.0 {
                Some(a * 3600.0 + b * 60.0 + c)
            } else {
                Some(a * 60.0 + b + c / 100.0)
            }
        }
        _ => None,
    }
}

fn parse_srt_time(value: &str) -> Option<f64> {
    let value = value.replace(',', ".");
    let mut parts = value.split(':');
    let hours: f64 = parts.next()?.trim().parse().ok()?;
    let minutes: f64 = parts.next()?.trim().parse().ok()?;
    let seconds: f64 = parts.next()?.trim().parse().ok()?;
    Some(hours * 3600.0 + minutes * 60.0 + seconds)
}

fn strip_word_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find('<') {
        output.push_str(&rest[..start]);
        match rest[start..].find('>') {
            Some(end) => rest = &rest[start + end + 1..],
            None => {
                output.push_str(rest);
                return output;
            }
        }
    }
    output.push_str(rest);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_lrc() {
        let doc = parse_lrc(
            "[ti:Hello]\n[ar:Ada]\n[00:01.00]Hi\n[00:02.50]There\n[offset:+0]\n",
        );
        assert_eq!(doc.title.as_deref(), Some("Hello"));
        assert_eq!(doc.artist.as_deref(), Some("Ada"));
        assert_eq!(doc.lines.len(), 2);
        assert_eq!(doc.lines[1].time, 2.5);
        assert_eq!(active_line_index(&doc.lines, 2.0), Some(0));
        assert_eq!(active_line_index(&doc.lines, 2.49), Some(0));
        assert_eq!(active_line_index(&doc.lines, 2.5), Some(1));
        assert_eq!(active_line_index(&doc.lines, 2.51), Some(1));
    }

    #[test]
    fn parses_colon_centiseconds() {
        let doc = parse_lrc("[00:01:70]Hello\n[00:02.50]There\n");
        assert!((doc.lines[0].time - 1.7).abs() < 1e-6);
        assert_eq!(doc.lines[1].time, 2.5);
    }

    #[test]
    fn click_target_uses_exact_timestamp() {
        let doc = parse_lrc("[00:01.00]One\n[00:02.50]Two\n[00:04.00]Three\n");
        assert_eq!(active_line_index(&doc.lines, 2.49), Some(0));
        assert_eq!(active_line_index(&doc.lines, 2.5), Some(1));
        assert_eq!(active_line_index(&doc.lines, 3.99), Some(1));
        assert_eq!(active_line_index(&doc.lines, 4.0), Some(2));
    }

    #[test]
    fn merges_translation_on_same_timestamp() {
        let doc = parse_lrc("[00:01.00]Hello\n[00:01.00]你好\n");
        assert_eq!(doc.lines.len(), 1);
        assert_eq!(doc.lines[0].text, "Hello");
        assert_eq!(doc.lines[0].translation.as_deref(), Some("你好"));
    }

    #[test]
    fn parses_srt_blocks() {
        let doc = parse_srt("1\n00:00:01,000 --> 00:00:02,000\nHello\n你好\n\n");
        assert_eq!(doc.lines[0].text, "Hello");
        assert_eq!(doc.lines[0].translation.as_deref(), Some("你好"));
    }

    #[test]
    fn repeats_the_locked_line_even_after_overshoot() {
        let lines = vec![
            LyricLine {
                id: 0,
                time: 1.0,
                text: "a".into(),
                translation: None,
            },
            LyricLine {
                id: 1,
                time: 3.0,
                text: "b".into(),
                translation: None,
            },
        ];
        assert_eq!(line_end(&lines, 0, 10.0), 3.0);
        assert!(!should_repeat_line(&lines, 0, 1.05, 10.0));
        assert!(should_repeat_line(&lines, 0, 3.0, 10.0));
        assert!(should_repeat_line(&lines, 0, 3.4, 10.0));
        assert!(should_repeat_line(&lines, 1, 9.95, 10.0));
    }
}
