//! Hello world example for Rust on GPU.

#![forbid(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use std::io::{self, Write};
use std::process::Command;

fn main() -> anyhow::Result<()> {
    if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        build_metal()
    } else {
        Ok(())
    }
}

fn build_metal() -> anyhow::Result<()> {
    println!("Compiling src/metal/dotprod.metal into the Apple intermediate language");

    let output = Command::new("xcrun")
        .args(
            "-sdk macosx metal -c src/metal/dotprod.metal -o target/dotprod.air"
                .split_ascii_whitespace(),
        )
        .output()?;

    println!("status: {}", output.status);
    io::stdout().write_all(&output.stdout)?;
    io::stderr().write_all(&output.stderr)?;

    println!("Compiling src/metal/dotprod.air into our compiled MSL library");

    let output = Command::new("xcrun")
        .args(
            "-sdk macosx metallib target/dotprod.air -o target/dotprod.metallib"
                .split_ascii_whitespace(),
        )
        .output()?;

    println!("status: {}", output.status);
    io::stdout().write_all(&output.stdout)?;
    io::stderr().write_all(&output.stderr)?;

    Ok(())
}
