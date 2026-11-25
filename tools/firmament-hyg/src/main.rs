use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use firmament_hyg::hyg::{self, Star};
use firmament_hyg::pack;

/// Packs the HYG star catalog into Firmament's binary star tables.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// HYG catalog CSV (v4.x)
    input: PathBuf,

    /// Directory to write stars.bin, names.bin and hip.bin into
    #[arg(short, long)]
    out_dir: PathBuf,

    /// Faintest magnitude to keep
    #[arg(short, long, default_value_t = 10.0)]
    max_mag: f64,
}

type Writer = fn(&mut BufWriter<File>, &[Star]) -> io::Result<()>;

fn main() -> Result<()> {
    let args = Args::parse();

    let rows = hyg::read(&args.input)?;
    let (stars, skipped) = hyg::select(&rows, args.max_mag);

    println!("read {} rows from {}", rows.len(), args.input.display());
    println!(
        "skipped: {} sun, {} no coords, {} no mag, {} fainter than {}",
        skipped.sun, skipped.no_coords, skipped.no_mag, skipped.faint, args.max_mag
    );
    if let (Some(first), Some(last)) = (stars.first(), stars.last()) {
        println!(
            "kept {} stars, mag {:.2} to {:.2}",
            stars.len(),
            first.mag,
            last.mag
        );
    }
    for cutoff in [4.0, 6.5, 8.0] {
        let n = stars.partition_point(|s| s.mag <= cutoff);
        println!("  mag <= {cutoff:.1}: {n}");
    }

    fs::create_dir_all(&args.out_dir)
        .with_context(|| format!("creating {}", args.out_dir.display()))?;
    let files: [(&str, Writer); 3] = [
        ("stars.bin", pack::write_stars),
        ("names.bin", pack::write_names),
        ("hip.bin", pack::write_hip),
    ];
    for (name, write) in files {
        write_file(&args.out_dir.join(name), &stars, write)?;
    }
    Ok(())
}

fn write_file(path: &Path, stars: &[Star], write: Writer) -> Result<()> {
    let file = File::create(path).with_context(|| format!("creating {}", path.display()))?;
    let mut w = BufWriter::new(file);
    write(&mut w, stars)
        .and_then(|_| w.flush())
        .with_context(|| format!("writing {}", path.display()))?;
    println!(
        "wrote {} ({} bytes)",
        path.display(),
        fs::metadata(path)?.len()
    );
    Ok(())
}
