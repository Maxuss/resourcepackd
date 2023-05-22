use std::{path::Path, time::Instant};

use anyhow::bail;
use json_comments::StripComments;
use serde_json::Value;
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};

#[tracing::instrument(skip(path_p, jsonc))]
pub async fn validate_json<P: AsRef<Path>>(path_p: P, jsonc: bool) -> anyhow::Result<String> {
    let path = path_p.as_ref();
    let start_time = Instant::now();

    let file = File::open(&path).await?;
    let mut out = String::with_capacity(file.metadata().await?.len() as usize);
    BufReader::new(file).read_to_string(&mut out).await?;

    let res = if jsonc {
        let stripping = StripComments::new(out.as_bytes());
        serde_json::from_reader::<StripComments<&[u8]>, Value>(stripping)
    } else {
        serde_json::from_str::<Value>(&out)
    };
    if let Err(error) = res {
        tracing::error!("{path:?} contains invalid JSON!");
        tracing::error!("{error}");
        bail!("Invalid JSON")
    }

    let elapsed = (Instant::now() - start_time).as_millis();
    tracing::trace!("Validation took {elapsed}ms");

    let start_time = Instant::now();

    let result = serde_json::to_string(&res?)?;

    let elapsed = (Instant::now() - start_time).as_millis();
    tracing::trace!("Minifying took {elapsed}ms");

    Ok(result)
}
