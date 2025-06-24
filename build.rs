use std::collections::HashMap;
use std::fs;

/// Key mob_type to path to access the mob data
fn main() -> anyhow::Result<()> {
    let mob_manifest_path = "assets/mob_manifest.json5";
    let mut mob_data_path = HashMap::new();
    for entry in fs::read_dir("assets/sprites")? {
        let entry = entry?;
        // claude help this line pls trying to do the logic below but non-recursive
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        if let Some(extension) = path.extension() {
            if extension != "json5" {
                continue;
            }
            if let Some(file_name) = entry.file_name().to_str() {
                // fucking apple
                if file_name.starts_with("._") {
                    continue;
                }
                let data_str = std::fs::read_to_string(path).unwrap();
                let data: HashMap<String, serde_json::Value> = json5::from_str(&data_str)?;
                // zzzzz
                let mob_type = data
                    .get("sprite_type")
                    .unwrap()
                    .clone()
                    .as_u64()
                    .unwrap()
                    .to_string();
                mob_data_path.insert(mob_type, format!("sprites/{}", file_name.to_string()));
            }
        }
    }
    let out_data = json5::to_string(&mob_data_path)?;
    fs::write(mob_manifest_path, out_data)?;
    Ok(())
}
