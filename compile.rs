use inquire::Select;
use notify::Error;
use reqwest::header::USER_AGENT;
use reqwest::Client;
use std::env::temp_dir;
use std::fs::{set_permissions, File, Permissions};
use std::io::copy;
use std::io::prelude::*;
use std::io::{BufReader, Seek, Write};
use std::iter::Iterator;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tinyjson::JsonValue;
use uuid::Uuid;
use walkdir::{DirEntry, WalkDir};
use zip::result::ZipError;
use zip::write::FileOptions;

use crate::download::luwak_downloader;

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

pub async fn do_pkg(input: &PathBuf, output: &PathBuf) -> Result<(), Error> {
    let temp_dir = temp_dir();
    let id = Uuid::new_v4();
    let work_dir = temp_dir.join(id.to_string());
    let _ = std::fs::create_dir_all(work_dir.to_str().unwrap());

    // DO Compress file
    let source_file = work_dir.join("source.zip");
    if input.is_dir() {
        zip(&input.to_str().unwrap(), source_file.to_str().unwrap());
    } else {
        let _ = zipfile(&input.to_str().unwrap(), source_file.to_str().unwrap());
    }

    let os_arch = do_select_os();
    let _ = download_latest_binary(os_arch.unwrap().as_str(), work_dir.to_str().unwrap()).await;

    let _ = add_file_to_zip(
        work_dir.join("luwak").to_str().unwrap(),
        source_file.to_str().unwrap(),
    );

    let mut source_file = File::open(source_file).expect("no such source file");
    let mut source_buf = Vec::new();
    source_file
        .read_to_end(&mut source_buf)
        .expect("Can't read source file");

    let loader = init_script(input.to_str().unwrap());

    let standalone = [loader, source_buf].concat();

    let mut package_file: File = File::create(output).expect("Unable to create package file!");
    package_file
        .write_all(&standalone)
        .expect("Unable to write data to package file");

    set_permissions(output, Permissions::from_mode(0o770)).unwrap();

    let _ = std::fs::remove_dir_all(work_dir);
    Ok(())
}

fn do_select_os() -> Result<String, Error> {
    let options: Vec<&str> = vec!["Linux - x86_64", "Mac OS - x86_64", "Mac OS - Apple Chip"];

    let choice: &str = Select::new("Select your target operating system?", options)
        .prompt()
        .unwrap();

    match choice {
        "Linux - x86_64" => Ok(String::from("ubuntu")),
        "Mac OS - x86_64" => Ok(String::from("macOS")),
        "Mac OS - Apple Chip" => Ok(String::from("macOS-arm64")),
        _ => panic!("There was an error, please try again"),
    }
}

fn init_byte(script: &str) -> Vec<u8> {
    format!(
        r#"
    #!/usr/bin/env bash

    # commands that you need to do ...
    # ...
    TEMPDIR=`mktemp -d`;
    unzip -qq $(dirname "$0")/$(basename "$0") -d $TEMPDIR &>/dev/null
    chmod +x $TEMPDIR/luwak && cd $TEMPDIR && $TEMPDIR/luwak ./{}
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

pub async fn download_latest_binary(os_arch: &str, path: &str) -> Result<(), String> {
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
    let version_name = version["name"].format().unwrap().replace("\"", "");
    let download_url = format!(
        r#"https://github.com/graphteon/luwak/releases/download/{}/luwak-{}-latest"#,
        version_name, os_arch
    );
    luwak_downloader(download_url.as_str(), format!("{path}/luwak").as_str()).await

    //Ok(())
}

fn add_file_to_zip(file: &str, zip_file: &str) -> zip::result::ZipResult<()> {
    let zip_file_path = Path::new(zip_file);
    let mut zip_file = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .open(&zip_file_path)
        .unwrap();
    let mut zip = zip::ZipWriter::new_append(&mut zip_file)?;
    let files_to_compress: Vec<PathBuf> = vec![PathBuf::from(file)];
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    for file_path in &files_to_compress {
        let file = File::open(file_path)?;
        let file_name = file_path.file_name().unwrap().to_str().unwrap();

        // Adding the file to the ZIP archive.
        zip.start_file(file_name, options)?;

        let mut buffer = Vec::new();
        copy(&mut file.take(u64::MAX), &mut buffer)?;

        zip.write_all(&buffer)?;
    }

    let _ = zip.finish();
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
}
