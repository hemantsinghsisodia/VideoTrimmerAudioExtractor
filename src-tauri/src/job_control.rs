use std::process::Child;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

pub const CANCELLED_BY_USER: &str = "Cancelled by user";

pub struct JobController {
    cancel_requested: AtomicBool,
    child: Mutex<Option<Child>>,
}

impl Default for JobController {
    fn default() -> Self {
        Self {
            cancel_requested: AtomicBool::new(false),
            child: Mutex::new(None),
        }
    }
}

impl JobController {
    pub fn begin_job(&self) {
        self.cancel_requested.store(false, Ordering::SeqCst);
        if let Ok(mut guard) = self.child.lock() {
            *guard = None;
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel_requested.load(Ordering::SeqCst)
    }

    pub fn set_child(&self, child: Child) {
        if let Ok(mut guard) = self.child.lock() {
            *guard = Some(child);
        }
    }

    pub fn clear_child(&self) {
        if let Ok(mut guard) = self.child.lock() {
            *guard = None;
        }
    }

    pub fn cancel(&self) -> bool {
        self.cancel_requested.store(true, Ordering::SeqCst);
        if let Ok(mut guard) = self.child.lock() {
            if let Some(mut child) = guard.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
        true
    }
}

pub fn wait_active_child(jobs: &JobController) -> Result<std::process::ExitStatus, String> {
    loop {
        if jobs.is_cancelled() {
            jobs.clear_child();
            return Err(CANCELLED_BY_USER.into());
        }

        let wait_result = {
            let mut guard = jobs
                .child
                .lock()
                .map_err(|e| format!("Job lock error: {e}"))?;
            let child = guard
                .as_mut()
                .ok_or_else(|| "No active process to wait on".to_string())?;
            child
                .try_wait()
                .map_err(|e| format!("Failed while waiting for process: {e}"))
        };

        match wait_result? {
            Some(status) => {
                jobs.clear_child();
                return Ok(status);
            }
            None => std::thread::sleep(Duration::from_millis(100)),
        }
    }
}
