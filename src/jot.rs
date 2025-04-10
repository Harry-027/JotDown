use std::fs::{self, File};
use std::io::Write;
use std::process::{Command, Stdio};
use rmcp::{Error as McpError, ServerHandler, model::*, schemars, tool};

use crate::notion::Notion;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AddPageRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UpdatePageRequest {
    pub page_id: String,
    pub content: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AddGitBook {
    pub name: String,
    pub content: Vec<GitBookChapter>
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GitBookChapter {
    pub name: String,
    pub content: String
}


#[allow(dead_code)]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Jotter {
    data_store: Notion,
}

#[tool(tool_box)]
impl Jotter {
    pub fn new(store: Notion) -> Self {
        Self { data_store: store }
    }

    async fn search_ref_db(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let ref_db_name = "Jot It Down MCP server database";
        match self.data_store.search_ref(ref_db_name, "database").await {
            Ok((_, json_resp)) => {
                if let Some(db_id) = json_resp
                    .get("results")
                    .and_then(|v| v.get(0))
                    .and_then(|v| v.get("id"))
                    .and_then(|v| v.as_str())
                {
                    Ok(db_id.to_string())
                } else {
                    Err("db not found".into())
                }
            }
            Err(e) => Err(e.to_string().into()),
        }
    }

    async fn search_ref_page(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let ref_db_name = "Jot It Down";
        match self.data_store.search_ref(ref_db_name, "page").await {
            Ok((_, json_resp)) => {
                if let Some(page_id) = json_resp
                    .get("results")
                    .and_then(|v| v.get(0))
                    .and_then(|v| v.get("id"))
                    .and_then(|v| v.as_str())
                {
                    Ok(page_id.to_string())
                } else {
                    Err("ref page not found".into())
                }
            }
            Err(e) => Err(e.to_string().into()),
        }
    }

    async fn create_ref_db(
        &self,
        page_id: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match self.data_store.create_database(page_id).await {
            Ok((_, json_resp)) => {
                if let Some(db_id) = json_resp
                    .get("results")
                    .and_then(|v| v.get(0))
                    .and_then(|v| v.get("id"))
                    .and_then(|v| v.as_str())
                {
                    Ok(db_id.to_string())
                } else {
                    Err("DB id not found".to_string().into())
                }
            }
            Err(e) => Err(e.to_string().into()),
        }
    }

    async fn create_page(
        &self,
        db_id: &str,
        title: &str,
        content: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match self.data_store.create_page(db_id, title, content).await {
            Ok((_, json_resp)) => Ok(json_resp.to_string()),
            Err(e) => Err(e.to_string().into()),
        }
    }

    #[tool(description = "Retrieve a page by its title or content to get the page id")]
    async fn retrieve_page(&self, #[tool(param)] content: String) -> Result<CallToolResult, McpError> {
        match self.data_store.search_ref(&content, "page").await {
            Ok((_, json_resp)) => {
                if let Some(page_id) = json_resp
                    .get("results")
                    .and_then(|v| v.get(0))
                    .and_then(|v| v.get("id"))
                    .and_then(|v| v.as_str())
                {
                    Ok(CallToolResult::success(vec![Content::text(page_id)]))
                } else {
                    Err(McpError::internal_error(
                        "error occurred: error finding page",
                        None,
                    ))
                }
            }
            Err(e) => Err(McpError::internal_error(
                format!("error occurred: error finding page: {}", e),
                None,
            )),
        }
    }

    #[tool(description = "Updates a page for given content and page id")]
    async fn update_page(&self, #[tool(aggr)] UpdatePageRequest { page_id, content }: UpdatePageRequest) -> Result<CallToolResult, McpError> {
            match self.data_store.update_page(page_id.as_str(), content.as_str()).await {
                Ok((_, val)) => Ok(CallToolResult::success(vec![Content::text(val.to_string())])),
                Err(e) => Err(McpError::internal_error(
                    format!("error occurred: error updating page: {}", e),
                    None,
                )),
            }
    }

    #[tool(description = "Create a new page")]
    async fn create_new_page(
        &self,
        #[tool(aggr)] AddPageRequest { title, content }: AddPageRequest,
    ) -> Result<CallToolResult, McpError> {
        match self.search_ref_db().await {
            Ok(db_id) => match self
                .create_page(db_id.as_str(), title.as_str(), content.as_str())
                .await
            {
                Ok(resp) => Ok(CallToolResult::success(vec![Content::text(
                    resp.to_string(),
                )])),
                Err(e) => Err(McpError::internal_error(
                    format!("error occurred: error creating page: {}", e),
                    None,
                )),
            },
            Err(e) => {
                // eprintln!("error finding ref db:: {}", e);
                // println!("will search for ref page to create a new db");
                if let Ok(page_id) = self.search_ref_page().await {
                    // println!("creating a new ref db!");
                    match self.create_ref_db(page_id.as_str()).await {
                        Ok(db_id) => {
                            if let Ok(resp) = self
                                .create_page(db_id.as_str(), title.as_str(), content.as_str())
                                .await
                            {
                                Ok(CallToolResult::success(vec![Content::text(
                                    resp.to_string(),
                                )]))
                            } else {
                                Err(McpError::internal_error(
                                    format!("error occurred: error creating page: {}", e),
                                    None,
                                ))
                            }
                        }
                        Err(e) => Err(McpError::internal_error(
                            format!("error occurred: error creating database: {}", e),
                            None,
                        )),
                    }
                } else {
                    Err(McpError::internal_error(
                        "error occurred: ref page not found",
                        None,
                    ))
                }
            }
        }
    }

    fn bundle_gitbook(&self, name: &str, content: Vec<GitBookChapter>) -> Result<std::path::PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = std::env::temp_dir().join(name);    
        fs::create_dir_all(&file_path)?;
        // Write README.md
        let readme_path = file_path.join("README.md");
        fs::write(&readme_path, "# My GitBook\nWelcome to my book!")?;
        // Write SUMMARY.md
        let summary_path = file_path.join("src/SUMMARY.md");
        if let Some(parent) = summary_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut summary = File::create(&summary_path)?;
        writeln!(summary, "# Summary")?;
        writeln!(summary, "* [Introduction](README.md)")?;
        // Write chapters
        for (i, chapter) in content.iter().enumerate() {
            let chapter_filename = format!("chapter_{}.md", i);
            writeln!(summary, "* [{}]({})", chapter.name, chapter_filename)?;
            let chapter_path = file_path.join(format!("src/{}", chapter_filename));
            fs::write(&chapter_path, &chapter.content)?;
        }
        // Run gitbook
        let status = Command::new("mdbook")
            .arg("build")
            .arg(&file_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;
    
        if status.success() {
            
            let path_clone = file_path.clone();
            Ok(path_clone)
        } else {
            Err("mdbook build failed".into())
        }
    }

    #[tool(description = "Create a mdbook")]
    async fn create_gitbook(
        &self,
        #[tool(aggr)] AddGitBook { name, content}: AddGitBook,
    ) -> Result<CallToolResult, McpError> {
      match self.bundle_gitbook(&name, content) {
        Ok(path_buf) => {
            // Command::new("gitbook").arg("serve").arg(path_buf);
            Ok(CallToolResult::success(vec![Content::text(
                format!("mdbook was created successfully at path: {}", path_buf.display()),
            )]))
        },
        Err(e) => {
            Err(McpError::internal_error(
                format!("error occurred: create the gitbook operation failed: {}", e),
                None,
            ))
        }
      }

    }
}

#[tool(tool_box)]
impl ServerHandler for Jotter {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides a tool to jot down your notes to Notion. You can create new pages on Notion or also bundle those notes into a gitbook".to_string()),
        }
    }
}
