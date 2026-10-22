#![recursion_limit = "256"]

mod config;
mod ignore;
mod index;
mod parser;
mod roblox;
mod types;
mod unity;
mod utils;
mod watcher;
mod writer;

use clap::Parser;
use colored::Colorize;
use config::load_config;
use ignore::load_loom_ignore;
use index::cross_ref::build_cross_reference_map;
use index::{load_index, save_index, upsert_file};
use parser::{get_language_for_extension, is_supported_file, parse_file};
use roblox::{build_roblox_model_note, is_roblox_project, scan_and_analyze_roblox};
use std::fs;
use std::path::{Path, PathBuf};
use types::FileIndex;
use unity::{build_unity_asset_note, is_unity_project, scan_and_analyze_unity};
use utils::file_path_to_note_id;
use watcher::start_watcher;
use writer::template::{build_index_note, build_note, build_symbol_note};
use writer::{ensure_vault_structure, write_note_if_changed};

#[derive(Parser, Debug)]
#[command(
    name = "loom",
    version,
    about = "Weave your codebase into a navigable Obsidian vault — one note per file, one note per symbol."
)]
struct Args {

    #[arg(default_value = ".")]
    project_path: PathBuf,

    #[arg(short, long)]
    name: Option<String>,

    #[arg(short, long)]
    watch: bool,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(short, long)]
    unity: bool,

    #[arg(short, long)]
    roblox: bool,

    #[arg(short, long)]
    verbose: bool,
}

fn collect_source_files(project_root: &Path, ignore: &ignore::LoomIgnore) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let walker = walkdir::WalkDir::new(project_root)
        .follow_links(false)
        .into_iter();

    for entry in walker.filter_entry(|e| {
        let relative = match e.path().strip_prefix(project_root) {
            Ok(r) => r,
            Err(_) => return false,
        };
        if relative.as_os_str().is_empty() {
            return true;
        }
        !ignore.is_ignored(relative, e.file_type().is_dir())
    }) {
        if let Ok(entry) = entry {
            if entry.file_type().is_file() && is_supported_file(entry.path()) {
                files.push(entry.into_path());
            }
        }
    }

    files.sort();
    files
}

fn run_once(
    project_root: &Path,
    vault_path: &Path,
    vault_dir_name: Option<&str>,
    unity_flag: bool,
    roblox_flag: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(60).dimmed());
    println!("{} {}", "Parsing project:".bold(), project_root.display());
    println!("{}   {}", "Vault location:".bold(), vault_path.display());

    let unity_mode = unity_flag || is_unity_project(project_root);
    if unity_mode {
        println!("{}      {}", "Unity Mode:".bold().yellow(), "ENABLED".green().bold());
    }

    let roblox_mode = roblox_flag || is_roblox_project(project_root);
    if roblox_mode {
        println!("{}     {}", "Roblox Mode:".bold().cyan(), "ENABLED".green().bold());
    }
    println!("{}", "=".repeat(60).dimmed());

    ensure_vault_structure(vault_path)?;

    let unity_analysis = if unity_mode {
        println!("\n{} Scanning Unity assets, prefabs, scenes, and .meta files...", "[unity]".yellow().bold());
        let analysis = scan_and_analyze_unity(project_root);
        println!(
            "{} Indexed {} Unity assets with {} script attachment links.\n",
            "✓".green(),
            analysis.assets.len().to_string().cyan().bold(),
            analysis.script_attachments.len().to_string().cyan().bold()
        );
        Some(analysis)
    } else {
        None
    };

    let roblox_analysis = if roblox_mode {
        println!("\n{} Scanning Roblox project, Rojo mappings, and .rbxmx models...", "[roblox]".cyan().bold());
        let analysis = scan_and_analyze_roblox(project_root);
        println!(
            "{} Mapped {} files to DataModel and indexed {} models with {} script attachments.\n",
            "✓".green(),
            analysis.file_to_datamodel.len().to_string().cyan().bold(),
            analysis.models.len().to_string().cyan().bold(),
            analysis.script_attachments.len().to_string().cyan().bold()
        );
        Some(analysis)
    } else {
        None
    };

    let loom_ignore = load_loom_ignore(project_root, vault_dir_name);
    let mut index = load_index(vault_path, project_root);

    let files = collect_source_files(project_root, &loom_ignore);
    println!("Found {} files to parse\n", files.len().to_string().cyan().bold());

    let mut files_processed = 0;
    let mut file_notes_written = 0;
    let mut file_notes_skipped = 0;
    let mut symbol_notes_written = 0;
    let mut symbol_notes_skipped = 0;
    let mut unity_notes_written = 0;
    let mut unity_notes_skipped = 0;
    let mut roblox_notes_written = 0;
    let mut roblox_notes_skipped = 0;

    println!("{}", "=== Pass 1: Parsing source files ===".bold());

    for absolute_path in &files {
        let relative_path = match absolute_path.strip_prefix(project_root) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => continue,
        };

        let contents = match fs::read_to_string(absolute_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{} Error reading {}: {e}", "✗".red(), relative_path);
                continue;
            }
        };

        let symbols = parse_file(&relative_path, &contents);
        let ext = Path::new(&relative_path)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();
        let language = get_language_for_extension(&ext).to_string();

        let existing_timestamp = index
            .files
            .iter()
            .find(|f| f.file_path == relative_path)
            .map(|f| f.last_parsed);

        let last_parsed = existing_timestamp.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0)
        });

        let file_index = FileIndex {
            file_path: relative_path.clone(),
            language,
            symbols,
            last_parsed,
        };

        upsert_file(&mut index, file_index);
        files_processed += 1;
        if verbose {
            println!("  [1/2] Parsed: {}", relative_path);
        }
    }
    println!("✓ Parsed {} source files.", files_processed);

    println!("\nBuilding cross-reference map...");
    let cross_refs = build_cross_reference_map(&index);
    println!("✓ Cross-reference map built.\n");

    println!("{}", "=== Pass 2: Writing notes ===".bold());

    for file_index in &index.files {
        let relative_path = &file_index.file_path;
        let imported_by = cross_refs.imported_by.get(relative_path).map(|v| v.as_slice());

        let unity_file_attachments = unity_analysis
            .as_ref()
            .and_then(|ua| ua.script_attachments.get(relative_path))
            .map(|v| v.as_slice());

        let roblox_dm = roblox_analysis
            .as_ref()
            .and_then(|ra| ra.file_to_datamodel.get(relative_path))
            .map(|s| s.as_str());

        let roblox_file_attachments = roblox_analysis
            .as_ref()
            .and_then(|ra| ra.script_attachments.get(relative_path))
            .map(|v| v.as_slice());

        let note_content = build_note(
            file_index,
            imported_by,
            unity_file_attachments,
            roblox_dm,
            roblox_file_attachments,
        );
        let note_filename = format!("{}.md", file_path_to_note_id(relative_path));
        let note_path = vault_path.join("notes").join(note_filename);

        match write_note_if_changed(&note_path, &note_content) {
            Ok(true) => {
                file_notes_written += 1;
                if verbose {
                    println!("  [2/2] Wrote note: {}", relative_path);
                }
            }
            Ok(false) => {
                file_notes_skipped += 1;
                if verbose {
                    println!("  [2/2] Skipped (unchanged): {}", relative_path);
                }
            }
            Err(e) => {
                eprintln!("{} Error writing note for {}: {e}", "✗".red(), relative_path);
            }
        }

        let mut seen_symbol_notes = std::collections::HashSet::new();
        for symbol in &file_index.symbols {
            if symbol.kind == types::SymbolKind::Import {
                continue;
            }

            let is_colliding = cross_refs.colliding_names.contains(&symbol.name);
            let note_id = utils::symbol_note_id_disambiguated(symbol, is_colliding);
            if !seen_symbol_notes.insert(note_id.clone()) {
                continue;
            }

            let called_by_refs = cross_refs.called_by.get(&symbol.name).map(|v| v.as_slice());
            let used_by_files = cross_refs.imported_by.get(relative_path).map(|v| v.as_slice());
            let referenced_by = cross_refs.referenced_by.get(&symbol.name).map(|v| v.as_slice());

            let unity_symbol_attachments = unity_analysis
                .as_ref()
                .and_then(|ua| ua.script_attachments.get(&symbol.name))
                .map(|v| v.as_slice());

            let roblox_symbol_attachments = roblox_analysis
                .as_ref()
                .and_then(|ra| ra.script_attachments.get(&symbol.name))
                .map(|v| v.as_slice());

            let symbol_note_content = build_symbol_note(
                symbol,
                &file_index.language,
                called_by_refs,
                Some(&cross_refs.callee_note_id),
                used_by_files,
                referenced_by,
                unity_symbol_attachments,
                roblox_symbol_attachments,
            );

            let symbol_filename = format!("{}.md", note_id);
            let symbol_note_path = vault_path.join("notes").join(symbol_filename);

            match write_note_if_changed(&symbol_note_path, &symbol_note_content) {
                Ok(true) => {
                    symbol_notes_written += 1;
                    if verbose {
                        println!("    ✓ Wrote symbol: {}", note_id);
                    }
                }
                Ok(false) => {
                    symbol_notes_skipped += 1;
                    if verbose {
                        println!("    - Skipped symbol: {} (unchanged)", note_id);
                    }
                }
                Err(e) => {
                    eprintln!("{} Error writing symbol note {}: {e}", "✗".red(), note_id);
                }
            }
        }
    }

    if let Some(ref ua) = unity_analysis {
        for asset in &ua.assets {
            let note_content = build_unity_asset_note(asset);
            let note_filename = format!("{}.md", file_path_to_note_id(&asset.file_path));
            let note_path = vault_path.join("notes").join(note_filename);

            match write_note_if_changed(&note_path, &note_content) {
                Ok(true) => {
                    unity_notes_written += 1;
                    if verbose {
                        println!("  [unity] Wrote asset note: {}", asset.file_path);
                    }
                }
                Ok(false) => {
                    unity_notes_skipped += 1;
                }
                Err(e) => {
                    eprintln!("{} Error writing Unity asset note {}: {e}", "✗".red(), asset.file_path);
                }
            }
        }
    }

    if let Some(ref ra) = roblox_analysis {
        for model in &ra.models {
            let note_content = build_roblox_model_note(model);
            let note_filename = format!("{}.md", file_path_to_note_id(&model.file_path));
            let note_path = vault_path.join("notes").join(note_filename);

            match write_note_if_changed(&note_path, &note_content) {
                Ok(true) => {
                    roblox_notes_written += 1;
                    if verbose {
                        println!("  [roblox] Wrote model note: {}", model.file_path);
                    }
                }
                Ok(false) => {
                    roblox_notes_skipped += 1;
                }
                Err(e) => {
                    eprintln!("{} Error writing Roblox model note {}: {e}", "✗".red(), model.file_path);
                }
            }
        }
    }

    let index_note_content = build_index_note(&index);
    let index_note_path = vault_path.join("notes").join("_Overview.md");
    let _ = write_note_if_changed(&index_note_path, &index_note_content);

    save_index(vault_path, &mut index)?;

    println!("\n{}", "=".repeat(60).dimmed());
    println!("{}", "Summary:".bold().green());
    println!("  Source files processed: {}", files_processed.to_string().cyan());
    println!("  File notes written:     {}", file_notes_written.to_string().green());
    println!("  File notes skipped:     {}", file_notes_skipped.to_string().dimmed());
    println!("  Symbol notes written:   {}", symbol_notes_written.to_string().green());
    println!("  Symbol notes skipped:   {}", symbol_notes_skipped.to_string().dimmed());
    if unity_mode {
        println!("  Unity asset notes:      {}", (unity_notes_written + unity_notes_skipped).to_string().yellow());
        println!("    Written:              {}", unity_notes_written.to_string().green());
        println!("    Skipped:              {}", unity_notes_skipped.to_string().dimmed());
    }
    if roblox_mode {
        println!("  Roblox model notes:     {}", (roblox_notes_written + roblox_notes_skipped).to_string().cyan());
        println!("    Written:              {}", roblox_notes_written.to_string().green());
        println!("    Skipped:              {}", roblox_notes_skipped.to_string().dimmed());
    }
    println!("{}", "=".repeat(60).dimmed());

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let project_root = fs::canonicalize(&args.project_path)?;
    let config = load_config(&project_root);

    let vault_dir_name = args
        .name
        .or_else(|| std::env::var("LOOM_VAULT_NAME").ok())
        .unwrap_or(config.vault.name);

    let vault_path = args
        .output
        .unwrap_or_else(|| project_root.join(&vault_dir_name));

    if args.watch {
        println!("{} Starting watch mode...", "[loom]".bold().cyan());

        run_once(&project_root, &vault_path, Some(&vault_dir_name), args.unity, args.roblox, args.verbose)?;

        let loom_ignore = load_loom_ignore(&project_root, Some(&vault_dir_name));
        start_watcher(
            &project_root,
            &config.watch,
            &loom_ignore,
            |_trigger| run_once(&project_root, &vault_path, Some(&vault_dir_name), args.unity, args.roblox, args.verbose),
        );
    } else {
        run_once(&project_root, &vault_path, Some(&vault_dir_name), args.unity, args.roblox, args.verbose)?;
    }

    Ok(())
}
