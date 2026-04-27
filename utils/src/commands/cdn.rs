use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use futures::stream::{self, StreamExt};
use reqwest::Client;

use crate::signing;

fn collect_files_recursive(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    collect_files_inner(dir, &mut files)?;
    Ok(files)
}

fn collect_files_inner(
    current: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_inner(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

fn relative_path_str(base: &Path, file: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let rel = file.strip_prefix(base)?;
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

async fn upload_one(
    client: &Client,
    file_path: &Path,
    cdn_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read(file_path)?;
    let (url, headers) = signing::sign_put_request(cdn_path, data.len())?;

    let result = client.put(&url).headers(headers).body(data).send().await;
    match result {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                tracing::error!("upload failed: status={status}, body={body}");
                return Err(format!("upload failed: {status} - {body}").into());
            }
            tracing::debug!("uploaded {cdn_path}");
            Ok(())
        },
        Err(e) => {
            tracing::error!("upload error: {e:?}");
            Err(e.into())
        },
    }
}

fn build_list_query(prefix: Option<&str>) -> String {
    let mut query = "list-type=2".to_string();
    if let Some(p) = prefix {
        query.push_str(&format!("&prefix={p}"));
    }
    query
}

fn parse_list_keys(xml: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut search_from = 0;

    while let Some(start) = xml[search_from..].find("<Key>") {
        let abs_start = search_from + start + "<Key>".len();
        if let Some(end) = xml[abs_start..].find("</Key>") {
            keys.push(xml[abs_start..abs_start + end].to_string());
            search_from = abs_start + end + "</Key>".len();
        } else {
            break;
        }
    }

    keys
}

fn load_failed_keys(
    path: Option<&Path>,
) -> Result<Option<HashSet<String>>, Box<dyn std::error::Error>> {
    let Some(path) = path else {
        return Ok(None);
    };

    let content = fs::read_to_string(path)?;
    let keys: HashSet<String> = content.lines().map(|l| l.to_string()).collect();
    tracing::info!("loaded {} failed keys from {}", keys.len(), path.display());
    Ok(Some(keys))
}

pub async fn run_upload(dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let files = collect_files_recursive(&dir)?;
    tracing::info!("found {} files to upload", files.len());

    let client = Client::new();
    let total = files.len();
    let counter = std::sync::atomic::AtomicUsize::new(0);

    stream::iter(files)
        .map(|file| {
            let client = &client;
            let counter = &counter;
            let dir = dir.clone();
            async move {
                let cdn_path = format!("/{}", relative_path_str(&dir, &file)?);
                upload_one(client, &file, &cdn_path).await?;
                let n = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                tracing::info!("{n}/{total} uploaded {cdn_path}");
                Ok::<_, Box<dyn std::error::Error>>(())
            }
        })
        .buffer_unordered(20)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    tracing::info!("upload complete: {total} files");
    Ok(())
}

pub async fn run_upload_audio(
    dir: PathBuf,
    workers: usize,
    only_failed: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    let failed_keys = load_failed_keys(only_failed.as_deref())?;

    let files = collect_files_recursive(&dir)?;
    let files: Vec<PathBuf> = files
        .into_iter()
        .filter(|f| {
            let Ok(rel) = relative_path_str(&dir, f) else {
                return false;
            };
            let key = format!("/{rel}");
            match &failed_keys {
                None => true,
                Some(keys) => keys.contains(&key),
            }
        })
        .collect();

    tracing::info!(
        "uploading {} audio files with {workers} workers",
        files.len()
    );

    let client = Client::new();
    let total = files.len();
    let counter = std::sync::atomic::AtomicUsize::new(0);

    stream::iter(files)
        .map(|file| {
            let client = &client;
            let counter = &counter;
            let dir = dir.clone();
            async move {
                let cdn_path = format!("/{}", relative_path_str(&dir, &file)?);
                upload_one(client, &file, &cdn_path).await?;
                let n = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                tracing::info!("{n}/{total} uploaded {cdn_path}");
                Ok::<_, Box<dyn std::error::Error>>(())
            }
        })
        .buffer_unordered(workers)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    tracing::info!("audio upload complete: {total} files");
    Ok(())
}

pub async fn run_list(prefix: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let query = build_list_query(prefix.as_deref());
    let path_with_query = format!("/?{query}");
    let list_url = signing::cdn_url(&path_with_query)?;

    let client = Client::new();
    let response = client.get(&list_url).send().await?.error_for_status()?;
    let body = response.text().await?;

    let keys = parse_list_keys(&body);
    if keys.is_empty() {
        tracing::warn!("list returned 0 keys, response body: {body}");
    }
    for key in &keys {
        println!("{key}");
    }
    tracing::info!("listed {} objects", keys.len());
    Ok(())
}
