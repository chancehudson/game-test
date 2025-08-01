use std::path::PathBuf;

use sp1_build::BuildArgs;

fn main() -> anyhow::Result<()> {
    let target = std::env::var("CARGO_CFG_TARGET_OS")?;
    if target == "zkvm" {
        return Ok(());
    }
    // Add new zk binaries here
    let zk_bins = vec!["test".into(), "engine".into()];

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let build_args = BuildArgs {
        docker: false,
        binaries: zk_bins,
        no_default_features: true,
        output_directory: Some(
            PathBuf::from(manifest_dir)
                .join("bin/")
                .to_string_lossy()
                .to_string(),
        ),
        ..Default::default()
    };
    sp1_build::execute_build_program(&build_args, None)?;
    Ok(())
}
