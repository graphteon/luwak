use reqwest::{Client, Request, Response};
use reqwest_middleware::{ClientBuilder, Middleware, Next, Result};
use task_local_extensions::Extensions;
use std::path::Path;
use std::fs;
use deno_core::url::Url;

pub struct LuwakReqwest;

#[async_trait::async_trait]
impl Middleware for LuwakReqwest {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        let luwak_path = Path::new(env!("HOME")).join(".luwak/modules");
        if !luwak_path.exists() {
            fs::create_dir_all(&luwak_path);
        }
        let module_url = Url::parse(&req.url().to_string()).unwrap();
        let module_url_path = luwak_path.join(
            module_url
                .join("./")
                .unwrap()
                .as_str()
                .replace("https://", "")
                .replace("http://", ""),
        );
        let module_url_file = luwak_path.join(
            module_url
                .as_str()
                .replace("https://", "")
                .replace("http://", ""),
        );

        println!("file {}", module_url_file.to_string_lossy());
        if !module_url_path.exists() {
            println!("directory {}", module_url_path.to_string_lossy());
            fs::create_dir_all(module_url_path);
        }

        // if module_url_file.exists() {
        //     let bytes = tokio::fs::read(module_url_file).await?;
        // }
        //println!("Request started {:?}", req);
        let res = next.run(req, extensions).await;
        let data = &res?.error_for_status();
        println!("Result: {:?}", data.text().await?);
        res
    }
}