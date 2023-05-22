#![allow(clippy::declare_interior_mutable_const)]

use std::{
    cell::OnceCell,
    path::PathBuf,
    sync::atomic::{AtomicU32, Ordering},
    time::Instant,
};

use anyhow::bail;
use async_walkdir::WalkDir;
use futures_lite::StreamExt;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
};

use async_zip::{tokio::write::ZipFileWriter, ZipEntryBuilder};
use regex::{Captures, Regex};

use crate::validate;

fn init_json_regex() -> Regex {
    Regex::new(r"\.(json|mcmeta)$").unwrap()
}

fn init_jsonc_regex() -> Regex {
    Regex::new(r"\.(json|mcmeta)c$").unwrap()
}

const JSON_REGEX: OnceCell<Regex> = OnceCell::new();
const JSONC_REGEX: OnceCell<Regex> = OnceCell::new();

#[tracing::instrument(skip(root, out_p, validate))]
pub async fn compile_pack(root: PathBuf, out_p: PathBuf, validate: bool) -> anyhow::Result<()> {
    if out_p.exists() {
        tokio::fs::remove_file(out_p.clone()).await?;
    } else if !out_p
        .parent()
        .ok_or(anyhow::anyhow!("Invalid path"))?
        .exists()
    {
        tokio::fs::create_dir_all(out_p.parent().unwrap()).await?;
    }
    let mut out = File::create(out_p.clone()).await?;
    let mut writer = BufWriter::new(&mut out);
    let mut zip_writer = ZipFileWriter::with_tokio(&mut writer);

    let start_time = Instant::now();
    let err_count = AtomicU32::new(0);

    let mut entries = WalkDir::new(&root);

    loop {
        match entries.next().await {
            Some(Ok(file)) => {
                let path_p = file.path();
                let path = path_p
                    .strip_prefix(&root)?
                    .to_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid path"))?;

                if file.file_type().await?.is_file() {
                    if validate {
                        let c_json_regex = JSON_REGEX;
                        let c_jsonc_regex = JSONC_REGEX;
                        let json_regex = c_json_regex.get_or_init(init_json_regex).clone();
                        let jsonc_regex = c_jsonc_regex.get_or_init(init_jsonc_regex).clone();
                        let st = &path.to_string();

                        if json_regex.is_match(st) {
                            let entry =
                                ZipEntryBuilder::new(path.into(), async_zip::Compression::Deflate);

                            if let Ok(s) = validate::validate_json(file.path(), false).await {
                                zip_writer.write_entry_whole(entry, s.as_bytes()).await?;
                            } else {
                                err_count.fetch_add(1, Ordering::SeqCst);
                            }
                        } else if jsonc_regex.is_match(st) {
                            let file_path = &*jsonc_regex.replace(path, |capture: &Captures| {
                                let matching = capture.get(0).unwrap().as_str();
                                matching[..matching.len() - 1].to_string()
                            });

                            let entry = ZipEntryBuilder::new(
                                file_path.into(),
                                async_zip::Compression::Deflate,
                            );

                            if let Ok(s) = validate::validate_json(file.path(), true).await {
                                zip_writer.write_entry_whole(entry, s.as_bytes()).await?;
                            } else {
                                err_count.fetch_add(1, Ordering::SeqCst);
                            }
                        } else {
                            let entry =
                                ZipEntryBuilder::new(path.into(), async_zip::Compression::Deflate);

                            let f = File::open(file.path()).await?;
                            let mut buffer =
                                Vec::with_capacity(file.metadata().await?.len() as usize);
                            BufReader::new(f).read_to_end(&mut buffer).await?;

                            zip_writer.write_entry_whole(entry, &buffer).await?;
                        };
                    } else {
                        let entry =
                            ZipEntryBuilder::new(path.into(), async_zip::Compression::Deflate);

                        let f = File::open(file.path()).await?;
                        let mut buffer = Vec::with_capacity(file.metadata().await?.len() as usize);
                        BufReader::new(f).read_to_end(&mut buffer).await?;

                        zip_writer.write_entry_whole(entry, &buffer).await?;
                    }
                }
            }
            Some(Err(e)) => {
                bail!(e)
            }
            None => break,
        }
    }

    zip_writer.close().await?;
    writer.flush().await?;
    out.flush().await?;

    if err_count.load(Ordering::SeqCst) > 0 {
        tracing::error!(
            "{} errors found while compiling!",
            err_count.load(Ordering::SeqCst)
        );
    } else {
        let elapsed = (Instant::now() - start_time).as_millis();
        tracing::info!("Compiled pack to {out_p:?} in {elapsed}ms!");
    }

    Ok(())
}
