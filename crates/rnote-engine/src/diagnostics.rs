//! rnote-mod: counters and snapshots for the performance diagnostics log.
//!
//! The counters are plain atomics that are only increased (cheap, no locking). The UI reads them
//! every few seconds and logs the change since the previous reading.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Runs of the viewport render scan (it walks over every stroke of the document) ...
pub static VIEWPORT_SCANS: AtomicU64 = AtomicU64::new(0);
/// ... and the time they took on the calling thread.
pub static VIEWPORT_SCAN_NANOS: AtomicU64 = AtomicU64::new(0);
/// Stroke render jobs handed to the thread pool ...
pub static RENDER_JOBS_STARTED: AtomicU64 = AtomicU64::new(0);
/// ... jobs that are finished ...
pub static RENDER_JOBS_FINISHED: AtomicU64 = AtomicU64::new(0);
/// ... and the CPU time the jobs spent drawing the strokes.
pub static RENDER_NANOS: AtomicU64 = AtomicU64::new(0);
/// Engine tasks handled on the main thread (this is where finished images are put to use) ...
pub static ENGINE_TASKS: AtomicU64 = AtomicU64::new(0);
/// ... and the time that took.
pub static ENGINE_TASK_NANOS: AtomicU64 = AtomicU64::new(0);

pub fn bump(counter: &AtomicU64) {
    counter.fetch_add(1, Ordering::Relaxed);
}

pub fn add_elapsed(counter: &AtomicU64, since: Instant) {
    counter.fetch_add(
        u64::try_from(since.elapsed().as_nanos()).unwrap_or(u64::MAX),
        Ordering::Relaxed,
    );
}

/// A reading of all counters.
#[derive(Debug, Clone, Copy, Default)]
pub struct Counters {
    pub viewport_scans: u64,
    pub viewport_scan_nanos: u64,
    pub render_jobs_started: u64,
    pub render_jobs_finished: u64,
    pub render_nanos: u64,
    pub engine_tasks: u64,
    pub engine_task_nanos: u64,
}

impl Counters {
    pub fn read() -> Self {
        Self {
            viewport_scans: VIEWPORT_SCANS.load(Ordering::Relaxed),
            viewport_scan_nanos: VIEWPORT_SCAN_NANOS.load(Ordering::Relaxed),
            render_jobs_started: RENDER_JOBS_STARTED.load(Ordering::Relaxed),
            render_jobs_finished: RENDER_JOBS_FINISHED.load(Ordering::Relaxed),
            render_nanos: RENDER_NANOS.load(Ordering::Relaxed),
            engine_tasks: ENGINE_TASKS.load(Ordering::Relaxed),
            engine_task_nanos: ENGINE_TASK_NANOS.load(Ordering::Relaxed),
        }
    }

    /// The change since an earlier reading.
    pub fn since(&self, earlier: &Self) -> Self {
        Self {
            viewport_scans: self.viewport_scans.saturating_sub(earlier.viewport_scans),
            viewport_scan_nanos: self
                .viewport_scan_nanos
                .saturating_sub(earlier.viewport_scan_nanos),
            render_jobs_started: self
                .render_jobs_started
                .saturating_sub(earlier.render_jobs_started),
            render_jobs_finished: self
                .render_jobs_finished
                .saturating_sub(earlier.render_jobs_finished),
            render_nanos: self.render_nanos.saturating_sub(earlier.render_nanos),
            engine_tasks: self.engine_tasks.saturating_sub(earlier.engine_tasks),
            engine_task_nanos: self
                .engine_task_nanos
                .saturating_sub(earlier.engine_task_nanos),
        }
    }
}

/// What the stroke store holds.
#[derive(Debug, Clone, Copy, Default)]
pub struct StoreDiagnostics {
    pub strokes: usize,
    pub strokes_trashed: usize,
    pub strokes_holding_images: usize,
    pub images: usize,
    pub image_bytes: usize,
    pub rendernodes: usize,
    pub history_len: usize,
}

/// What the engine holds.
#[derive(Debug, Clone, Copy, Default)]
pub struct EngineDiagnostics {
    pub zoom: f64,
    pub store: StoreDiagnostics,
    pub background_tile_bytes: usize,
    pub background_rendernodes: usize,
}
