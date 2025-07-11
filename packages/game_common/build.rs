use std::collections::HashMap;
use std::fs;

/// Combine all game data files into a single json5 file. Does not include
/// images or sound assets.
fn main() -> anyhow::Result<()> {
    let manifest_path = "assets/game_data.json5";
    let mut combined_data = HashMap::new();
    let paths = vec![
        ("items", "assets/items"),
        ("maps", "assets/maps"),
        ("mobs", "assets/mobs"),
        ("npc", "assets/npc"),
        ("players", "assets/player"),
    ];
    for (name, path) in paths {
        let mut datas = vec![];
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let path = entry.path();
            if let Some(extension) = path.extension() {
                if extension != "json5" {
                    continue;
                }
            } else {
                continue;
            }
            if let Some(file_name) = entry.file_name().to_str() {
                // fucking apple
                if file_name.starts_with("._") {
                    continue;
                }
                let data_str = std::fs::read_to_string(&path).unwrap();
                let data: HashMap<String, serde_json::Value> = json5::from_str(&data_str)?;
                // let id = data
                //     .get("id")
                //     .unwrap()
                //     .clone()
                //     .as_u64()
                //     .unwrap()
                //     .to_string();
                datas.push(data);
            }
        }
        let out = combined_data.insert(name, datas);
        assert!(out.is_none(), "duplicate data name!");
    }
    let out_data = json5::to_string(&combined_data)?;
    fs::write(manifest_path, out_data)?;
    Ok(())
}
