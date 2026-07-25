use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::conversion::job::{ConversionJob, JobStatus};
use crate::persistence;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRecord {
    pub display_name: String,
    pub source_format: String,
    pub target_format: String,
    pub succeeded: bool,
    pub output_path: Option<PathBuf>,
    pub error: Option<String>,
}

impl JobRecord {
    pub fn from_job(job: &ConversionJob) -> Option<Self> {
        let (succeeded, output_path, error) = match &job.status {
            JobStatus::Done(p) => (true, Some(p.clone()), None),
            JobStatus::Failed(e) => (false, None, Some(e.clone())),
            _ => return None,
        };
        Some(Self {
            display_name: job.display_name(),
            source_format: job.source_format.clone(),
            target_format: job.target_format.clone(),
            succeeded,
            output_path,
            error,
        })
    }
}

const HISTORY_FILE: &str = "history.json";
const MAX_HISTORY: usize = 50;

pub fn load() -> Vec<JobRecord> {
    persistence::load_json(HISTORY_FILE).unwrap_or_default()
}

pub fn save(records: &[JobRecord]) {
    let _ = persistence::save_json(HISTORY_FILE, records);
}

pub fn push(records: &mut Vec<JobRecord>, record: JobRecord) {
    records.insert(0, record);
    records.truncate(MAX_HISTORY);
    save(records);
}