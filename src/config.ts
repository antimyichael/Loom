import { readFile } from 'node:fs/promises';
import { join } from 'path';

export interface LoomWatchConfig {
    intervalMinutes: number;
    debounceSeconds: number;
}

export interface LoomConfig {
    watch: LoomWatchConfig;
}

const DEFAULTS: LoomConfig = {
    watch: {
        intervalMinutes: 2,
        debounceSeconds: 5,
    },
};

export async function loadConfig(projectRoot: string): Promise<LoomConfig> {
    const configPath = join(projectRoot, 'loom.config.json');

    try {
        const raw = await readFile(configPath, 'utf8');
        const parsed: unknown = JSON.parse(raw);

        if (typeof parsed !== 'object' || parsed === null) {
            console.warn('⚠ loom.config.json is not a valid object — using defaults.');
            return DEFAULTS;
        }

        const config = parsed as Record<string, unknown>;
        const watch = typeof config['watch'] === 'object' && config['watch'] !== null
            ? config['watch'] as Record<string, unknown>
            : {};

        const intervalMinutes = typeof watch['intervalMinutes'] === 'number' && watch['intervalMinutes'] > 0
            ? watch['intervalMinutes']
            : DEFAULTS.watch.intervalMinutes;

        const debounceSeconds = typeof watch['debounceSeconds'] === 'number' && watch['debounceSeconds'] >= 0
            ? watch['debounceSeconds']
            : DEFAULTS.watch.debounceSeconds;

        return { watch: { intervalMinutes, debounceSeconds } };
    } catch (err: unknown) {
        // File not found — silently use defaults
        if (typeof err === 'object' && err !== null && (err as NodeJS.ErrnoException).code === 'ENOENT') {
            return DEFAULTS;
        }
        console.warn(`⚠ Could not read loom.config.json: ${(err as Error).message} — using defaults.`);
        return DEFAULTS;
    }
}