use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    zkpo::sp1::build(
        &["noop".into(), "engine".into()],
        &["zk".into()],
        true,
        Some(&PathBuf::from("elf/")),
    )
}
