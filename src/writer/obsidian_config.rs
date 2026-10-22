use std::fs;
use std::path::Path;

pub fn write_obsidian_graph_config(vault_path: &Path) -> std::io::Result<()> {
    let obsidian_dir = vault_path.join(".obsidian");
    fs::create_dir_all(&obsidian_dir)?;

    let graph_json_path = obsidian_dir.join("graph.json");
    if graph_json_path.exists() {
        return Ok(());
    }

    let default_graph = serde_json::json!({
        "collapse-filter": false,
        "search": "",
        "localJumps": 1,
        "localBacklinks": true,
        "localForelinks": true,
        "localInterlinks": false,
        "showTags": false,
        "showAttachments": false,
        "hideUnresolved": false,
        "collapse-color-groups": false,
        "colorGroups": [
            { "query": "tag:#class", "color": { "a": 1, "rgb": 3848700 } },
            { "query": "tag:#function", "color": { "a": 1, "rgb": 3918506 } },
            { "query": "tag:#method", "color": { "a": 1, "rgb": 15822394 } },
            { "query": "tag:#variable", "color": { "a": 1, "rgb": 15764064 } },
            { "query": "tag:#overview", "color": { "a": 1, "rgb": 16738660 } },
            { "query": "tag:#unity/prefab", "color": { "a": 1, "rgb": 3402447 } },
            { "query": "tag:#unity/scene", "color": { "a": 1, "rgb": 16766720 } },
            { "query": "tag:#unity", "color": { "a": 1, "rgb": 5885918 } }
        ],
        "collapse-display": false,
        "showArrow": true,
        "textFadeMultiplier": 0,
        "nodeSizeMultiplier": 1.25,
        "lineSizeMultiplier": 1
    });

    fs::write(
        graph_json_path,
        serde_json::to_string_pretty(&default_graph).unwrap(),
    )?;

    Ok(())
}
