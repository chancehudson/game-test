use std::path::PathBuf;

use sp1_build::BuildArgs;

fn main() -> anyhow::Result<()> {
    // if we're in a CI don't rebuild the elf files
    // use the committed ones
    let is_ci = std::env::var("CI").is_ok();
    let target = std::env::var("CARGO_CFG_TARGET_OS")?;
    if target == "zkvm" || is_ci {
        return Ok(());
    }
    // Add new zk binaries here
    let zk_bins = vec!["noop".into(), "engine".into()];

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let build_args = BuildArgs {
        docker: false,
        binaries: zk_bins,
        features: vec!["zk".into()],
        no_default_features: true,
        output_directory: Some(
            PathBuf::from(manifest_dir)
                .join("elf/")
                .to_string_lossy()
                .to_string(),
        ),
        ..Default::default()
    };
    sp1_build::execute_build_program(&build_args, None)?;
    Ok(())
}
