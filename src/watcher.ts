import chokidar from 'chokidar';
import { getSupportedExtensions } from './parser/index.js';
import type { LoomWatchConfig } from './config.js';

type RunFn = () => Promise<void>;

/**
 * Start watch mode for a project directory.
 *
 * Strategy (hybrid):
 *   1. chokidar watches all supported source files, filtered by the same
 *      isIgnored() predicate used by the CLI (derived from .loomignore).
 *   2. Any change resets a debounce timer. When the timer fires Loom runs.
 *   3. A guaranteed interval timer fires regardless of file-system events —
 *      safety net for missed events (network drives, some IDEs, etc.).
 *   4. If a run is already in progress the incoming trigger is queued so we
 *      never run two passes concurrently. At most one pending run is queued.
 */
export function startWatcher(
    projectRoot: string,
    config: LoomWatchConfig,
    isIgnored: (relativePath: string) => boolean,
    run: RunFn,
): void {
    const debounceMs = config.debounceSeconds * 1_000;
    const intervalMs = config.intervalMinutes * 60_000;

    const exts = getSupportedExtensions();

    let debounceTimer: ReturnType<typeof setTimeout> | null = null;
    let intervalTimer: ReturnType<typeof setInterval> | null = null;
    let running = false;
    let pendingRun = false;

    async function safeRun(trigger: string): Promise<void> {
        if (running) {
            pendingRun = true;
            return;
        }

        running = true;
        clearDebounce();
        clearInterval_();

        try {
            console.log(`\n[watch] Triggered by: ${trigger}`);
            await run();
        } catch (err) {
            console.error('[watch] Run failed:', (err as Error).message);
        } finally {
            running = false;
            resetInterval();

            if (pendingRun) {
                pendingRun = false;
                void safeRun('queued trigger');
            }
        }
    }

    function clearDebounce(): void {
        if (debounceTimer !== null) {
            clearTimeout(debounceTimer);
            debounceTimer = null;
        }
    }

    function clearInterval_(): void {
        if (intervalTimer !== null) {
            clearInterval(intervalTimer);
            intervalTimer = null;
        }
    }

    function resetInterval(): void {
        clearInterval_();
        intervalTimer = setInterval(() => {
            void safeRun(`interval (every ${config.intervalMinutes}m)`);
        }, intervalMs);
    }

    function scheduleDebounce(filePath: string): void {
        clearDebounce();
        debounceTimer = setTimeout(() => {
            void safeRun(`file change: ${filePath}`);
        }, debounceMs);
    }

    // Translate the loomignore predicate into the format chokidar expects.
    // chokidar passes absolute paths; we convert to relative before checking.
    const ignored = (absolutePath: string, _stats?: unknown): boolean => {
      // chokidar also calls this on the root itself — never ignore the root
      if (absolutePath === projectRoot) return false;
      const relativePath = absolutePath
        .slice(projectRoot.length)
        .replace(/^[\\/]/, '')   // strip leading separator
        .replace(/\\/g, '/');    // normalise to forward slashes
      if (!relativePath) return false;
      return isIgnored(relativePath);
    };

    const watcher = chokidar.watch(projectRoot, {
        ignored,
        persistent: true,
        ignoreInitial: true,
        followSymlinks: false,
        // Only react to extensions Loom can parse
        depth: Infinity,
    });

    const onFileEvent = (filePath: string): void => {
        const ext = filePath.slice(filePath.lastIndexOf('.'));
        if (!exts.includes(ext)) return;
        scheduleDebounce(filePath);
    };

    watcher.on('change', onFileEvent);
    watcher.on('add', onFileEvent);
    watcher.on('unlink', onFileEvent);

    watcher.on('error', (err: unknown) => {
        console.error('[watch] Watcher error:', (err as Error).message);
    });

    resetInterval();

    console.log(
        `[watch] Watching ${projectRoot}\n` +
        `        Debounce: ${config.debounceSeconds}s after last change\n` +
        `        Interval: every ${config.intervalMinutes}m (safety net)\n` +
        `        Press Ctrl+C to stop.\n`,
    );

    // Keep the process alive
    process.on('SIGINT', () => shutdown());
    process.on('SIGTERM', () => shutdown());

    function shutdown(): void {
        console.log('\n[watch] Shutting down...');
        clearDebounce();
        clearInterval_();
        void watcher.close().then(() => process.exit(0));
    }
}