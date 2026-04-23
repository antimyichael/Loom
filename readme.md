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

**Node.js 22 LTS is required.** Tree-sitter's native addons do not compile correctly on other versions.

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

```bash
# Run against any project directory
loom /path/to/your/project

# Examples
loom ~/projects/my-typescript-app
loom ~/unity-projects/MyGame/Assets
loom .
```

Loom will create a vault at `{projectPath}/.obsidian-index/`. Open that folder in Obsidian to explore your codebase graph.

---

## How It Works

Loom runs in two passes:

**Pass 1 — Parse:** Walks every supported file in the project, extracts all symbols (classes, methods, functions, variables, imports) using Tree-sitter ASTs, and builds a project-wide index.

**Pass 2 — Write:** Resolves cross-file references, then writes one Markdown note per file and one per significant symbol. Notes are only written if their content has changed (SHA-256 hash diff), so Obsidian only reloads what actually changed.

### Note structure

**File note** (`src__types.ts.md`):
- Lists all classes, functions, variables, and imports it contains
- `## Imported by` — two-way link back to every file that imports it

**Symbol note** (`Corgi.Update.md`):
- Shows the symbol's source code
- `## Calls` — links to every symbol it calls
- `## Called by` — two-way link back to every symbol that calls it
- `## Used by` — files that import the file this symbol lives in
- `## Referenced by` — symbols whose bodies reference this variable (variables only)

---

## VS Code Extension

A VS Code extension is in development that automatically runs Loom on every file save, keeping your vault up to date as you code. See the `loom-vscode/` directory.

---

## Contributing

Contributions, bug reports, and language support improvements are very welcome.

```bash
# Clone the repo
git clone https://github.com/your-org/loom.git
cd loom

# Install dependencies (--legacy-peer-deps is required)
npm install --legacy-peer-deps

# Run directly from source during development
npm run dev /path/to/test/project

# Build compiled output
npm run build
```

### Project structure

```
src/
├── cli.ts                      # Entry point — two-pass orchestrator
├── types.ts                    # Shared interfaces
├── utils.ts                    # filePathToNoteId() helper
├── parser/
│   ├── index.ts                # Parser router by file extension
│   ├── extractSymbols.ts       # TypeScript / TSX
│   ├── extractSymbolsPython.ts
│   ├── extractSymbolsCSharp.ts
│   ├── extractSymbolsJava.ts
│   ├── extractSymbolsKotlin.ts
│   ├── extractSymbolsCpp.ts
│   └── extractSymbolsBash.ts
├── writer/
│   ├── noteTemplate.ts         # Builds Markdown note content
│   └── diffWriter.ts           # Incremental write with SHA-256 hash diff
└── index/
    └── symbolIndex.ts          # Symbol index + cross-reference map builder
```

### Key constraints (please read before contributing)

- **Node.js 22 LTS only** — do not upgrade to Node 25+, native compilation breaks
- **tree-sitter@0.21.1** — do not upgrade; breaking ABI change in 0.25.x
- **All grammar packages are pinned exactly** — do not add `^` carets
- **All npm installs must use `--legacy-peer-deps`**
- **No recursion in AST traversal** — all traversal uses explicit iterative stacks to prevent stack overflow on large files
- **`filePathToNoteId()` must be used everywhere** a file note filename is constructed
- **Method notes use `ClassName.MethodName.md`** format to avoid name collisions

---
## LICENSE

All Rights Reserved

* DataThorn Technologies, @antimyichael, @giusiam