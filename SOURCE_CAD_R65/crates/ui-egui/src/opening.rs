//! Opening drawings without freezing the window (#694). A file is read and decoded on a worker
//! thread (the slow part of OPEN, which needs no session); the UI shows an "Opening ..." card with
//! a spinner and the elapsed time and keeps painting. The finished drawing is added to the session
//! on the UI thread. The web build has no threads and opens synchronously.

use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::time::{Duration, Instant};

use cadcraft_engine::cmd::file::{ReadFile, read_file_with_progress};

use crate::theme::Tokens;

type Outcome = Result<ReadFile, String>;

#[derive(Default)]
struct LoadProgress {
    bytes: AtomicU64,
    total: AtomicU64,
    parsing: AtomicBool,
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
        if !self.jobs.is_empty() {
            self.queued.push_back(path.to_string());
            return;
        }
        let (tx, rx) = channel();
        let p = path.to_string();
        let progress = Arc::new(LoadProgress::default());
        let worker_progress = Arc::clone(&progress);
        let spawned = std::thread::Builder::new().name("cadcraft-open".into()).spawn(move || {
            let result = read_file_with_progress(&p, |bytes, total| {
                worker_progress.total.store(total, Ordering::Relaxed);
                worker_progress.bytes.store(bytes, Ordering::Relaxed);
                if bytes >= total { worker_progress.parsing.store(true, Ordering::Release); }
            });
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
        if self.jobs.is_empty() {
            if let Some(path) = self.queued.pop_front() { self.start(&path); }
        }
        done
    }

    /// The text of the card: the file and how long it has taken, or how many files are queued.
    fn label(&self) -> Option<String> {
        let first = self.jobs.first()?;
        let name = std::path::Path::new(&first.path).file_name().map_or_else(|| first.path.clone(), |n| n.to_string_lossy().to_string());
        let secs = first.started.elapsed().as_secs_f32();
        let more = self.jobs.len() + self.queued.len() - 1;
        let state = if first.progress.parsing.load(Ordering::Acquire) {
            "Đang giải mã hình học CAD".to_string()
        } else {
            let read = first.progress.bytes.load(Ordering::Relaxed);
            let total = first.progress.total.load(Ordering::Relaxed);
            if total > 0 { format!("Đọc dữ liệu: {:.1}% ({:.1}/{:.1} MiB)", 100.0 * (read.min(total) as f64) / total as f64, read as f64 / 1_048_576.0, total as f64 / 1_048_576.0) }
            else { "Đang mở file".to_string() }
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
    fn files_open_serially_to_bound_decoder_memory() {
        let mut opening = super::Opening::default();
        opening.start("/nonexistent/dir/first.dxf");
        opening.start("/nonexistent/dir/second.dxf");
        assert_eq!(opening.jobs.len(), 1);
        assert_eq!(opening.queued.len(), 1);
        let started = Instant::now();
        while opening.busy() && started.elapsed() < Duration::from_secs(10) {
            opening.poll();
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(!opening.busy(), "all queued open attempts must finish");
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
