use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use rmcp::{Error as McpError, ServerHandler, model::*, schemars, tool};

use crate::notion::Notion;
use crate::formatter::{split_content, format_for_notion};

// Maximum size of a block in the Notion API
const MAX_BLOCK_SIZE: usize = 2000;

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
pub struct AddMdBook {
    pub name: String,
    pub content: Vec<MdBookChapter>
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MdBookChapter {
    pub name: String,
    pub content: String
}


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
        let ref_page_name = "Jot It Down";
        match self.data_store.search_ref(ref_page_name, "page").await {
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


    fn bundle_mdbook(&self, name: &str, content: Vec<MdBookChapter>) -> Result<std::path::PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let file_path = std::env::home_dir().unwrap().join(name);
        fs::create_dir_all(&file_path)?;
        // Write README.md
        let readme_path = file_path.join("README.md");
        fs::write(&readme_path, "# My MdBook\nWelcome to my book!")?;
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
        let path_clone = file_path.clone();
        Ok(path_clone)
    }

    async fn open_mdbook (&self, book_path: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let dir = Path::new(book_path.as_str());

            let mut child = Command::new("mdbook")
            .arg("serve")
            .arg("-o")
            .current_dir(dir)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

            child.wait()?;

            Ok(())
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
        // Split and format the content
        let content_chunks = split_content(&content, MAX_BLOCK_SIZE);
        let mut all_blocks = Vec::new();
        
        for chunk in content_chunks {
            all_blocks.extend(format_for_notion(&chunk));
        }
        
        // Use the new update_page_with_blocks method
        match self.data_store.update_page_with_blocks(page_id.as_str(), &all_blocks).await {
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
        // Split and format the content
        let content_chunks = split_content(&content, MAX_BLOCK_SIZE);
        let mut all_blocks = Vec::new();
        
        for chunk in content_chunks {
            all_blocks.extend(format_for_notion(&chunk));
        }
        
        match self.search_ref_db().await {
            Ok(db_id) => {
                // Use the new create_page_with_blocks method
                match self.data_store.create_page_with_blocks(&db_id, &title, &all_blocks).await {
                    Ok((_, json_resp)) => Ok(CallToolResult::success(vec![Content::text(
                        json_resp.to_string(),
                    )])),
                    Err(e) => Err(McpError::internal_error(
                        format!("error occurred: error creating page: {}", e),
                        None,
                    )),
                }
            },
            Err(_e) => {
                if let Ok(page_id) = self.search_ref_page().await {
                    match self.create_ref_db(page_id.as_str()).await {
                        Ok(db_id) => {
                            match self.data_store.create_page_with_blocks(&db_id, &title, &all_blocks).await {
                                Ok((_, json_resp)) => Ok(CallToolResult::success(vec![Content::text(
                                    json_resp.to_string(),
                                )])),
                                Err(e) => Err(McpError::internal_error(
                                    format!("error occurred: error creating page: {}", e),
                                    None,
                                )),
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

    #[tool(description = "Create an mdbook for the given name and content")]
    async fn create_mdbook(
        &self,
        #[tool(aggr)] AddMdBook { name, content}: AddMdBook,
    ) -> Result<CallToolResult, McpError> {
      match self.bundle_mdbook(&name, content) {
        Ok(path_buf) => {
            Ok(CallToolResult::success(vec![Content::text(
                format!("File created at: {}, now please run mdbook serve -o to serve it", path_buf.display()),
            )]))
        },
        Err(e) => {
            Err(McpError::internal_error(
                format!("error occurred: create the mdbook operation failed: {}", e),
                None,
            ))
        }
      }
    }

    #[tool(description = "Serve mdbook from a given path")]
    async fn serve_mdbook(&self, #[tool(param)] path: String) -> Result<CallToolResult, McpError> {
        match self.open_mdbook(path.clone()).await {
            Ok(_) => {
                Ok(CallToolResult ::success(vec![Content::text(
                    format!("book served successfully"),
                )]))
            },
            Err(e) => {
                Err(McpError::internal_error(
                    format!("error occurred: {}, since we are facing the issue ask user to run mdbook serve -o in the path {} to serve the book", e, path.clone()),
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
            instructions: Some("This server provides a tool to jot down your notes to Notion. You can create new pages on Notion or also bundle those notes into a mdbook".to_string()),
        }
    }
}
