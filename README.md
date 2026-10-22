# Loom

> Weave your codebase into a navigable Obsidian vault.

Loom is a fast CLI tool that parses codebases using Tree-sitter ASTs and generates an [Obsidian](https://obsidian.md) vault of interlinked Markdown notes. Every file, class, method, function, and variable becomes its own note, with `[[wikilinks]]` connecting everything together. The result is a fully navigable dependency graph of your entire codebase inside Obsidian's graph view.

---

## Supported Languages (50+ Languages)

Loom features a **two-tier polyglot parsing engine** combining deep Tree-sitter AST parsers for major languages with a high-performance universal grammar engine for 35+ specialized and domain languages.

| Category | Languages & Extensions |
|---|---|
| **Web & Scripting** | **TypeScript / TSX** (`.ts`, `.tsx`), **JavaScript / JSX** (`.js`, `.jsx`, `.mjs`, `.cjs`), **Python** (`.py`, `.pyw`), **PHP** (`.php`), **Ruby** (`.rb`), **Lua** (`.lua`), **Perl** (`.pl`, `.pm`), **Groovy** (`.groovy`), **Dart** (`.dart`) |
| **Systems & Native** | **Rust** (`.rs`), **Go** (`.go`), **C++** (`.cpp`, `.cc`, `.cxx`, `.hpp`), **C** (`.c`, `.h`), **Zig** (`.zig`), **Nim** (`.nim`), **D** (`.d`), **V / Vlang** (`.v`), **Crystal** (`.cr`) |
| **Enterprise & Mobile** | **C#** (`.cs`), **Java** (`.java`), **Kotlin** (`.kt`, `.kts`), **Swift** (`.swift`), **Scala** (`.scala`, `.sc`), **Apex** (`.cls`) |
| **Functional** | **Haskell** (`.hs`), **Elixir** (`.ex`, `.exs`), **Erlang** (`.erl`, `.hrl`), **OCaml** (`.ml`, `.mli`), **F#** (`.fs`, `.fsi`), **Clojure** (`.clj`, `.cljs`), **Elm** (`.elm`), **Scheme / Racket** (`.scm`, `.rkt`) |
| **Data & Scientific** | **SQL** (`.sql`), **R** (`.r`, `.R`), **Julia** (`.jl`), **Fortran** (`.f90`, `.f`, `.for`) |
| **Game Dev & Shaders** | **Luau / Roblox** (`.luau`, `.server.luau`, `.client.luau`, `.lua`, `.server.lua`, `.client.lua`), **GDScript / Godot** (`.gd`), **WGSL / GLSL / HLSL** (`.wgsl`, `.glsl`, `.hlsl`) |
| **Shell & Automation** | **Bash / Zsh / Shell** (`.sh`, `.bash`, `.zsh`), **PowerShell** (`.ps1`, `.psm1`) |
| **Infrastructure & APIs**| **Solidity / Smart Contracts** (`.sol`), **Terraform / HCL** (`.tf`, `.hcl`), **Protocol Buffers** (`.proto`), **GraphQL** (`.graphql`, `.gql`), **Pascal / Delphi** (`.pas`), **Ada** (`.adb`) |

---

## Desktop Application (macOS, Windows, Linux)

Loom is packaged as a standalone native desktop application with full graphical UI — **no terminal, Node.js, or npm required**:

| Platform | Installer Package | Format |
|---|---|---|
| **macOS** | `Loom-0.2.0-arm64.dmg` (Apple Silicon)<br>`Loom-0.2.0-x64.dmg` (Intel) | Native Apple Disk Image (`.dmg`) & `.zip` |
| **Windows** | `Loom-Setup-0.2.0.exe` | Standard Windows NSIS Installer & Portable `.exe` |
| **Linux** | `Loom-0.2.0.AppImage`<br>`loom_0.2.0_amd64.deb` | Universal AppImage & Debian/Ubuntu Package |

You can download the latest native installers directly from [GitHub Releases](https://github.com/antimyichael/Loom/releases).

### Desktop App Features
- **Zero Configuration**: Double-click to launch like any native app. The Rust parsing engine is bundled inside.
- **Drag & Drop Folder Selection**: Drag any codebase folder directly into the window or use the native OS file picker.
- **Custom Vault Naming**: Choose any folder name for your Obsidian vault (defaults to visible `LoomVault`).
- **Automatic Unity Detection**: Badges and configures Unity mode when a Unity repository (containing `Assets/`) is loaded.
- **Interactive Toggles**: One-click switches for **Unity Mode**, **Watch Mode**, and **Verbose Logging**.
- **Real-Time Terminal Console**: Streams live parser output and updates summary statistics in real time.
- **One-Click "Open in Obsidian"**: Automatically launches Obsidian with the newly woven vault once indexing finishes.

### Building Native Installers Locally

You can also build native installers directly on your machine:

```bash
# Build macOS .dmg & .app
npm run dist:mac

# Build Windows .exe installer
npm run dist:win

# Build Linux .AppImage & .deb
npm run dist:linux
```

---

## Installation

Loom is built in Rust as a self-contained, high-performance static binary with **zero external dependencies** — no Python, no Node.js, and no C++ compiler toolchains required on the user's machine.

### 1. One-Line Shell Installer (macOS & Linux)

```bash
curl -fsSL https://raw.githubusercontent.com/antimyichael/Loom/main/install.sh | sh
```

### 2. NPM / NPX (Zero Compilation)

If you already use Node or NPM, Loom is available with precompiled binaries:

```bash
# Run directly without installing
npx @loom-code/loom .

# Or install globally
npm install -g @loom-code/loom
```

### 3. Cargo (From Source)

```bash
cargo install --path .
```

### 4. Direct Binary Download

Prebuilt standalone binaries for **macOS (Apple Silicon & Intel)**, **Linux (x86_64 & aarch64)**, and **Windows (x86_64)** are available on the [GitHub Releases](https://github.com/antimyichael/Loom/releases) page.

---

## Usage

### One-shot mode

Parse a project and generate the vault once:

```bash
loom /path/to/your/project

# Examples
loom ~/projects/my-app
loom ~/unity-projects/MyGame/Assets
loom .
```

### Vault folder name & location

By default, Loom creates a clean, visible vault folder named **`LoomVault`** directly in the project root (`{projectPath}/LoomVault/`). This ensures users can easily view and select the vault in Obsidian without having to unhide dotfiles (`.obsidian-index`).

You can customize the vault name or path in three ways:

1. **Change the folder name** (within project root):
   ```bash
   loom . -n MyProjectDocs
   # or
   loom . --name CodebaseVault
   ```
2. **Specify a full custom output path**:
   ```bash
   loom . -o ~/Documents/ObsidianVaults/MyProject
   # or
   loom . --output /path/to/any/folder
   ```
3. **Configure in `loom.config.json` or Environment**:
   Set `"vault": { "name": "MyVault" }` in `loom.config.json` or export `LOOM_VAULT_NAME="MyVault"`.


### Unity project mode

Loom has built-in support for **Unity game projects** (via `--unity` or `-u`, auto-detected if `Assets/` is present). It scans `.meta` files and parses YAML hierarchies in `.prefab`, `.unity` (scene), and `.asset` (ScriptableObject) files:

```bash
loom ~/unity-projects/MyGame --unity
```

- **Hierarchy & Attachments**: Notes are generated for every Prefab and Scene detailing the full GameObject hierarchy, tags, parent-child trees, attached components, and attached C# scripts.
- **Bi-directional Script Links**: Every C# script note displays a **Unity Attachments** section listing exactly which GameObjects in which prefabs or scenes attach that script.
- **Obsidian Graph Integration**: Unity prefabs and scenes appear as dedicated color-coded nodes in Obsidian's graph view.

### Roblox / Luau project mode

Loom provides first-class support for **Roblox game projects** and **Luau** (via `--roblox` or `-r`, auto-detected if `default.project.json`, `*.project.json`, `wally.toml`, `.luau`, or `.rbxmx` are present):

```bash
loom ~/roblox-projects/MyGame --roblox
```

- **Luau Language Engine**: Parses gradual typing (`type` & `export type`), OOP tables (`setmetatable`), functions, typed methods (`Class:Method`), constants, and function calls.
- **Rojo DataModel Resolution**: Reads `default.project.json` and automatically maps files to their in-game DataModel tree (e.g. `game.ReplicatedStorage.Common.Utils`, `game.ServerScriptService.Combat.Hitbox`, `game.StarterPlayer.StarterPlayerScripts.Camera`).
- **Roblox Services & Requires**: Tracks `game:GetService(...)` calls (`Players`, `ReplicatedStorage`, `TweenService`) and instance-path requires (`require(ReplicatedStorage.Packages.Component)`).
- **Roblox XML Models (`.rbxmx` & `.rbxlx`)**: Reconstructs instance hierarchies (`Model`, `Part`, `Folder`, `RemoteEvent`, `ScreenGui`) and maps attached `Script`, `LocalScript`, and `ModuleScript` instances with bi-directional Obsidian links.
- **Graph View Integration**: Roblox models and Luau types receive dedicated color presets in Obsidian's graph view.

### Watch mode

Keep the vault continuously updated as you edit code:

```bash
loom /path/to/your/project --watch
```

In watch mode, Loom runs an initial pass and then monitors the project tree for file modifications. Changes trigger an incremental update after a configurable debounce period (default: 5s), backed by a safety-net interval timer.

---

## Features & Vault Design

- **Obsidian Graph View Styling**: Loom automatically configures `.obsidian/graph.json` with color presets so that classes (blue), functions (green), methods (purple), variables (orange), and overviews are clearly distinguished in Obsidian's graph view.
- **Clean Symbol Scoping**:
  - Methods are namespaced to their parent class (e.g. `[[ClassName.methodName]]`), preventing collisions across files.
  - Temporary loop indices and local function variables (like `i`, `stack`, `child`) are filtered out so they do not pollute the global graph.
- **Cross-Referencing**:
  - **Calls / Called by**: Direct call graph navigation.
  - **Imports / Imported by**: File-level dependency mapping.
  - **Referenced by**: Cross-file variable and constant usage tracking.
- **Incremental Hash-Based Diffing**: Only files and notes whose contents have actually changed (via SHA-256) are rewritten to disk, keeping Obsidian fast and preserving file metadata.
- **Codebase Dashboard**: Loom generates `_Overview.md` at the vault root summarizing all indexed files and symbol counts.

---

## Ignoring Files

On first run, Loom generates a `.loomignore` file in your project root, seeded from your `.gitignore`. It uses standard `.gitignore` pattern syntax.

```
# Loom-required exclusions (added automatically)
.git/
LoomVault/
.obsidian-index/
node_modules/
target/

# Contents seeded from your project's root .gitignore
...
```

Add your vault folder name (e.g. `LoomVault/`) and `.loomignore` to your project's `.gitignore`.

---

## Configuration

Loom reads an optional `loom.config.json` in the project root:

```json
{
  "vault": {
    "name": "LoomVault"
  },
  "watch": {
    "intervalMinutes": 2,
    "debounceSeconds": 5
  }
}
```

---

## Development & Testing

```bash
# Run unit and integration tests
cargo test

# Build optimized release binary
cargo build --release
```

---

## License

ISC License.
