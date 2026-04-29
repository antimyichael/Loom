import ignore from 'ignore';
type Ignore = ignore.Ignore;
import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'path';

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const LOOMIGNORE_FILENAME = '.loomignore';
const GITIGNORE_FILENAME = '.gitignore';

const LOOM_PREAMBLE = `# ---------------------------------------------------------------
# Loom-managed exclusions (added automatically)
# These are required for Loom to function correctly.
# You may add patterns below, but do not remove these.
# ---------------------------------------------------------------
.git/
.obsidian-index/
node_modules/

# ---------------------------------------------------------------
# Contents seeded from your project's root .gitignore
# (lines below were copied on first run — edit freely)
# ---------------------------------------------------------------
`;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

export interface LoomIgnore {
    /** Returns true when the given project-root-relative path should be skipped. */
    isIgnored: (relativePath: string) => boolean;
    /** Absolute path to the .loomignore file that was used. */
    loomIgnorePath: string;
}

export async function loadLoomIgnore(projectRoot: string): Promise<LoomIgnore> {
    const loomIgnorePath = join(projectRoot, LOOMIGNORE_FILENAME);

    let loomIgnoreExists = true;
    try {
        await readFile(loomIgnorePath, 'utf8');
    } catch (err: unknown) {
        if ((err as NodeJS.ErrnoException).code === 'ENOENT') {
            loomIgnoreExists = false;
        } else {
            throw err;
        }
    }

    if (!loomIgnoreExists) {
        await generateLoomIgnore(projectRoot, loomIgnorePath);
    }

    const raw = await readFile(loomIgnorePath, 'utf8');
    const ig = ignore().add(raw);

    return {
        loomIgnorePath,
        isIgnored: (relativePath: string) => {
            const normalised = relativePath.replace(/\\/g, '/');
            if (!normalised || normalised === '.') return false;
            return ig.ignores(normalised);
        },
    };
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

async function generateLoomIgnore(
    projectRoot: string,
    loomIgnorePath: string,
): Promise<void> {
    let gitignoreSection = '';

    try {
        const gitignoreRaw = await readFile(join(projectRoot, GITIGNORE_FILENAME), 'utf8');
        gitignoreSection = gitignoreRaw
            .split('\n')
            .map((line: string) => line.trimEnd())
            .join('\n');
    } catch (err: unknown) {
        if ((err as NodeJS.ErrnoException).code !== 'ENOENT') throw err;
        gitignoreSection = '# (no .gitignore found in project root)';
    }

    const content = LOOM_PREAMBLE + gitignoreSection + '\n';
    await writeFile(loomIgnorePath, content, 'utf8');

    console.log('✓ Generated .loomignore (seeded from .gitignore)');
}