use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::conversion::job::{ConversionJob, JobStatus};

pub struct ConversionRunner {
    pub jobs: Arc<Mutex<Vec<ConversionJob>>>,
    next_id: u64,
}

impl ConversionRunner {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(Vec::new())),
            next_id: 1,
        }
    }

    pub fn enqueue(&mut self, input: PathBuf, source: String, target: String, output_dir: Option<PathBuf>) {
        let output = output_dir.map(|d| {
            let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
            d.join(format!("{}.{}", stem, target))
        });
        let job = ConversionJob::new(self.next_id, input, source, target, output);
        self.next_id += 1;
        let mut jobs = self.jobs.lock().unwrap();
        jobs.push(job);
    }

    pub fn tick(&self) {
        let mut jobs = self.jobs.lock().unwrap();
        for job in jobs.iter_mut() {
            match &job.status {
                JobStatus::Queued => {
                    job.status = JobStatus::Running(0.0);
                }
                JobStatus::Running(p) => {
                    let new_p = (p + 0.05).min(1.0);
                    if new_p >= 1.0 {
                        let out = job.output_path.clone().unwrap_or_else(|| {
                            let stem = job.input_path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
                            PathBuf::from(format!("{}.{}", stem, job.target_format))
                        });
                        job.status = JobStatus::Done(out);
                    } else {
                        job.status = JobStatus::Running(new_p);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn clear_finished(&self) {
        let mut jobs = self.jobs.lock().unwrap();
        jobs.retain(|j| !j.status.is_terminal());
    }

    pub fn remove(&self, id: u64) {
        let mut jobs = self.jobs.lock().unwrap();
        jobs.retain(|j| j.id != id);
    }
}
