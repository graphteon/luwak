use std::fs;
use std::pin::Pin;

use crate::download::download_luwak_module;

use data_url::DataUrl;
use deno_ast::{MediaType, ParseParams, SourceTextInfo};
use deno_core::anyhow::{bail, Error};
use deno_core::futures::FutureExt;
use deno_core::resolve_import;
use deno_core::url::Url;
use deno_core::ModuleLoader;
use deno_core::ModuleSource;
use deno_core::ModuleSourceFuture;
use deno_core::ModuleSpecifier;
use deno_core::ModuleType;
use deno_core::ResolutionKind;

use crate::luwak_util::luwak_module;

pub struct LuwakModule;

impl ModuleLoader for LuwakModule {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _is_main: ResolutionKind,
    ) -> Result<ModuleSpecifier, Error> {
        Ok(resolve_import(
            specifier
                .replace("npm:@", "npm:npm@")
                .replace("npm:", "npm://")
                .replace("esm:@", "esm:npm@")
                .replace("esm:", "esm://")
                .as_str(),
            referrer,
        )?)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleSpecifier>,
        _is_dyn_import: bool,
    ) -> Pin<Box<ModuleSourceFuture>> {
        let module_specifier = module_specifier.clone();
        let string_specifier = module_specifier.to_string();
        async move {
            let bytes: _ = match module_specifier.scheme() {
                "node" | "npm" | "http" | "https" | "file" | "esm" => {
                    let luwak_path = luwak_module().unwrap();
                    let module_url = Url::parse(module_specifier.as_str()).unwrap();
                    //println!("DEBUG module_url {}", module_specifier.as_str());
                    let module_url_file = luwak_path.join(
                        module_url
                            .as_str()
                            .replace("https://", "")
                            .replace("http://", "")
                            .replace("npm://", "")
                            .replace("esm://", ""),
                    );

                    let path;
                    if module_specifier.scheme() != "file" {
                        let module_download_file = if module_url.as_str().contains("github.com") {
                            module_url
                                .as_str()
                                .replace("github.com", "raw.githubusercontent.com")
                                .replace("blob/", "")
                        } else {
                            module_url
                                .as_str()
                                .replace("npm://", "https://npm.graphteon.id/")
                                .replace("esm://", "https://esm.graphteon.id/")
                        };

                        //println!("module url : {}", &module_download_file);

                        let save_file_to;
                        if !module_url_file.extension().is_none()
                            && (module_url_file.extension().unwrap().to_str().unwrap() == "js"
                                || module_url_file.extension().unwrap().to_str().unwrap() == "ts"
                                || module_url_file.extension().unwrap().to_str().unwrap() == "mjs")
                        {
                            save_file_to = module_url_file;
                        } else {
                            save_file_to = module_url_file.join("index.js");
                        }
                        //create module directory
                        let module_url_path = save_file_to.parent().unwrap();
                        if !module_url_path.exists() && module_specifier.scheme() != "file" {
                            fs::create_dir_all(module_url_path).unwrap();
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

            let media_type = MediaType::from_specifier(&module_specifier.clone());

            let parsed = deno_ast::parse_module(ParseParams {
                specifier: string_specifier.clone(),
                text_info: SourceTextInfo::from_string(
                    String::from_utf8_lossy(&bytes).into_owned(),
                ),
                media_type,
                capture_tokens: false,
                scope_analysis: false,
                maybe_syntax: None,
            })?;

            Ok(ModuleSource::new_with_redirect(
                match media_type {
                    MediaType::Json => ModuleType::Json,
                    _ => ModuleType::JavaScript,
                },
                parsed.transpile(&Default::default())?.text.into(),
                &module_specifier,
                &module_specifier,
            ))
        }
        .boxed_local()
    }
}
