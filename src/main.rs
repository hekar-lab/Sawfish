use std::path::PathBuf;

use clap::Parser;
use sawfish::slaspec::builder::SLASpecBuilder;

/// Easiest side quest :)
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Output directory
    #[arg(short, long)]
    outdir: PathBuf,
}

fn main() {
    let args = Args::parse();
    let slab = SLASpecBuilder::new();
    slab.build(&args.outdir);
}
