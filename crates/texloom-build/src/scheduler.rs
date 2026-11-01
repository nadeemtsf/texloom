use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};

#[derive(Clone, Default)]
pub struct Cancellation(Arc<AtomicBool>);
impl Cancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
    pub fn check(&self) -> Result<(), String> {
        if self.cancelled() {
            Err("Compilation cancelled or superseded".into())
        } else {
            Ok(())
        }
    }
}
#[derive(Default)]
struct State {
    next: u64,
    running: Option<u64>,
    latest: Option<(u64, String, Cancellation, Option<u64>)>,
    fences: std::collections::BTreeMap<String, u64>,
}
#[derive(Default)]
struct Shared {
    state: Mutex<State>,
    changed: Condvar,
}
/// One active compiler, one newest request. A waiting older request never runs.
#[derive(Clone, Default)]
pub struct Scheduler(Arc<Shared>);
pub struct Ticket {
    shared: Arc<Shared>,
    pub id: u64,
    cancel: Cancellation,
}
pub struct Lease {
    shared: Arc<Shared>,
    pub id: u64,
    pub cancel: Cancellation,
}
impl Scheduler {
    pub fn request(&self, project: &str) -> Result<Ticket, String> {
        self.request_inner(project, None)
    }
    pub fn request_sequence(&self, project: &str, sequence: u64) -> Result<Ticket, String> {
        self.request_inner(project, Some(sequence))
    }
    fn request_inner(&self, project: &str, sequence: Option<u64>) -> Result<Ticket, String> {
        let mut s = self.0.state.lock().map_err(|e| e.to_string())?;
        if sequence.is_some_and(|seq| s.fences.get(project).is_some_and(|f| seq <= *f)) {
            return Err("Compilation already cancelled".into());
        }
        if let Some((_, _, cancel, _)) = &s.latest {
            cancel.cancel();
        }
        s.next = s.next.checked_add(1).ok_or("Build sequence exhausted")?;
        let id = s.next;
        let cancel = Cancellation::default();
        s.latest = Some((id, project.into(), cancel.clone(), sequence));
        self.0.changed.notify_all();
        Ok(Ticket {
            shared: self.0.clone(),
            id,
            cancel,
        })
    }
    pub fn cancel_project(&self, project: &str) {
        if let Ok(s) = self.0.state.lock() {
            if let Some((_, p, c, _)) = &s.latest {
                if p == project {
                    c.cancel();
                }
            }
            self.0.changed.notify_all();
        }
    }
    pub fn cancel_sequence(&self, project: &str, sequence: u64) {
        if let Ok(mut s) = self.0.state.lock() {
            let fence = s.fences.entry(project.into()).or_default();
            *fence = (*fence).max(sequence);
            if let Some((_, p, c, seq)) = &s.latest {
                if p == project && seq.is_none_or(|n| n <= sequence) {
                    c.cancel();
                }
            }
            self.0.changed.notify_all();
        }
    }
}
impl Ticket {
    pub fn acquire(self) -> Result<Lease, String> {
        let mut s = self.shared.state.lock().map_err(|e| e.to_string())?;
        loop {
            self.cancel.check()?;
            if s.latest.as_ref().map(|x| x.0) != Some(self.id) {
                return Err("Build superseded".into());
            }
            if s.running.is_none() {
                s.running = Some(self.id);
                break;
            }
            s = self.shared.changed.wait(s).map_err(|e| e.to_string())?;
        }
        drop(s);
        Ok(Lease {
            shared: self.shared,
            id: self.id,
            cancel: self.cancel,
        })
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        if let Ok(mut s) = self.shared.state.lock() {
            if s.running == Some(self.id) {
                s.running = None;
            }
            self.shared.changed.notify_all();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_newest_waiting_request_runs_and_active_job_gets_cancelled() {
        let scheduler = Scheduler::default();
        let active = scheduler.request("p").unwrap().acquire().unwrap();
        let old = scheduler.request("p").unwrap();
        let newest = scheduler.request("p").unwrap();
        assert!(active.cancel.cancelled());
        assert!(old.acquire().is_err());
        let worker = std::thread::spawn(move || newest.acquire().unwrap());
        drop(active);
        let next = worker.join().unwrap();
        assert!(!next.cancel.cancelled());
    }
    #[test]
    fn project_cancellation_does_not_cancel_another_project() {
        let s = Scheduler::default();
        let job = s.request("a").unwrap().acquire().unwrap();
        s.cancel_project("b");
        assert!(!job.cancel.cancelled());
        s.cancel_project("a");
        assert!(job.cancel.cancelled());
    }
    #[test]
    fn cancellation_fence_also_rejects_a_job_that_has_not_started_yet() {
        let s = Scheduler::default();
        s.cancel_sequence("p", 4);
        assert!(s.request_sequence("p", 4).is_err());
        let newer = s.request_sequence("p", 5).unwrap().acquire().unwrap();
        s.cancel_sequence("p", 4);
        assert!(!newer.cancel.cancelled());
    }
}
