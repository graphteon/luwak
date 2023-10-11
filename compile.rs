use std::io::prelude::*;
use std::io::{BufReader, Seek, Write};
use std::iter::Iterator;
use std::path::{Path, PathBuf};
use zip::result::ZipError;
use zip::write::FileOptions;

use std::fs::File;
use walkdir::{DirEntry, WalkDir};

use reqwest::header::USER_AGENT;
use reqwest::Client;
use tinyjson::{JsonParseError, JsonValue};

const METHOD_STORED: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Stored);

#[cfg(any(
    feature = "deflate",
    feature = "deflate-miniz",
    feature = "deflate-zlib"
))]
const METHOD_DEFLATED: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Deflated);
#[cfg(not(any(
    feature = "deflate",
    feature = "deflate-miniz",
    feature = "deflate-zlib"
)))]
const METHOD_DEFLATED: Option<zip::CompressionMethod> = None;

#[cfg(feature = "bzip2")]
const METHOD_BZIP2: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Bzip2);
#[cfg(not(feature = "bzip2"))]
const METHOD_BZIP2: Option<zip::CompressionMethod> = None;

#[cfg(feature = "zstd")]
const METHOD_ZSTD: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Zstd);
#[cfg(not(feature = "zstd"))]
const METHOD_ZSTD: Option<zip::CompressionMethod> = None;

fn init_byte(script: &str) -> Vec<u8> {
    format!(
        r#"
    #!/usr/bin/env bash

    # commands that you need to do ...
    # ...
    TEMPDIR=`mktemp -d`;
    unzip -qq $(basename "$0") -d $TEMPDIR &>/dev/null
    $TEMPDIR/luwak $TEMPDIR/{}
    exit
    "#,
        script
    )
    .into_bytes()
}

fn init_script(script_path: &str) -> Vec<u8> {
    let script_path = Path::new(&script_path);
    let script_md = script_path.metadata().unwrap();
    let mut exe_script: PathBuf = PathBuf::new();

    if script_md.is_dir() {
        // let dirname = script_path.to_str().unwrap().split("/").last().unwrap();
        // let _ = exe_script.push(dirname);
        if script_path.join("main.ts").exists() {
            let _ = exe_script.push("main.ts");
        } else if script_path.join("main.js").exists() {
            let _ = exe_script.push("main.js");
        } else if script_path.join("main.mjs").exists() {
            let _ = exe_script.push("main.mjs");
        } else {
            panic!(
                "Can't find initial script like main.js, main.ts, or main.mjs in your directory!"
            );
        };
    } else {
        let filename = script_path.file_name().unwrap().to_str().unwrap();
        exe_script.push(filename);
    }
    init_byte(exe_script.to_str().unwrap())
}

pub async fn download_latest_binary() -> Result<(), JsonParseError> {
    let url = "https://api.github.com/repos/graphteon/luwak/releases/latest";
    let response = Client::new()
        .get(url)
        .header(USER_AGENT, "Awesome-Octocat-App")
        .send()
        .await
        .expect("failed to get response")
        .text()
        .await
        .expect("failed to get payload");
    let version: JsonValue = response.parse().unwrap();
    let version_name = version["name"].clone();

    print!("{:?}", version_name);

    Ok(())
}

fn zip(src_dir: &str, dst_file: &str) -> i32 {
    for &method in [METHOD_STORED, METHOD_DEFLATED, METHOD_BZIP2, METHOD_ZSTD].iter() {
        if method.is_none() {
            continue;
        }
        match dozip(src_dir, dst_file, method.unwrap()) {
            Ok(_) => println!("done: {src_dir} written to {dst_file}"),
            Err(e) => println!("Error: {e:?}"),
        }
    }

    0
}

fn zipfile(src_file: &str, dst_file: &str) -> zip::result::ZipResult<()> {
    // read file
    let f = File::open(src_file)?;
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    // Read file into vector.
    reader.read_to_end(&mut buffer)?;

    // source file
    let source_file = Path::new(src_file).file_name().unwrap().to_str().unwrap();

    let path = Path::new(dst_file);
    let file = File::create(path).unwrap();

    let mut zip = zip::ZipWriter::new(file);

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o755);
    zip.start_file(source_file, options)?;
    zip.write_all(&buffer)?;

    zip.finish()?;
    Ok(())
}

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            println!("adding file {path:?} as {name:?} ...");
            #[allow(deprecated)]
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            println!("adding dir {path:?} as {name:?} ...");
            #[allow(deprecated)]
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}

fn dozip(
    src_dir: &str,
    dst_file: &str,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let path = Path::new(dst_file);
    let file = File::create(path).unwrap();

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initscript_file() {
        assert_eq!(
            init_script(format!("{}/examples/cli/cli.js", env!("CARGO_MANIFEST_DIR")).as_str()),
            init_byte("cli.js")
        );
    }

    #[test]
    fn test_initscript_dir() {
        assert_eq!(
            init_script(format!("{}/examples/module", env!("CARGO_MANIFEST_DIR")).as_str()),
            init_byte("main.js")
        );
    }

    #[test]
    fn test_zip_dir() {
        assert_eq!(
            0,
            zip(
                format!("{}/examples/module", env!("CARGO_MANIFEST_DIR")).as_str(),
                format!("/tmp/testluwak.zip").as_str()
            )
        );
    }

    #[test]
    fn test_zip_file() {
        assert_eq!(
            (),
            zipfile(
                format!("{}/examples/cli/cli.js", env!("CARGO_MANIFEST_DIR")).as_str(),
                format!("/tmp/testfileluwak.zip").as_str()
            )
            .unwrap()
        );
    }

    #[tokio::test]
    async fn test_something_async() {
        let _ = download_latest_binary().await;
    }
}
