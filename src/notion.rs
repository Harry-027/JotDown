use crate::util::{
    CREATE_DATABASE_URL, CREATE_PAGE_URL, ReqMethod, SEARCH_BY_FILTER_URL, send_request,
};
use anyhow::Result;
use reqwest::StatusCode;
use serde_json::Value;
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

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

    /// Creates a page using formatted blocks
    ///
    /// # Arguments
    ///
    /// * `database_id` - ID of the Notion database
    /// * `title` - Title of the page
    /// * `blocks` - Formatted content blocks
    ///
    /// # Returns
    ///
    /// * `Result<(StatusCode, Value)>` - API status and response
    pub async fn create_page_with_blocks(
        &self,
        database_id: &str,
        title: &str,
        blocks: &[Value],
    ) -> Result<(StatusCode, Value)> {
        // Take the first 100 blocks (Notion API limit)
        let first_batch = if blocks.len() > 100 { &blocks[..100] } else { blocks };
        
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
            "children": first_batch
        });
        
        // Create the page with the first batch of blocks
        let (status, response) = send_request(
            CREATE_PAGE_URL,
            ReqMethod::Post,
            Some(body),
            self.token.as_str(),
        ).await?;
        
        // If there are more blocks and the page was created successfully
        if blocks.len() > 100 && status.is_success() {
            if let Some(page_id) = response.get("id").and_then(|v| v.as_str()) {
                // Add remaining blocks in batches of 100
                for chunk_start in (100..blocks.len()).step_by(100) {
                    let chunk_end = (chunk_start + 100).min(blocks.len());
                    let chunk = &blocks[chunk_start..chunk_end];
                    
                    let _ = self.append_blocks(page_id, chunk).await?;
                    
                    // Add a small delay to avoid rate limits
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
        
        Ok((status, response))
    }
    
    /// Adds blocks to an existing page
    ///
    /// # Arguments
    ///
    /// * `page_id` - ID of the Notion page
    /// * `blocks` - Content blocks to add
    ///
    /// # Returns
    ///
    /// * `Result<(StatusCode, Value)>` - API status and response
    pub async fn append_blocks(
        &self,
        page_id: &str,
        blocks: &[Value],
    ) -> Result<(StatusCode, Value)> {
        let url = format!("https://api.notion.com/v1/blocks/{}/children", page_id);
        
        let body = json!({
            "children": blocks
        });
        
        send_request(
            &url,
            ReqMethod::Patch,
            Some(body),
            self.token.as_str(),
        ).await
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
    
    /// Updates a page with new formatted blocks
    ///
    /// # Arguments
    ///
    /// * `page_id` - ID of the Notion page
    /// * `blocks` - Formatted content blocks
    ///
    /// # Returns
    ///
    /// * `Result<(StatusCode, Value)>` - API status and response
    pub async fn update_page_with_blocks(
        &self,
        page_id: &str,
        blocks: &[Value],
    ) -> Result<(StatusCode, Value)> {
        // The Notion API doesn't allow replacing all blocks at once
        // So we need to add the new blocks
        
        // Take the first 100 blocks (Notion API limit)
        let first_batch = if blocks.len() > 100 { &blocks[..100] } else { blocks };
        
        let url = format!("https://api.notion.com/v1/blocks/{}/children", page_id);
        
        let body = json!({
            "children": first_batch
        });
        
        let (status, response) = send_request(
            &url,
            ReqMethod::Patch,
            Some(body),
            self.token.as_str(),
        ).await?;
        
        // If there are more blocks and the update was successful
        if blocks.len() > 100 && status.is_success() {
            // Add remaining blocks in batches of 100
            for chunk_start in (100..blocks.len()).step_by(100) {
                let chunk_end = (chunk_start + 100).min(blocks.len());
                let chunk = &blocks[chunk_start..chunk_end];
                
                let _ = self.append_blocks(page_id, chunk).await?;
                
                // Add a small delay to avoid rate limits
                sleep(Duration::from_millis(100)).await;
            }
        }
        
        Ok((status, response))
    }
}
