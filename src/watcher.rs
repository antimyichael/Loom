use crate::config::LoomWatchConfig;
use crate::ignore::LoomIgnore;
use crate::parser::is_supported_file;
use colored::Colorize;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::{Duration, Instant};

pub fn start_watcher<F>(
    project_root: &Path,
    config: &LoomWatchConfig,
    ignore: &LoomIgnore,
    mut run_fn: F,
) where
    F: FnMut(&str) -> Result<(), Box<dyn std::error::Error>>,
{
    let debounce_dur = Duration::from_secs(config.debounce_seconds);
    let interval_dur = Duration::from_secs(config.interval_minutes * 60);

    let (tx, rx) = channel();
    let mut watcher = match RecommendedWatcher::new(tx, notify::Config::default()) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("{} Could not initialize file watcher: {e}", "✗".red());
            return;
        }
    };

    if let Err(e) = watcher.watch(project_root, RecursiveMode::Recursive) {
        eprintln!("{} Could not watch directory: {e}", "✗".red());
        return;
    }

    println!(
        "\n{} Watching {}\n        Debounce: {}s after last change\n        Interval: every {}m (safety net)\n        Press Ctrl+C to stop.\n",
        "[watch]".cyan().bold(),
        project_root.display(),
        config.debounce_seconds,
        config.interval_minutes
    );

    let mut last_interval_run = Instant::now();
    let mut debounce_deadline: Option<Instant> = None;
    let mut pending_trigger: Option<String> = None;

    loop {
        let timeout = if let Some(deadline) = debounce_deadline {
            let now = Instant::now();
            if now >= deadline {
                Duration::from_millis(0)
            } else {
                deadline - now
            }
        } else {
            let now = Instant::now();
            let elapsed = now.duration_since(last_interval_run);
            if elapsed >= interval_dur {
                Duration::from_millis(0)
            } else {
                (interval_dur - elapsed).min(Duration::from_secs(1))
            }
        };

        match rx.recv_timeout(timeout) {
            Ok(Ok(event)) => {
                if is_relevant_event(&event, project_root, ignore) {
                    let path_desc = event
                        .paths
                        .first()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "file change".to_string());
                    pending_trigger = Some(format!("file change: {}", path_desc));
                    debounce_deadline = Some(Instant::now() + debounce_dur);
                }
            }
            Ok(Err(err)) => {
                eprintln!("{} Watcher error: {err}", "⚠".yellow());
            }
            Err(RecvTimeoutError::Timeout) => {
                let now = Instant::now();

                if let Some(deadline) = debounce_deadline {
                    if now >= deadline {
                        let trigger = pending_trigger.take().unwrap_or_else(|| "file change".into());
                        debounce_deadline = None;
                        last_interval_run = now;
                        println!("\n{} Triggered by: {}", "[watch]".cyan(), trigger);
                        if let Err(e) = run_fn(&trigger) {
                            eprintln!("{} Run failed: {e}", "[watch]".red());
                        }
                    }
                } else if now.duration_since(last_interval_run) >= interval_dur {

                    last_interval_run = now;
                    let trigger = format!("interval (every {}m)", config.interval_minutes);
                    println!("\n{} Triggered by: {}", "[watch]".cyan(), trigger);
                    if let Err(e) = run_fn(&trigger) {
                        eprintln!("{} Run failed: {e}", "[watch]".red());
                    }
                }
            }
            Err(RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }
}

fn is_relevant_event(event: &Event, project_root: &Path, ignore: &LoomIgnore) -> bool {
    use notify::EventKind::*;
    match event.kind {
        Create(_) | Modify(_) | Remove(_) => {}
        _ => return false,
    }

    for path in &event.paths {
        if !is_supported_file(path) {
            continue;
        }

        let relative = path
            .strip_prefix(project_root)
            .unwrap_or(path);

        if !ignore.is_ignored(relative, path.is_dir()) {
            return true;
        }
    }

    false
}
