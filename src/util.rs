use anyhow::{Result, anyhow};
use reqwest::{Client, StatusCode};
use serde_json::Value;

pub const SEARCH_BY_FILTER_URL: &str = "https://api.notion.com/v1/search";
pub const CREATE_DATABASE_URL: &str = "https://api.notion.com/v1/databases/";
pub const CREATE_PAGE_URL: &str = "https://api.notion.com/v1/pages";

pub enum ReqMethod {
    Get,
    Post,
    Patch
}

pub async fn send_request(
    url: &str,
    method: ReqMethod,
    body: Option<serde_json::Value>,
    auth_token: &str,
) -> Result<(StatusCode, Value)> {
    let client = Client::new();
    match method {
        ReqMethod::Get => {
            let response = client
                .get(url)
                .header("Notion-Version", "2022-06-28")
                .header("Authorization", auth_token)
                .send()
                .await?;
            let status = response.status();
            let json_result = response.json::<Value>().await?;
            Ok((status, json_result))
        }
        ReqMethod::Post => {
            if let Some(req_body) = body {
                let response = client
                    .post(url)
                    .header("Notion-Version", "2022-06-28")
                    .header("Authorization", auth_token)
                    .json(&req_body)
                    .send()
                    .await?;
                let status = response.status();
                let json_result = response.json::<Value>().await?;
                Ok((status, json_result))
            } else {
                Err(anyhow!("request body is missing"))
            }
        }
        ReqMethod::Patch => {
            if let Some(req_body) = body {
                let response = client
                    .patch(url)
                    .header("Notion-Version", "2022-06-28")
                    .header("Authorization", auth_token)
                    .json(&req_body)
                    .send()
                    .await?;
                let status = response.status();
                let json_result = response.json::<Value>().await?;
                Ok((status, json_result))
            } else {
                Err(anyhow!("request body is missing"))
            }
        }
    }
}
