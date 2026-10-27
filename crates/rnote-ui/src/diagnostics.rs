//! rnote-mod: performance diagnostics.
//!
//! - `CountingAllocator`: counts how many bytes the Rust side of the app currently holds.
//! - A logger that writes one CSV row every 15 seconds (memory, cached images, rendering work, ...)
//!   to a file in the log folder. The file contains numbers only: no note content.

use crate::RnAppWindow;
use gtk4::glib::{self, clone};
use gtk4::prelude::*;
use rnote_engine::diagnostics::Counters;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicIsize, AtomicU64, Ordering};

/// Label of this build, part of the log file name. Bump together with the installer label.
const MOD_VERSION: &str = "v1-6";
const INTERVAL_SECS: u32 = 15;
const HEADER: &str = "time,window_active,zoom,rust_heap_mb,allocs_per_s,strokes,strokes_trashed,strokes_holding_images,images,image_mb,rendernodes,background_tile_mb,background_rendernodes,history_len,viewport_scans,viewport_scan_ms,render_jobs_started,render_jobs_finished,render_cpu_ms,engine_tasks,engine_task_ms";

static LIVE_BYTES: AtomicIsize = AtomicIsize::new(0);
static ALLOC_CALLS: AtomicU64 = AtomicU64::new(0);

/// Wraps the system allocator and keeps count of the bytes in use.
#[derive(Debug)]
pub(crate) struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            LIVE_BYTES.fetch_add(layout.size() as isize, Ordering::Relaxed);
            ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc_zeroed(layout) };
        if !ptr.is_null() {
            LIVE_BYTES.fetch_add(layout.size() as isize, Ordering::Relaxed);
            ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        LIVE_BYTES.fetch_sub(layout.size() as isize, Ordering::Relaxed);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            LIVE_BYTES.fetch_add(new_size as isize - layout.size() as isize, Ordering::Relaxed);
            ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        }
        new_ptr
    }
}

/// The folder the log files are written to.
pub(crate) fn log_dir() -> PathBuf {
    glib::user_data_dir().join("rnote-mod").join("diagnostics")
}

#[derive(Debug, Default)]
struct LoggerState {
    file: Option<File>,
    previous_counters: Counters,
    previous_alloc_calls: u64,
}

/// Starts the timer that writes the log (when the setting "diagnostics-logging" is on).
pub(crate) fn start(appwindow: &RnAppWindow) {
    let state = Rc::new(RefCell::new(LoggerState::default()));

    let _ = glib::timeout_add_seconds_local(
        INTERVAL_SECS,
        clone!(
            #[weak]
            appwindow,
            #[strong]
            state,
            #[upgrade_or]
            glib::ControlFlow::Break,
            move || {
                log_tick(&appwindow, &mut state.borrow_mut());
                glib::ControlFlow::Continue
            }
        ),
    );
}

fn timestamp(format: &str) -> String {
    glib::DateTime::now_local()
        .ok()
        .and_then(|now| now.format(format).ok())
        .map_or_else(|| String::from("?"), |text| text.to_string())
}

/// Creates a new log file for this run, with the header row.
fn open_log_file() -> Option<File> {
    let dir = log_dir();
    std::fs::create_dir_all(&dir).ok()?;
    let path = dir.join(format!(
        "rnote-mod_{MOD_VERSION}_diagnostics_{}.csv",
        timestamp("%Y-%m-%d_%H-%M-%S")
    ));
    let mut file = File::create(path).ok()?;
    writeln!(file, "{HEADER}").ok()?;
    Some(file)
}

fn log_tick(appwindow: &RnAppWindow, state: &mut LoggerState) {
    let enabled = appwindow
        .app()
        .app_settings()
        .is_none_or(|settings| settings.boolean("diagnostics-logging"));
    if !enabled {
        // closing the file also makes the next switch-on start a new file
        state.file = None;
        return;
    }
    let Some(canvas) = appwindow.active_tab_canvas() else {
        return;
    };

    if state.file.is_none() {
        state.file = open_log_file();
        // the first row of a file shows the change since it was opened, not since the app started
        state.previous_counters = Counters::read();
        state.previous_alloc_calls = ALLOC_CALLS.load(Ordering::Relaxed);
    }
    let Some(file) = state.file.as_mut() else {
        return;
    };

    let engine_diagnostics = canvas.engine_ref().diagnostics();
    let store = engine_diagnostics.store;
    let counters = Counters::read();
    let change = counters.since(&state.previous_counters);
    let alloc_calls = ALLOC_CALLS.load(Ordering::Relaxed);
    let allocs_per_second =
        alloc_calls.saturating_sub(state.previous_alloc_calls) / u64::from(INTERVAL_SECS);
    state.previous_counters = counters;
    state.previous_alloc_calls = alloc_calls;

    let megabytes = |bytes: usize| bytes as f64 / 1_048_576.0;
    let milliseconds = |nanos: u64| nanos as f64 / 1_000_000.0;
    let heap_mb = LIVE_BYTES.load(Ordering::Relaxed) as f64 / 1_048_576.0;

    let row = format!(
        "{},{},{:.2},{:.1},{},{},{},{},{},{:.1},{},{:.1},{},{},{},{:.1},{},{},{:.1},{},{:.1}",
        timestamp("%Y-%m-%d %H:%M:%S"),
        u8::from(appwindow.is_active()),
        engine_diagnostics.zoom,
        heap_mb,
        allocs_per_second,
        store.strokes,
        store.strokes_trashed,
        store.strokes_holding_images,
        store.images,
        megabytes(store.image_bytes),
        store.rendernodes,
        megabytes(engine_diagnostics.background_tile_bytes),
        engine_diagnostics.background_rendernodes,
        store.history_len,
        change.viewport_scans,
        milliseconds(change.viewport_scan_nanos),
        change.render_jobs_started,
        change.render_jobs_finished,
        milliseconds(change.render_nanos),
        change.engine_tasks,
        milliseconds(change.engine_task_nanos),
    );
    let _ = writeln!(file, "{row}");
    let _ = file.flush();
}
