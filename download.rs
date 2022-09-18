// use std::cmp::min;
use std::fs::{ File, OpenOptions };
use std::io::Write;

use futures_util::StreamExt;
// use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use crate::cli_parser;
use std::env;

pub async fn download_luwak_module(url: &str, path: &str) -> Result<(), String> {
    let args = cli_parser::args();
    if args.libdump {
        println!("Download and save to luwaklibs.lock : {}", url);
        let luwak_libs = env::current_dir().unwrap().join("luwaklibs.lock");
        if !luwak_libs.exists() {
            File::create(luwak_libs.as_path()).or(Err(format!("Failed to create file '{}'", luwak_libs.to_string_lossy())))?;
        }
        let mut luwak_libs_file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(luwak_libs.as_path())
        .unwrap();

        if let Err(e) = writeln!(luwak_libs_file, "export * from '{}';", url) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }
    else{
        println!("Download : {}", url);
    }
    let client = Client::new();
    let res = client
        .get(url)
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))?;
    //// TODO
    // let total_size = res
    //     .content_length()
    //     .ok_or(format!("Failed to get content length from '{}'", &url))?;

    // // Indicatif setup
    // let pb = ProgressBar::new(total_size);
    // pb.set_style(ProgressStyle::default_bar()
    //     .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
    //     .progress_chars("#>-"));
    // pb.set_message(&format!("Downloading {}", url));

    // download chunks
    let mut file = File::create(path).or(Err(format!("Failed to create file '{}'", path)))?;
    //let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write_all(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        // let new = min(downloaded + (chunk.len() as u64), total_size);
        // downloaded = new;
        //pb.set_position(new);
    }

    //pb.finish_with_message(&format!("Downloaded {} to {}", url, path));
    return Ok(());
}
