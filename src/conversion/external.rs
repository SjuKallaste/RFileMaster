use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub enum ExternalTool {
    Ffmpeg,
    Pandoc,
    LibreOffice,
    YtDlp,
}

impl ExternalTool {
    pub fn find(&self) -> Option<std::path::PathBuf> {
        let names: &[&str] = match self {
            ExternalTool::Ffmpeg => &["ffmpeg"],
            ExternalTool::Pandoc => &["pandoc"],
            ExternalTool::LibreOffice => &["libreoffice", "soffice", "libreoffice7.6", "libreoffice7.5"],
            ExternalTool::YtDlp => &["yt-dlp", "youtube-dl"],
        };
        for name in names {
            if let Ok(path) = which::which(name) {
                return Some(path);
            }
        }
        None
    }

    pub fn name(&self) -> &'static str {
        match self {
            ExternalTool::Ffmpeg => "ffmpeg",
            ExternalTool::Pandoc => "pandoc",
            ExternalTool::LibreOffice => "LibreOffice",
            ExternalTool::YtDlp => "yt-dlp",
        }
    }
}

pub fn require(tool: ExternalTool) -> Result<std::path::PathBuf, String> {
    tool.find().ok_or_else(|| format!(
        "{} is not installed or not on PATH. Install it to use this conversion.",
        tool.name()
    ))
}

pub fn ffmpeg(input: &Path, output: &Path, extra_args: &[&str]) -> Result<(), String> {
    let bin = require(ExternalTool::Ffmpeg)?;
    let status = Command::new(bin)
        .arg("-y")
        .arg("-i")
        .arg(input)
        .args(extra_args)
        .arg(output)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("ffmpeg exited with status {}", status))
    }
}

pub fn pandoc(input: &Path, output: &Path, extra_args: &[&str]) -> Result<(), String> {
    let bin = require(ExternalTool::Pandoc)?;
    let status = Command::new(bin)
        .arg(input)
        .arg("-o")
        .arg(output)
        .args(extra_args)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("pandoc exited with status {}", status))
    }
}

pub fn libreoffice_convert(input: &Path, target_ext: &str, output_dir: &Path) -> Result<(), String> {
    let bin = require(ExternalTool::LibreOffice)?;
    let status = Command::new(bin)
        .arg("--headless")
        .arg("--convert-to")
        .arg(target_ext)
        .arg("--outdir")
        .arg(output_dir)
        .arg(input)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("LibreOffice exited with status {}", status))
    }
}

pub fn ffmpeg_audio(input: &Path, output: &Path, target: &str) -> Result<(), String> {
    let extra: &[&str] = match target {
        "mp3" => &["-q:a", "2"],
        "ogg" => &["-c:a", "libvorbis", "-q:a", "4"],
        "opus" => &["-c:a", "libopus", "-b:a", "128k"],
        "flac" => &["-c:a", "flac"],
        "aac" | "m4a" => &["-c:a", "aac", "-b:a", "192k"],
        "wav" => &["-c:a", "pcm_s16le"],
        _ => &[],
    };
    ffmpeg(input, output, extra)
}

pub fn yt_dlp_download(url: &str, output_dir: &Path, want_audio_only: bool, target_ext: &str) -> Result<PathBuf, String> {
    let bin = require(ExternalTool::YtDlp)?;
    let out_template = output_dir.join("%(title)s.%(ext)s");
    let mut cmd = Command::new(bin);
    cmd.arg(url).arg("-o").arg(&out_template).arg("--no-playlist");

    if want_audio_only {
        cmd.arg("-x").arg("--audio-format").arg(target_ext);
    } else {
        cmd.arg("--merge-output-format").arg(target_ext);
    }

    let before: std::collections::HashSet<PathBuf> = fs::read_dir(output_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();

    let status = cmd.status().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("yt-dlp exited with status {}", status));
    }

    let after: std::collections::HashSet<PathBuf> = fs::read_dir(output_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();

    after.difference(&before)
        .find(|p| p.extension().and_then(|e| e.to_str()) == Some(target_ext))
        .cloned()
        .ok_or_else(|| "yt-dlp finished but the output file could not be located".to_string())
}
pub fn ffmpeg_video(input: &Path, output: &Path, target: &str) -> Result<(), String> {
    let extra: &[&str] = match target {
        "webm" => &["-c:v", "libvpx-vp9", "-crf", "30", "-b:v", "0", "-c:a", "libopus"],
        "mkv" => &["-c:v", "copy", "-c:a", "copy"],
        "avi" => &["-c:v", "mpeg4", "-c:a", "mp3"],
        "mov" => &["-c:v", "libx264", "-c:a", "aac"],
        "gif" => &["-vf", "fps=15,scale=640:-1:flags=lanczos"],
        "mp3" => &["-vn", "-q:a", "2"],
        "wav" => &["-vn", "-c:a", "pcm_s16le"],
        _ => &[],
    };
    ffmpeg(input, output, extra)
}