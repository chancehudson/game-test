use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn install_risc0_toolchain() -> anyhow::Result<()> {
    println!("cargo:warning=Installing RISC Zero toolchain...");

    // Install the RISC Zero toolchain
    let install_output = Command::new("rustup")
        .args(&["toolchain", "install", "risc0"])
        .output()?;

    if !install_output.status.success() {
        eprintln!("Failed to install risc0 toolchain:");
        eprintln!("{}", String::from_utf8_lossy(&install_output.stderr));
        anyhow::bail!("Toolchain installation failed");
    }

    println!("cargo:warning=RISC Zero toolchain installed successfully");
    Ok(())
}

fn get_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| {
        // Fallback to common shells
        if cfg!(windows) {
            "cmd".to_string()
        } else {
            "/bin/sh".to_string()
        }
    })
}

fn main() -> anyhow::Result<()> {
    // RUSTFLAGS='
    //     --cfg getrandom_backend="custom" -C panic=abort
    //     --cfg portable_atomic_unsafe_assume_single_core
    //     -C target-feature=+crt-static
    // ' cargo +risc0 build \
    //     --target riscv32im-risc0-zkvm-elf\
    //     --release

    let target = std::env::var("CARGO_CFG_TARGET_OS")?;
    println!("cargo:warning={}", target);

    if target == "zkvm" {
        return Ok(());
    }

    println!("cargo:warning={}", "======================================");
    println!("cargo:warning={}", "======================================");
    println!("cargo:warning={}", "======================================");
    println!("cargo:warning={}", "======================================");
    println!("cargo:warning={}", "======================================");
    println!("cargo:warning={}", "======================================");

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    println!("cargo:warning={}", manifest_dir);

    let outdir = tempdir()?;
    let target_dir = outdir.path();
    let target_dir = PathBuf::from(manifest_dir.clone()).join("bin");
    println!("cargo:warning={:?}", target_dir);
    let bin_name = "test";
    // otherwise build all binaries and stash the .bin elf files
    let rust_flags = &[
        "--cfg=getrandom_backend=\"custom\"",
        "--cfg=portable_atomic_unsafe_assume_single_core",
        "-C target-feature=+crt-static",
    ];
    let out = Command::new(get_shell())
        .args(&[
            "-c",
            "cargo",
            &format!(
                "'{}'",
                &[
                    "+risc0",
                    "build",
                    &format!("--manifest-path={manifest_dir}/Cargo.toml"),
                    &format!("--target-dir={}", target_dir.to_str().unwrap()),
                    "--target=riscv32im-risc0-zkvm-elf",
                    "--release",
                ]
                .join(" "),
            ),
        ])
        .env("RUSTFLAGS", rust_flags.join(" "))
        .output()?;
    if !out.status.success() {
        println!("cargo:warning={:?}", out);
        println!("cargo:warning={}", "compilation failed!");
        std::process::exit(1);
    } else {
        println!("cargo:warning={:?}", out);
    }

    // fs::copy(
    //     PathBuf::from(target_dir).join(format!("riscv32im-risc0-zkvm-elf/release/{bin_name}")),
    //     PathBuf::from(manifest_dir).join(format!("bin/{bin_name}.riscv32im-risc0-zkvm-elf")),
    // )?;

    Ok(())
}
