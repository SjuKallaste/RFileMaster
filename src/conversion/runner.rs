use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use crate::conversion::job::{ConversionJob, JobStatus};
use crate::conversion::engine;

enum JobUpdate {
    Done(u64, PathBuf),
    Failed(u64, String),
}

pub struct ConversionRunner {
    pub jobs: Arc<Mutex<Vec<ConversionJob>>>,
    next_id: u64,
    tx: Sender<JobUpdate>,
    rx: Receiver<JobUpdate>,
}

impl ConversionRunner {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            jobs: Arc::new(Mutex::new(Vec::new())),
            next_id: 1,
            tx,
            rx,
        }
    }

    pub fn enqueue(&mut self, inputs: Vec<PathBuf>, source: String, target: String, output_path: PathBuf, merge: bool) {
        let job = ConversionJob::new(self.next_id, inputs, source, target, output_path, merge);
        self.next_id += 1;
        let mut jobs = self.jobs.lock().unwrap();
        jobs.push(job);
    }

    pub fn tick(&mut self) {
        while let Ok(update) = self.rx.try_recv() {
            let mut jobs = self.jobs.lock().unwrap();
            match update {
                JobUpdate::Done(id, path) => {
                    if let Some(job) = jobs.iter_mut().find(|j| j.id == id) {
                        job.status = JobStatus::Done(path);
                    }
                }
                JobUpdate::Failed(id, err) => {
                    if let Some(job) = jobs.iter_mut().find(|j| j.id == id) {
                        job.status = JobStatus::Failed(err);
                    }
                }
            }
        }

        let queued_ids: Vec<u64> = {
            let jobs = self.jobs.lock().unwrap();
            jobs.iter()
                .filter(|j| j.status == JobStatus::Queued)
                .map(|j| j.id)
                .collect()
        };

        for id in queued_ids {
            let (inputs, source, target, output_path, merge) = {
                let mut jobs = self.jobs.lock().unwrap();
                if let Some(job) = jobs.iter_mut().find(|j| j.id == id) {
                    job.status = JobStatus::Running(0.0);
                    (job.input_paths.clone(), job.source_format.clone(), job.target_format.clone(), job.output_path.clone(), job.merge)
                } else {
                    continue;
                }
            };

            let tx = self.tx.clone();
            std::thread::spawn(move || {
                match engine::convert(&inputs, &source, &target, &output_path, merge) {
                    Ok(()) => { let _ = tx.send(JobUpdate::Done(id, output_path)); }
                    Err(e) => { let _ = tx.send(JobUpdate::Failed(id, e)); }
                }
            });
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