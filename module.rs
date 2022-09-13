use std::fs;
use std::path::Path;
use std::pin::Pin;

use std::cmp::min;
use std::fs::File;
use std::io::Write;

use crate::download::download_luwak_module;
use http_cache_reqwest::{CACacheManager, Cache, CacheMode, HttpCache};
use reqwest::Client;
use reqwest_middleware::ClientBuilder;

use data_url::DataUrl;
use deno_ast::{MediaType, ParseParams, SourceTextInfo};
use deno_core::anyhow::{anyhow, bail, Error};
use deno_core::futures::FutureExt;
use deno_core::resolve_import;
use deno_core::url::Url;
use deno_core::ModuleLoader;
use deno_core::ModuleSource;
use deno_core::ModuleSourceFuture;
use deno_core::ModuleSpecifier;
use deno_core::ModuleType;

pub struct LuwakModule;

impl ModuleLoader for LuwakModule {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _is_main: bool,
    ) -> Result<ModuleSpecifier, Error> {
        Ok(resolve_import(specifier, referrer)?)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<ModuleSpecifier>,
        _is_dyn_import: bool,
    ) -> Pin<Box<ModuleSourceFuture>> {
        let module_specifier = module_specifier.clone();
        let string_specifier = module_specifier.to_string();
        async move {
            let bytes: _ = match module_specifier.scheme() {
                "node" | "http" | "https" | "file" => {
                    let luwak_path = Path::new(env!("HOME")).join(".luwak/modules");
                    if !luwak_path.exists() {
                        fs::create_dir_all(&luwak_path);
                    }
                    let module_url = Url::parse(module_specifier.as_str()).unwrap();
                    let module_url_path = luwak_path.join(
                        module_url
                            .join("./")
                            .unwrap()
                            .as_str()
                            .replace("https://", "")
                            .replace("http://", "")
                            .replace("node://", ""),
                    );
                    let module_url_file = luwak_path.join(
                        module_url
                            .as_str()
                            .replace("https://", "")
                            .replace("http://", "")
                            .replace("node://", ""),
                    );

                    // println!("file {}", module_specifier.as_str());
                    if !module_url_path.exists() && module_specifier.scheme() != "file" {
                        fs::create_dir_all(module_url_path);
                    }

                    let path;
                    if module_specifier.scheme() != "file" {
                        let module_download = Url::parse(module_specifier.as_str()).unwrap();
                        let module_download_file =
                            module_url.as_str().replace("node://", "https://esm.sh/");
                        let save_file_to;
                        if module_url_file.extension().unwrap().to_str().unwrap() != "0" {
                            save_file_to = module_url_file;
                        } else {
                            save_file_to = module_url_file.join("index.js");
                        }
                        if !save_file_to.exists() {
                            download_luwak_module(
                                module_download_file.as_str(),
                                &save_file_to.to_string_lossy(),
                            )
                            .await
                            .unwrap();
                        }
                        path = save_file_to;
                    } else {
                        path = match module_specifier.to_file_path() {
                            Ok(path) => path,
                            Err(_) => bail!("Invalid file URL."),
                        };
                    }

                    let bytes = tokio::fs::read(path).await?;
                    bytes

                    // let module_url = Url::parse(module_specifier.as_str()).unwrap();
                    // let module_url_file = module_url.as_str().replace("node://", "https://esm.sh/");

                    // println!("Download : {}", module_url);

                    // let res = reqwest::get(module_url_file).await?;
                    // // TODO: The HTML spec says to fail if the status is not
                    // // 200-299, but `error_for_status()` fails if the status is
                    // // 400-599.
                    // let res = res.error_for_status()?;
                    // res.bytes().await?
                }
                "data" => {
                    let url = match DataUrl::process(module_specifier.as_str()) {
                        Ok(url) => url,
                        Err(_) => bail!("Not a valid data URL."),
                    };
                    let bytes = match url.decode_to_vec() {
                        Ok((bytes, _)) => bytes,
                        Err(_) => bail!("Not a valid data URL."),
                    };
                    bytes.into()
                }
                schema => bail!("Invalid schema {}", schema),
            };

            // Strip BOM
            // let bytes = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            //     bytes.slice(3..)
            // } else {
            //     bytes
            // };

            let parsed = deno_ast::parse_module(ParseParams {
                specifier: string_specifier.clone(),
                text_info: SourceTextInfo::from_string(
                    String::from_utf8_lossy(&bytes).into_owned(),
                ),
                media_type: MediaType::TypeScript,
                capture_tokens: false,
                scope_analysis: false,
                maybe_syntax: None,
            })?;

            Ok(ModuleSource {
                code: parsed
                    .transpile(&Default::default())?
                    .text
                    .into_bytes()
                    .into_boxed_slice(),
                module_type: ModuleType::JavaScript,
                module_url_specified: string_specifier.clone(),
                module_url_found: string_specifier.clone(),
            })
        }
        .boxed_local()
    }
}
