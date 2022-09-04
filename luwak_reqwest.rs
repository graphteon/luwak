use reqwest::{Request, Response, Client};
use reqwest_middleware::{Middleware, Next, Result};
use std::time;

#[derive(Debug)]
pub struct LuwakReqwest;

#[async_trait::async_trait]
impl Middleware for LuwakReqwest {
    async fn handle(
        &self,
        req: Request,
        client: Client,
        next: Next<'_>,
    ) -> Result<Response> {
        println!("sending request to {}", req.url());
        let now = time::Instant::now();
        let res = next.run(req, client).await?;
        println!("request completed ({:?})", now.elapsed());
        Ok(res)
    }
}