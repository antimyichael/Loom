# Loom

> Weave your codebase into a navigable Obsidian vault.

Loom is a CLI tool that parses a programming project using Tree-sitter ASTs and generates an [Obsidian](https://obsidian.md) vault of interlinked Markdown notes. Every file, class, method, function, and variable becomes its own note, with `[[wikilinks]]` connecting everything together. The result is a fully navigable dependency graph of your entire codebase inside Obsidian's graph view.

---

## Supported Languages

| Language | Extensions |
|---|---|
| TypeScript / TSX | `.ts` `.tsx` |
| Python | `.py` |
| C# | `.cs` |
| Java | `.java` |
| Kotlin | `.kt` `.kts` |
| C++ | `.cpp` `.cc` `.cxx` `.h` `.hpp` |
| Bash / Shell | `.sh` `.bash` |

---

## Requirements

**Node.js 22 LTS is required.** Tree-sitter's native addons do not compile correctly on other versions. Node 25+ breaks native compilation; Node 19 causes ABI mismatches.

Install and manage Node versions with [nvm](https://github.com/nvm-sh/nvm) (macOS/Linux) or [nvm-windows](https://github.com/coreybutler/nvm-windows):

```bash
nvm install 22
nvm use 22
```

---

## Installation

### Global (recommended)

```bash
npm install -g @loom-code/loom --legacy-peer-deps
```

### Local (per-project)

```bash
npm install --save-dev @loom-code/loom --legacy-peer-deps
```

> **Note:** `--legacy-peer-deps` is required. Tree-sitter grammar packages have peer dependency constraints that conflict without it.

---

## Usage

### One-shot mode

Parse a project and generate the vault once:

```bash
loom /path/to/your/project

# Examples
loom ~/projects/my-typescript-app
loom ~/unity-projects/MyGame/Assets
loom .
```

### Watch mode

Keep the vault continuously updated as you edit files:

```bash
loom /path/to/your/project --watch
```

In watch mode, Loom runs immediately on start and then re-runs whenever a supported source file changes. Two layers of triggering are used: a debounce timer that fires after a quiet period following the last change, and a guaranteed interval that fires regardless of file-system events (a safety net for network drives or editors that don't emit standard events). At most one run is queued at a time — concurrent runs are never started.

Watch behaviour is configurable via `loom.config.json` in the project root (see [Configuration](#configuration)).

---

Loom creates a vault at `{projectPath}/.obsidian-index/`. Open that folder in Obsidian to explore your codebase as a graph. Add `.obsidian-index/` to your project's `.gitignore` — vaults are local to each developer.

---

## Ignoring Files

On first run, Loom generates a `.loomignore` file in your project root, seeded from your existing `.gitignore`. It uses the same pattern syntax as `.gitignore` (globs, `**`, negation with `!`, directory-only patterns with trailing `/`).

The generated file has two clearly labelled sections:

```
# Loom-required exclusions (added automatically)
.git/
.obsidian-index/
node_modules/

# Contents seeded from your project's root .gitignore
# (edit freely)
...
```

Edit the second section freely. Do not remove the Loom-required exclusions at the top. Add `.loomignore` itself to your project's `.gitignore` so each developer gets their own.

---

## Configuration

Loom reads an optional `loom.config.json` from the project root. Copy `loom.config.example.json` as a starting point:

```json
{
  "watch": {
    "intervalMinutes": 2,
    "debounceSeconds": 5
  }
}
```

| Key | Default | Description |
|---|---|---|
| `watch.intervalMinutes` | `2` | How often the guaranteed interval timer fires in watch mode |
| `watch.debounceSeconds` | `5` | Seconds of inactivity after a file change before Loom runs |

If `loom.config.json` is absent or a field is invalid, defaults are used silently.

---

## How It Works

Loom runs in two passes:

**Pass 1 — Parse:** Walks every supported file in the project, extracts all symbols (classes, methods, functions, variables, imports) using Tree-sitter ASTs, and builds a project-wide index.

**Pass 2 — Write:** Resolves cross-file references from the full index, then writes one Markdown note per file and one per significant symbol. Notes are only written when their content has changed (SHA-256 hash diff), so Obsidian only reloads what actually changed.

The two-pass design is required: a file note needs to know which other files import it, and that information is only available after all files have been parsed.

### Note structure

**File note**:
- Lists all classes, functions, variables, and imports it contains
- `## Imported by` — two-way link back to every file that imports this one

**Symbol note**:
- Shows the symbol's source code
- `## Calls` — links to every symbol it calls
- `## Called by` — two-way link back to every symbol that calls it
- `## Used by` — files that import the file this symbol lives in
- `## Referenced by` — symbols whose bodies reference this variable (variables only)

Method notes are named `ClassName.MethodName.md` to avoid collisions when multiple classes define methods with the same name (common in Unity C# and Android Kotlin projects).

---

## Contributing

Contributions, bug reports, and language support improvements are very welcome.

```bash
# Clone the repo
git clone https://github.com/antimyichael/loom.git
cd loom

# Install dependencies (--legacy-peer-deps is required)
npm install --legacy-peer-deps

# Run directly from source during development
npm run loom -- /path/to/test/project

# Or use the dev alias (equivalent)
npm run dev -- /path/to/test/project

# Build compiled output
npm run build

# Run the compiled output directly
node dist/cli.js /path/to/test/project
```

### Key constraints (please read before contributing)

- **Node.js 22 LTS only** — Node 25+ breaks native compilation; Node 19 causes ABI mismatches
- **`tree-sitter@0.21.1` exactly** — breaking ABI change in 0.25.x; do not upgrade
- **All grammar packages are pinned to exact versions** — do not add `^` carets
- **All npm installs must use `--legacy-peer-deps`**
- **No recursion in AST traversal** — all traversal uses explicit iterative stacks to prevent stack overflow on large files
- **`filePathToNoteId()` must be used everywhere** a file note filename is constructed; never inline the path-separator replacement
- **Method notes use `ClassName.MethodName.md`** format to avoid name collisions
- **Do not merge the two passes** in `cli.ts` into one — cross-file import resolution requires the full index to be built first

### Pinned grammar versions

| Package | Version |
|---|---|
| `tree-sitter` | `0.21.1` |
| `tree-sitter-typescript` | `0.21.2` |
| `tree-sitter-python` | `0.21.0` |
| `tree-sitter-bash` | `0.21.0` |
| `tree-sitter-kotlin` | `0.3.8` |
| `tree-sitter-cpp` | `0.22.3` |
| `tree-sitter-c-sharp` | `0.21.1` |
| `tree-sitter-java` | `0.21.0` |

---

## Roadmap
- **Phase 4 - VS Code Extension** Extension version of Loom for VS Code and VS Code forks, VS Codium, Eclipse Theia, and similar editors
- **Phase 5 — JetBrains Plugin:** IntelliJ/Rider/Android Studio plugin in Kotlin, mirroring the VS Code extension architecture
- **Phase 6 — Obsidian Plugin:** In-vault companion with a manual refresh button and last-parsed status in the sidebar
- **Phase 7 — Incremental Parsing:** Only reparse files that have changed since the last run, using `lastParsed` timestamps from the symbol index

---

## License

All Rights Reserved

DataThorn Technologies, @antimyichael, @giusiam