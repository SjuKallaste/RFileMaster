use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    Queued,
    Running(f32),
    Done(PathBuf),
    Failed(String),
}

impl JobStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, JobStatus::Done(_) | JobStatus::Failed(_))
    }
}

#[derive(Debug, Clone)]
pub enum JobSource {
    Files(Vec<PathBuf>),
    Url(String),
}

#[derive(Debug, Clone)]
pub struct ConversionJob {
    pub id: u64,
    pub source: JobSource,
    pub source_format: String,
    pub target_format: String,
    pub output_path: PathBuf,
    pub merge: bool,
    pub status: JobStatus,
}

impl ConversionJob {
    pub fn new(id: u64, input_paths: Vec<PathBuf>, source_format: String, target_format: String, output_path: PathBuf, merge: bool) -> Self {
        Self {
            id,
            source: JobSource::Files(input_paths),
            source_format,
            target_format,
            output_path,
            merge,
            status: JobStatus::Queued,
        }
    }

    pub fn from_url(id: u64, url: String, target_format: String, output_dir: PathBuf) -> Self {
        Self {
            id,
            source: JobSource::Url(url),
            source_format: "youtube".to_string(),
            target_format,
            output_path: output_dir,
            merge: false,
            status: JobStatus::Queued,
        }
    }

    pub fn display_name(&self) -> String {
        match &self.source {
            JobSource::Files(paths) if paths.len() == 1 => {
                paths[0].file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string()
            }
            JobSource::Files(paths) => {
                format!("{} files -> {}", paths.len(), self.target_format.to_uppercase())
            }
            JobSource::Url(url) => {
                let short = if url.len() > 48 { format!("{}...", &url[..45]) } else { url.clone() };
                format!("YouTube: {}", short)
            }
        }
    }
}