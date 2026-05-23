use crate::emit_progress_from;
use regex::Regex;
use std::sync::OnceLock;
use tauri::AppHandle;

pub struct PhaseProgress<'a> {
    pub app: &'a AppHandle,
    pub start: f64,
    pub end: f64,
}

impl PhaseProgress<'_> {
    pub fn emit_fraction(&self, fraction: f64, message: &str) {
        let f = fraction.clamp(0.0, 1.0);
        let percent = self.start + (self.end - self.start) * f;
        emit_progress_from(self.app, percent, message);
    }
}

#[derive(Clone, Copy)]
pub enum ProgressKind {
    YtDlp,
    Ffmpeg { duration_secs: f64 },
}

#[derive(Debug, Clone)]
pub struct ParsedProgress {
    pub fraction: f64,
    pub message: String,
}

pub fn parse_progress_line(kind: ProgressKind, line: &str) -> Option<ParsedProgress> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    match kind {
        ProgressKind::YtDlp => parse_ytdlp(trimmed),
        ProgressKind::Ffmpeg { duration_secs } => parse_ffmpeg(trimmed, duration_secs),
    }
}

fn ytdlp_percent_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\[download\]\s+(\d+(?:\.\d+)?)%").unwrap())
}

fn parse_ytdlp(line: &str) -> Option<ParsedProgress> {
    if let Some(caps) = ytdlp_percent_re().captures(line) {
        let pct: f64 = caps.get(1)?.as_str().parse().ok()?;
        let frac = (pct / 100.0).clamp(0.0, 1.0);
        return Some(ParsedProgress {
            fraction: frac,
            message: format!("Downloading… {pct:.1}%"),
        });
    }

    if line.contains("[Merger]") {
        return Some(ParsedProgress {
            fraction: 0.9,
            message: "Merging audio and video…".into(),
        });
    }
    if line.contains("[ExtractAudio]") {
        return Some(ParsedProgress {
            fraction: 0.85,
            message: "Extracting audio…".into(),
        });
    }

    None
}

fn parse_ffmpeg(line: &str, total_secs: f64) -> Option<ParsedProgress> {
    if total_secs <= 0.0 {
        return None;
    }

    let time_idx = line.find("time=")?;
    let after = &line[time_idx + 5..];
    let time_token: String = after
        .chars()
        .take_while(|c| *c == ':' || *c == '.' || c.is_ascii_digit())
        .collect();
    let elapsed = parse_ffmpeg_time(&time_token)?;
    let frac = (elapsed / total_secs).clamp(0.0, 0.99);
    let pct = frac * 100.0;
    Some(ParsedProgress {
        fraction: frac,
        message: format!("Processing… {pct:.0}%"),
    })
}

fn parse_ffmpeg_time(token: &str) -> Option<f64> {
    let parts: Vec<&str> = token.split(':').collect();
    match parts.len() {
        3 => {
            let h: f64 = parts[0].parse().ok()?;
            let m: f64 = parts[1].parse().ok()?;
            let s: f64 = parts[2].parse().ok()?;
            Some(h * 3600.0 + m * 60.0 + s)
        }
        2 => {
            let m: f64 = parts[0].parse().ok()?;
            let s: f64 = parts[1].parse().ok()?;
            Some(m * 60.0 + s)
        }
        1 => parts[0].parse().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ytdlp_download_percent() {
        let line = "[download]  42.5% of  10.00MiB at  1.00MiB/s ETA 00:05";
        let parsed = parse_progress_line(ProgressKind::YtDlp, line).unwrap();
        assert!((parsed.fraction - 0.425).abs() < 0.001);
    }

    #[test]
    fn parses_ffmpeg_time_fraction() {
        let line = "frame=  100 fps= 30 q=28.0 size=    1024kB time=00:00:30.00 bitrate= 500kbits/s";
        let parsed =
            parse_progress_line(ProgressKind::Ffmpeg { duration_secs: 60.0 }, line).unwrap();
        assert!((parsed.fraction - 0.5).abs() < 0.02);
    }
}
