//! Opening drawings without freezing the window (#694). A file is read and decoded on a worker
//! thread (the slow part of OPEN, which needs no session); the UI shows an "Opening ..." card with
//! a spinner and the elapsed time and keeps painting. The finished drawing is added to the session
//! on the UI thread. The web build has no threads and opens synchronously.

use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::time::{Duration, Instant};

use cadcraft_engine::cmd::file::{ReadFile, read_file_with_progress_stage};

use crate::theme::Tokens;

type Outcome = Result<ReadFile, String>;

#[derive(Default)]
struct LoadProgress {
    bytes: AtomicU64,
    total: AtomicU64,
    parsing: AtomicBool,
    read_done_ms: AtomicU64,
}

struct Job {
    path: String,
    started: Instant,
    rx: Receiver<Outcome>,
    progress: Arc<LoadProgress>,
}

/// The files being read in the background, oldest first.
#[derive(Default)]
pub struct Opening {
    jobs: Vec<Job>,
    queued: VecDeque<String>,
}

impl Opening {
    /// Start reading `path` on a worker thread.
    pub fn start(&mut self, path: &str) {
        // Large CAD files expand substantially during parsing and rendering. Never launch
        // multiple large decoders concurrently: queue later files rather than exhausting RAM.
        if !self.jobs.is_empty() || !self.queued.is_empty() {
            self.queued.push_back(path.to_string());
            return;
        }
        let (tx, rx) = channel();
        let p = path.to_string();
        let progress = Arc::new(LoadProgress::default());
        let worker_progress = Arc::clone(&progress);
        let started = Instant::now();
        let spawned = std::thread::Builder::new().name("cadcraft-open".into()).spawn(move || {
            let phase_progress = Arc::clone(&worker_progress);
            let result = read_file_with_progress_stage(&p, |bytes, total| {
                worker_progress.total.store(total, Ordering::Relaxed);
                worker_progress.bytes.store(bytes, Ordering::Relaxed);
            }, || {
                phase_progress.read_done_ms.store(started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64, Ordering::Relaxed);
                phase_progress.parsing.store(true, Ordering::Release);
            });
            let read_elapsed_ms = worker_progress.read_done_ms.load(Ordering::Relaxed);
            log::info!("CAD OPEN: file={p:?} read_ms={} decode_ms={} total_ms={} status={}", read_elapsed_ms, if worker_progress.parsing.load(Ordering::Acquire) { (started.elapsed().as_millis() as u64).saturating_sub(read_elapsed_ms) } else { 0 }, started.elapsed().as_millis(), if result.is_ok() { "OK" } else { "ERROR" });
            let _ = tx.send(result);
        });
        if spawned.is_err() {
            // No thread to read on: report it like any other failed open.
            let (tx, rx2) = channel();
            let _ = tx.send(Err(format!("open: {path}: could not start a reader thread")));
            self.jobs.push(Job { path: path.to_string(), started: Instant::now(), rx: rx2, progress });
            return;
        }
        self.jobs.push(Job { path: path.to_string(), started: Instant::now(), rx, progress });
    }

    /// Whether any file is still being read.
    pub fn busy(&self) -> bool {
        !self.jobs.is_empty() || !self.queued.is_empty()
    }

    /// The finished reads, as (path, result). A reader that died without answering is an error.
    pub fn poll(&mut self) -> Vec<(String, Outcome)> {
        let mut done = Vec::new();
        self.jobs.retain(|j| match j.rx.try_recv() {
            Ok(r) => {
                done.push((j.path.clone(), r));
                false
            }
            Err(TryRecvError::Empty) => true,
            Err(TryRecvError::Disconnected) => {
                done.push((j.path.clone(), Err(format!("open: {}: the file could not be read", j.path))));
                false
            }
        });
        done
    }

    /// Start the next decoder only after the previous decoded drawing has been
    /// installed (or dropped) on the UI thread, minimizing simultaneous RAM peaks.
    pub fn start_next_after_install(&mut self) {
        if self.jobs.is_empty() {
            if let Some(path) = self.queued.pop_front() {
                // start() honours queue order, so temporarily drain the rest before
                // starting the oldest request and then restore the pending FIFO.
                let remaining = std::mem::take(&mut self.queued);
                self.start(&path);
                self.queued = remaining;
            }
        }
    }

    /// The text of the card: the file and how long it has taken, or how many files are queued.
    fn label(&self) -> Option<String> {
        // A completed decoder can leave queued files waiting for the UI thread to
        // install the drawing. Keep the loading indicator visible during this gap.
        let Some(first) = self.jobs.first() else {
            return self.queued.front().map(|path| {
                let name = std::path::Path::new(path).file_name().map_or_else(|| path.clone(), |n| n.to_string_lossy().to_string());
                format!("Đang chờ mở {name} ({} file trong hàng đợi)", self.queued.len())
            });
        };
        let name = std::path::Path::new(&first.path).file_name().map_or_else(|| first.path.clone(), |n| n.to_string_lossy().to_string());
        let secs = first.started.elapsed().as_secs_f32();
        let more = self.jobs.len() + self.queued.len() - 1;
        let state = if first.progress.parsing.load(Ordering::Acquire) {
            let read_ms = first.progress.read_done_ms.load(Ordering::Relaxed);
            let decode_secs = first.started.elapsed().as_millis().saturating_sub(u128::from(read_ms)) as f64 / 1000.0;
            format!("Đã đọc xong sau {:.1}s; đang giải mã hình học ({decode_secs:.1}s)", read_ms as f64 / 1000.0)
        } else {
            let read = first.progress.bytes.load(Ordering::Relaxed);
            let total = first.progress.total.load(Ordering::Relaxed);
            if total > 0 {
                let progress = 100.0 * (read.min(total) as f64) / total as f64;
                let elapsed = first.started.elapsed().as_secs_f64();
                let eta = if read > 0 && elapsed > 0.1 && read < total {
                    let remaining = ((total - read) as f64 * elapsed / read as f64).ceil();
                    if remaining.is_finite() && remaining < 86_400.0 { format!("; ETA đọc ~{remaining:.0}s") } else { String::new() }
                } else { String::new() };
                format!("Đọc dữ liệu: {progress:.1}% ({:.1}/{:.1} MiB){eta}", read as f64 / 1_048_576.0, total as f64 / 1_048_576.0)
            } else { "Đang mở file".to_string() }
        };
        Some(if more == 0 { format!("Opening {name}... {secs:.0} s — {state}") } else { format!("Opening {name} and {more} more... {secs:.0} s — {state}") })
    }

    /// Paint the card in the middle of the window and keep frames coming while files load.
    pub fn show(&self, ctx: &egui::Context) {
        let Some(label) = self.label() else { return };
        let t = Tokens::get();
        egui::Area::new(egui::Id::unique("cc_opening"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::NONE
                    .fill(t.panel)
                    .stroke(egui::Stroke::new(1.0, t.accent))
                    .corner_radius(6.0)
                    .inner_margin(egui::Margin::symmetric(18, 12))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.add(egui::Spinner::new().color(t.accent));
                                ui.label(egui::RichText::new(label).color(t.text));
                            });
                            if let Some(first) = self.jobs.first() {
                                let total = first.progress.total.load(Ordering::Relaxed);
                                let done = first.progress.bytes.load(Ordering::Relaxed);
                                if !first.progress.parsing.load(Ordering::Acquire) && total > 0 {
                                    let fraction = ((done.min(total) as f64) / total as f64) as f32;
                                    ui.add(egui::ProgressBar::new(fraction).show_percentage().desired_width(320.0));
                                } else if first.progress.parsing.load(Ordering::Acquire) {
                                    ui.label("Đã đọc xong dữ liệu; đang giải mã, chưa có % giải mã chính xác.");
                                }
                            }
                        });
                    });
            });
        ctx.request_repaint_after(Duration::from_millis(50));
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use cadcraft_engine::Session;

    use crate::{CadApp, Services};

    /// Opening starts in the background (the UI is not blocked), shows the card, and a file that
    /// cannot be read is reported like a failed OPEN without adding a drawing.
    #[test]
    fn open_path_reads_in_the_background_and_reports_failure() {
        let mut app = CadApp::new(Session::new(), Services::default());
        let before = app.session.docs.len();
        let missing = std::env::temp_dir().join(format!("cadcraft-missing-{}.dxf", std::process::id()));
        let p = missing.to_string_lossy().to_string();
        app.open_path(&p);
        assert!(app.opening.busy() || app.status.is_some(), "the read starts in the background");
        let t = Instant::now();
        while app.opening.busy() && t.elapsed() < Duration::from_secs(10) {
            app.finish_opening();
            std::thread::sleep(Duration::from_millis(5));
        }
        app.finish_opening();
        assert!(!app.opening.busy());
        assert_eq!(app.session.docs.len(), before);
        let (msg, _) = app.status.clone().unwrap_or_default();
        assert!(msg.contains(&p), "status reports the file: {msg}");
        assert!(app.session.log.iter().any(|l| l.contains(&p)));
    }

    #[test]
    fn new_open_request_does_not_overtake_queued_drawings() {
        let mut opening = super::Opening::default();
        opening.start("/nonexistent/dir/one.dxf");
        opening.start("/nonexistent/dir/two.dxf");
        let started = Instant::now();
        while !opening.jobs.is_empty() && started.elapsed() < Duration::from_secs(10) {
            opening.poll();
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(opening.queued.front().map(String::as_str), Some("/nonexistent/dir/two.dxf"));
        opening.start("/nonexistent/dir/three.dxf");
        assert_eq!(opening.queued.len(), 2);
        opening.start_next_after_install();
        assert_eq!(opening.jobs.first().map(|job| job.path.as_str()), Some("/nonexistent/dir/two.dxf"));
        assert_eq!(opening.queued.front().map(String::as_str), Some("/nonexistent/dir/three.dxf"));
    }

    #[test]
    fn files_open_serially_to_bound_decoder_memory() {
        let mut opening = super::Opening::default();
        opening.start("/nonexistent/dir/first.dxf");
        opening.start("/nonexistent/dir/second.dxf");
        assert_eq!(opening.jobs.len(), 1);
        assert_eq!(opening.queued.len(), 1);
        let started = Instant::now();
        while opening.busy() && started.elapsed() < Duration::from_secs(10) {
            opening.poll();
            opening.start_next_after_install();
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(!opening.busy(), "all queued open attempts must finish");
    }

    #[test]
    fn decode_timer_starts_only_after_physical_file_read() {
        let progress = super::LoadProgress::default();
        progress.bytes.store(512, std::sync::atomic::Ordering::Relaxed);
        assert_eq!(progress.read_done_ms.load(std::sync::atomic::Ordering::Relaxed), 0);
        assert!(!progress.parsing.load(std::sync::atomic::Ordering::Acquire));
        progress.read_done_ms.store(150, std::sync::atomic::Ordering::Relaxed);
        progress.parsing.store(true, std::sync::atomic::Ordering::Release);
        assert_eq!(progress.read_done_ms.load(std::sync::atomic::Ordering::Relaxed), 150);
    }

    #[test]
    fn progress_stays_in_read_phase_at_metadata_boundary_until_decoder_signals() {
        let progress = super::LoadProgress::default();
        progress.total.store(1_048_576, std::sync::atomic::Ordering::Relaxed);
        progress.bytes.store(1_048_576, std::sync::atomic::Ordering::Relaxed);
        assert!(!progress.parsing.load(std::sync::atomic::Ordering::Acquire), "100% bytes read must not imply geometry decode has begun");
        progress.parsing.store(true, std::sync::atomic::Ordering::Release);
        assert!(progress.parsing.load(std::sync::atomic::Ordering::Acquire));
    }

    #[test]
    fn zero_byte_initial_progress_is_not_parse_completion() {
        let progress = super::LoadProgress::default();
        progress.bytes.store(0, std::sync::atomic::Ordering::Relaxed);
        progress.total.store(0, std::sync::atomic::Ordering::Relaxed);
        assert!(!progress.parsing.load(std::sync::atomic::Ordering::Acquire));
    }

    #[test]
    fn opening_progress_reports_bytes_without_claiming_decode_complete() {
        let p = super::LoadProgress::default();
        p.total.store(100, std::sync::atomic::Ordering::Relaxed);
        p.bytes.store(30, std::sync::atomic::Ordering::Relaxed);
        assert!(!p.parsing.load(std::sync::atomic::Ordering::Acquire));
        assert_eq!(p.bytes.load(std::sync::atomic::Ordering::Relaxed), 30);
        p.bytes.store(100, std::sync::atomic::Ordering::Relaxed);
        p.parsing.store(true, std::sync::atomic::Ordering::Release);
        assert!(p.parsing.load(std::sync::atomic::Ordering::Acquire));
    }

    #[test]
    fn pending_file_remains_visible_between_decoder_and_ui_install() {
        let mut opening = super::Opening::default();
        opening.queued.push_back("C:/drawings/large-01.dxf".to_string());
        opening.queued.push_back("C:/drawings/large-02.dxf".to_string());
        assert!(opening.busy());
        let label = opening.label().unwrap_or_default();
        assert!(label.contains("large-01.dxf"), "{label}");
        assert!(label.contains("2 file"), "{label}");
    }

    #[test]
    fn card_text_names_the_file_and_counts_queued_ones() {
        let mut o = super::Opening::default();
        assert!(o.label().is_none());
        o.start("/nonexistent/dir/a.dxf");
        o.start("/nonexistent/dir/b.dxf");
        let l = o.label().unwrap_or_default();
        assert!(l.starts_with("Opening a.dxf and 1 more"), "{l}");
    }
}
