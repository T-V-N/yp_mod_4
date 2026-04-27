use anyhow::Context;
use clap::Parser;
use image::ImageReader;
use image_processor::process;
use std::{fs::File, io::Read};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    input: String,
    #[arg(long)]
    output: String,
    #[arg(long)]
    plugin: String,
    #[arg(long)]
    params: String,
    #[arg(long, default_value = "target/debug")]
    plugin_path: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut img = ImageReader::open(&args.input)
        .with_context(|| format!("Failed to open input: {}/{}", args.plugin_path, args.input))?
        .decode()?
        .to_rgba8();
    let plugin_path = format!("{}/{}", args.plugin_path, args.plugin);

    let mut params = String::new();
    File::open(&args.params)
        .with_context(|| format!("Failed to open params file: {}", args.params))?
        .read_to_string(&mut params)?;

    process(&mut img, &plugin_path, &params)?;

    img.save(&args.output)?;
    Ok(())
}
