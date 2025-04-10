use crate::util::{
    CREATE_DATABASE_URL, CREATE_PAGE_URL, ReqMethod, SEARCH_BY_FILTER_URL, send_request,
};
use anyhow::Result;
use reqwest::StatusCode;
use serde_json::Value;
use serde_json::json;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Notion {
    token: String,
}

impl Notion {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_owned(),
        }
    }

    pub async fn search_ref(&self, title: &str, ref_type: &str) -> Result<(StatusCode, Value)> {
        let body = json!({
        "query":title,
            "filter": {
                "value": ref_type,
                "property": "object"
            },
            "sort":{
              "direction":"ascending",
              "timestamp":"last_edited_time"
            }
        });
        send_request(
            SEARCH_BY_FILTER_URL,
            ReqMethod::Post,
            Some(body),
            self.token.as_str(),
        )
        .await
    }

    pub async fn create_database(&self, page_id: &str) -> Result<(StatusCode, Value)> {
        let body = json!({
            "parent": {
                "type": "page_id",
                "page_id": page_id
            },
            "icon": {
                "type": "emoji",
                "emoji": "📝"
              },
            "title": [
                {
                    "type": "text",
                    "text": {
                        "content": "Jot It Down McpServer Database"
                    }
                }
            ],
            "properties": {
                "Name": {
                    "title": {}
                },
                "Content": {
                    "rich_text": {}
                }
            }
        });
        send_request(
            CREATE_DATABASE_URL,
            ReqMethod::Post,
            Some(body),
            self.token.as_str(),
        )
        .await
    }

    pub async fn create_page(
        &self,
        database_id: &str,
        title: &str,
        content: &str,
    ) -> Result<(StatusCode, Value)> {
        let body = json!({
            "parent": {
                "database_id": database_id
            },
            "icon": {
                "emoji": "🥬"
            },
            "properties": {
                "Name": {
                    "title": [
                        {
                            "text": {
                                "content": title
                            }
                        }
                    ]
                },
                "Content": {
                    "rich_text": [
                        {
                            "text": {
                                "content": "Content noted by Jotdown MCP server"
                            }
                        }
                    ]
                }
            },
            "children": [
                {
            "object": "block",
            "type": "paragraph",
            "paragraph": {
                "rich_text": [
                    {
                        "type": "text",
                        "text": {
                            "content": content
                        }
                    }
                ]
            }
        }
            ]
        });
        send_request(
            CREATE_PAGE_URL,
            ReqMethod::Post,
            Some(body),
            self.token.as_str(),
        )
        .await
    }

    pub async fn fetch_page_content(&self, page_id: &str) -> Result<(StatusCode, Value)> {
        let page_content_url = format!(
            "https://api.notion.com/v1/blocks/{}/children?page_size=100",
            page_id
        );
        send_request(
            page_content_url.as_str(),
            ReqMethod::Get,
            None,
            self.token.as_str(),
        )
        .await
    }

    pub async fn update_page(&self, page_id: &str, content: &str) -> Result<(StatusCode, Value)> {
        let page_update_url = format!("https://api.notion.com/v1/blocks/{}/children", page_id);
        let body = json!({
           	"children": [
          		{
         			"object": "block",
         			"type": "paragraph",
         			"paragraph": {
            				"rich_text": [
               					{
                  						"type": "text",
                  						"text": {
                         			        "content":content
                                        }
               					}
            				]
         			}
          		}
           	]
        });
        send_request(
            page_update_url.as_str(),
            ReqMethod::Patch,
            Some(body),
            self.token.as_str(),
        )
        .await
    }
}
