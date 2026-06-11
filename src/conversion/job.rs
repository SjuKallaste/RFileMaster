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
pub struct ConversionJob {
    pub id: u64,
    pub input_paths: Vec<PathBuf>,
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
            input_paths,
            source_format,
            target_format,
            output_path,
            merge,
            status: JobStatus::Queued,
        }
    }

    pub fn display_name(&self) -> String {
        if self.input_paths.len() == 1 {
            self.input_paths[0]
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        } else {
            format!("{} files -> {}", self.input_paths.len(), self.target_format.to_uppercase())
        }
    }
}