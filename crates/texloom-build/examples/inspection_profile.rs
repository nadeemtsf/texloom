//! Measure the checked PDF worker on one immutable PDF.
use sha2::{Digest, Sha256};
use std::{env, fs, io::Read, path::PathBuf, time::Instant};
use texloom_build::{inspection_worker::Worker, scheduler::Cancellation};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args_os().skip(1).map(PathBuf::from);
    let pdf = args.next().ok_or("Provide a PDF")?.canonicalize()?;
    let verifier = args
        .next()
        .ok_or("Provide the packaged verification directory")?;
    let scratch = args.next().ok_or("Provide a new scratch directory")?;
    let pairs: usize = args
        .next()
        .ok_or("Provide a pair count")?
        .to_string_lossy()
        .parse()?;
    if args.next().is_some() || !(1..=10).contains(&pairs) {
        return Err("Expected 1 to 10 pairs".into());
    }
    fs::create_dir(&scratch)?;
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(verifier.join("verifier-manifest.json"))?)?;
    let worker_hash = manifest["worker_sha256"]
        .as_str()
        .ok_or("Worker hash is missing")?;
    let mut pdf_bytes = Vec::new();
    fs::File::open(&pdf)?
        .take(40_000_001)
        .read_to_end(&mut pdf_bytes)?;
    if pdf_bytes.len() > 40_000_000 {
        return Err("PDF exceeds the inspection budget".into());
    }
    let sha = format!("{:x}", Sha256::digest(pdf_bytes));
    let worker = Worker::new(
        verifier.join("pdfium_worker.exe"),
        worker_hash.into(),
        verifier.join("pdfium.dll"),
        scratch,
    )?;
    let cancel = Cancellation::default();
    let mut reference = None;
    for pair in 0..pairs {
        for count in if pair % 2 == 0 { [4, 8] } else { [8, 4] } {
            let started = Instant::now();
            let (mut start, mut total, mut requests) = (0, None, 0);
            let mut worker_seconds = 0.;
            let mut max_window_raster_bytes = 0;
            let mut digest = Sha256::new();
            while total != Some(start) {
                let requested = Instant::now();
                let result = worker.inspect_window(&pdf, &sha, None, start, count, &cancel)?;
                worker_seconds += requested.elapsed().as_secs_f64();
                requests += 1;
                if total
                    .replace(result.page_total)
                    .is_some_and(|last| last != result.page_total)
                    || result.view.pages.is_empty()
                {
                    return Err("Inconsistent page coverage".into());
                }
                let raster_bytes: usize = result
                    .view
                    .rasters
                    .iter()
                    .map(|raster| raster.pixels.len())
                    .sum();
                max_window_raster_bytes = max_window_raster_bytes.max(raster_bytes);
                for (page, metadata) in result.view.pages.iter().enumerate() {
                    let raster = &result.view.rasters[page];
                    digest.update(format!("{metadata:?}{:?}", result.links[page]));
                    digest.update(format!(
                        "{:?}",
                        (
                            raster.width,
                            raster.height,
                            raster.stride,
                            raster.channels,
                            raster.scale
                        )
                    ));
                    digest.update(&raster.pixels);
                    for glyph in result.view.glyphs.iter().filter(|glyph| glyph.page == page) {
                        digest.update(format!("{:?}", (&glyph.text, glyph.bounds, glyph.target)));
                    }
                    for graphic in result
                        .view
                        .graphics
                        .iter()
                        .filter(|graphic| graphic.page == page)
                    {
                        digest.update(format!("{:?}", (graphic.kind, graphic.bounds)));
                    }
                }
                start += result.view.pages.len();
            }
            let evidence = format!("{:x}", digest.finalize());
            if reference
                .as_ref()
                .is_some_and(|expected| expected != &evidence)
            {
                return Err("Inspection content changed between window sizes".into());
            }
            reference = Some(evidence.clone());
            println!(
                "{}",
                serde_json::json!({
                    "pair": pair + 1, "window_pages": count, "pages": start, "requests": requests,
                    "worker_seconds": worker_seconds, "total_seconds": started.elapsed().as_secs_f64(),
                    "max_window_raster_bytes": max_window_raster_bytes,
                    "pdf_sha256": sha, "worker_sha256": worker_hash, "evidence_sha256": evidence,
                })
            );
        }
    }
    Ok(())
}
