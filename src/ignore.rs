use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::fs;
use std::path::{Path, PathBuf};

const LOOMIGNORE_FILENAME: &str = ".loomignore";
const GITIGNORE_FILENAME: &str = ".gitignore";

const LOOM_PREAMBLE: &str = "# ---------------------------------------------------------------
# Loom-managed exclusions (added automatically)
# These are required for Loom to function correctly.
# You may add patterns below, but do not remove these.
# ---------------------------------------------------------------
.git/
.obsidian-index/
LoomVault/
node_modules/
target/

# ---------------------------------------------------------------
# Contents seeded from your project's root .gitignore
# (lines below were copied on first run — edit freely)
# ---------------------------------------------------------------
";

pub struct LoomIgnore {
    gitignore: Gitignore,
    pub vault_dir_name: Option<String>,
    #[allow(dead_code)]
    pub loom_ignore_path: PathBuf,
}

impl LoomIgnore {
    pub fn is_ignored(&self, relative_path: &Path, is_dir: bool) -> bool {
        for component in relative_path.components() {
            let s = component.as_os_str().to_string_lossy();
            if s == ".git"
                || s == ".obsidian-index"
                || s == "LoomVault"
                || s == "node_modules"
                || s == "target"
            {
                return true;
            }
            if let Some(ref vname) = self.vault_dir_name {
                if s == vname.as_str() {
                    return true;
                }
            }
        }

        let matched = self.gitignore.matched(relative_path, is_dir);
        matched.is_ignore()
    }
}

pub fn load_loom_ignore(project_root: &Path, vault_dir_name: Option<&str>) -> LoomIgnore {
    let loom_ignore_path = project_root.join(LOOMIGNORE_FILENAME);

    if !loom_ignore_path.exists() {
        generate_loom_ignore(project_root, &loom_ignore_path);
    }

    let mut builder = GitignoreBuilder::new(project_root);

    let _ = builder.add_line(None, ".git/");
    let _ = builder.add_line(None, ".obsidian-index/");
    let _ = builder.add_line(None, "LoomVault/");
    let _ = builder.add_line(None, "node_modules/");
    let _ = builder.add_line(None, "target/");

    if let Some(vname) = vault_dir_name {
        let pattern = format!("{}/", vname.trim_matches(['/', '\\']));
        let _ = builder.add_line(None, &pattern);
    }

    if loom_ignore_path.exists() {
        let _ = builder.add(&loom_ignore_path);
    }

    let gitignore = builder.build().unwrap_or_else(|_| Gitignore::empty());

    LoomIgnore {
        gitignore,
        vault_dir_name: vault_dir_name.map(|s| s.to_string()),
        loom_ignore_path,
    }
}

fn generate_loom_ignore(project_root: &Path, loom_ignore_path: &Path) {
    let gitignore_path = project_root.join(GITIGNORE_FILENAME);
    let gitignore_section = if gitignore_path.exists() {
        fs::read_to_string(&gitignore_path).unwrap_or_else(|_| "# (could not read .gitignore)".to_string())
    } else {
        "# (no .gitignore found in project root)".to_string()
    };

    let content = format!("{}{}\n", LOOM_PREAMBLE, gitignore_section.trim_end());
    if let Err(e) = fs::write(loom_ignore_path, content) {
        eprintln!("⚠ Could not generate .loomignore: {e}");
    } else {
        println!("✓ Generated .loomignore (seeded from .gitignore)");
    }
}
