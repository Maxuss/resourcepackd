use std::{
    fs::File,
    io::{BufWriter, Read, Write},
    path::PathBuf,
    time::Instant,
};

use zip::write::FileOptions;

pub fn compile_pack(root: PathBuf, out: PathBuf) -> anyhow::Result<()> {
    tracing::info!("Compiling pack from {root:?} to {out:?}");
    if out.exists() {
        std::fs::remove_file(out.clone())?;
    }

    let mut out = File::create(out)?;
    let mut writer = BufWriter::new(&mut out);
    let mut zip_writer = zip::ZipWriter::new(&mut writer);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let start_time = Instant::now();
    for file in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = file
            .path()
            .strip_prefix(&root)?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        // tracing::trace!("Writing file {path:?}");
        if file.file_type().is_file() {
            zip_writer.start_file(path, options).unwrap();
            let mut f = File::open(file.path()).unwrap();
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer).unwrap();
            zip_writer.write_all(&buffer).unwrap();
        }
    }

    zip_writer.finish().unwrap();

    let elapsed = (Instant::now() - start_time).as_millis();
    tracing::info!("Compiled pack in {elapsed}ms!");

    Ok(())
}
