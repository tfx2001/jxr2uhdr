use anyhow::Result;
use clap::Parser;
use log::info;

mod convert;
mod decode;
mod encode;
mod types;

use types::Image;

#[derive(Parser, Debug)]
#[command(
    name = "jxr2uhdr",
    version = "0.1",
    about = "Convert JXR HDR images to Ultra HDR JPEGs"
)]
struct Cli {
    /// Input JXR file path
    #[arg(short, long)]
    input: String,

    /// Output Ultra HDR JPG file path
    #[arg(short, long)]
    output: String,

    /// Quality of the output base JPEG (0-100)
    #[arg(short, long, default_value_t = 90)]
    quality: u8,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    info!("Decoding {}...", cli.input);
    let mut hdr_image = decode::decode_jxr(&cli.input)?;

    info!("Encoding Ultra HDR...");
    encode::encode_ultra_hdr(&mut hdr_image, cli.quality as i32, &cli.output)?;

    info!("Successfully saved Ultra HDR image to {}", cli.output);

    Ok(())
}
