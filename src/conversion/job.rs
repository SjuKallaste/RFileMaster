use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    Queued,
    Running(f32),
    Done(PathBuf),
    Failed(String),
}

impl JobStatus {
    pub fn label(&self) -> &str {
        match self {
            JobStatus::Queued => "Queued",
            JobStatus::Running(_) => "Converting",
            JobStatus::Done(_) => "Done",
            JobStatus::Failed(_) => "Failed",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, JobStatus::Done(_) | JobStatus::Failed(_))
    }
}

#[derive(Debug, Clone)]
pub struct ConversionJob {
    pub id: u64,
    pub input_path: PathBuf,
    pub source_format: String,
    pub target_format: String,
    pub output_path: Option<PathBuf>,
    pub status: JobStatus,
}

impl ConversionJob {
    pub fn new(id: u64, input_path: PathBuf, source_format: String, target_format: String, output_path: Option<PathBuf>) -> Self {
        Self {
            id,
            input_path,
            source_format,
            target_format,
            output_path,
            status: JobStatus::Queued,
        }
    }

    pub fn file_name(&self) -> String {
        self.input_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    }
}
