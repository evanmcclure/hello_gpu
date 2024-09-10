//! Hello world example for Rust on GPU.

#![forbid(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

use clap::Parser;

mod dot_product;

/// Simple program to do vector dot product
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Use the CPU instead of the GPU
    #[arg(short, long)]
    use_cpu: bool,
}

/// foo bar
fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let command = dot_product::new(args.use_cpu);
    let result = command.execute(&[3, 4, 1, 7, 10, 20], &[2, 5, 6, 9, 5, 10])?;
    println!("result is {:?}", result);

    Ok(())
}
