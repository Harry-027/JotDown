use serde_json::{json, Value};
use regex::Regex;

/// Maximum size of a block in the Notion API
const MAX_BLOCK_SIZE: usize = 2000;

/// Split content into chunks to respect Notion API limits
///
/// # Arguments
///
/// * `text` - Text to be split
/// * `max_length` - Maximum length of each chunk (default: 2000 characters)
///
/// # Returns
///
/// * `Vec<String>` - List of text chunks
pub fn split_content(text: &str, max_length: usize) -> Vec<String> {
    if text.len() <= max_length {
        return vec![text.to_string()];
    }
    
    // Try to split by headers
    let header_regex = Regex::new(r"(?m)^(#{1,3}\s.+)$").unwrap();
    let headers: Vec<_> = header_regex.find_iter(text).collect();
    
    if headers.is_empty() {
        // No headers, use simple method
        return simple_split(text, max_length);
    }
    
    let mut parts = Vec::new();
    let mut last_pos = 0;
    let mut current_chunk = String::new();
    
    // Process headers as splitting points
    for (i, header_match) in headers.iter().enumerate() {
        // Get content from last point to current header
        if i > 0 {
            let header_content = &text[last_pos..header_match.start()];
            
            // If adding this header section would exceed max size,
            // start a new chunk
            if current_chunk.len() + header_content.len() > max_length {
                parts.push(current_chunk.clone());
                current_chunk = header_content.to_string();
            } else {
                current_chunk.push_str(header_content);
            }
        }
        
        // First header or after a split
        if current_chunk.is_empty() {
            current_chunk = text[header_match.start()..].to_string();
            // If still too large, we'll need to split it later
        }
        
        last_pos = header_match.start();
    }
    
    // Add final chunk
    if last_pos < text.len() {
        let final_content = &text[last_pos..];
        if current_chunk.len() + final_content.len() > max_length {
            parts.push(current_chunk.clone());
            parts.push(final_content.to_string());
        } else {
            current_chunk.push_str(final_content);
            parts.push(current_chunk.clone());
        }
    } else if !current_chunk.is_empty() {
        parts.push(current_chunk.clone());
    }
    
    // If any chunk is still too large, split it further
    let mut result = Vec::new();
    for chunk in parts {
        if chunk.len() > max_length {
            result.extend(simple_split(&chunk, max_length));
        } else {
            result.push(chunk);
        }
    }
    
    result
}

/// Fallback method to split text without headers
///
/// # Arguments
///
/// * `text` - Text to be split
/// * `max_length` - Maximum length of each chunk
///
/// # Returns
///
/// * `Vec<String>` - List of text chunks
fn simple_split(text: &str, max_length: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = String::new();
    let mut in_code_block = false;
    let mut code_block_content = String::new();
    
    for line in text.split('\n') {
        // Check for code block markers
        if line.trim().starts_with("```") {
            in_code_block = !in_code_block;
            
            // If we're starting a code block
            if in_code_block {
                code_block_content = format!("{}{}", line, "\n");
                continue;
            } else {
                // We're ending a code block, add it as a whole
                code_block_content.push_str(line);
                if current_chunk.len() + code_block_content.len() > max_length {
                    // If adding the whole block exceeds the limit,
                    // finalize the current chunk and start a new one
                    if !current_chunk.is_empty() {
                        chunks.push(current_chunk.clone());
                    }
                    chunks.push(code_block_content.clone());
                    current_chunk = String::new();
                } else {
                    current_chunk.push_str(&code_block_content);
                }
                code_block_content = String::new();
                continue;
            }
        }
        
        // If we're inside a code block, collect the content
        if in_code_block {
            code_block_content.push_str(line);
            code_block_content.push('\n');
            continue;
        }
        
        // For regular lines
        let line_with_newline = format!("{}{}", line, "\n");
        if current_chunk.len() + line_with_newline.len() > max_length {
            if !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());
            }
            current_chunk = line_with_newline;
        } else {
            current_chunk.push_str(&line_with_newline);
        }
    }
    
    // Add any remaining content
    if !code_block_content.is_empty() {
        if current_chunk.len() + code_block_content.len() > max_length {
            if !current_chunk.is_empty() {
                chunks.push(current_chunk.clone());
            }
            chunks.push(code_block_content.clone());
        } else {
            current_chunk.push_str(&code_block_content);
        }
    }
    
    if !current_chunk.is_empty() {
        chunks.push(current_chunk.clone());
    }
    
    chunks
}

/// Convert Markdown text into Notion blocks
///
/// # Arguments
///
/// * `text` - Markdown text to be converted
///
/// # Returns
///
/// * `Vec<Value>` - List of Notion blocks
pub fn format_for_notion(text: &str) -> Vec<Value> {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut blocks = Vec::new();
    let mut current_code_block: Option<Value> = None;
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i].trim_end();
        i += 1;
        
        // Detect start of code block (```language)
        if let Some(code_lang) = line.strip_prefix("```") {
            if current_code_block.is_none() {
                // Start a new code block
                current_code_block = Some(json!({
                    "type": "code",
                    "code": {
                        "rich_text": [],
                        "language": get_valid_notion_language(code_lang.trim())
                    }
                }));
                continue;
            }
        }
        
        // Detect end of code block
        if line.trim() == "```" && current_code_block.is_some() {
            // Close the current code block
            blocks.push(current_code_block.take().unwrap());
            continue;
        }
        
        // Add lines to current code block
        if let Some(ref mut code_block) = current_code_block {
            let line_with_newline = format!("{}{}", line, "\n");
            code_block["code"]["rich_text"].as_array_mut().unwrap().push(json!({
                "type": "text",
                "text": { "content": line_with_newline }
            }));
            continue;
        }
        
        // Ignore empty lines outside code blocks
        if line.trim().is_empty() {
            // Add a paragraph with a newline for spacing
            blocks.push(json!({
                "type": "paragraph",
                "paragraph": { "rich_text": [] }
            }));
            continue;
        }
        
        // Headers
        if line.starts_with("# ") {
            blocks.push(json!({
                "type": "heading_1",
                "heading_1": { "rich_text": [{ "text": { "content": &line[2..] } }] }
            }));
        } else if line.starts_with("## ") {
            blocks.push(json!({
                "type": "heading_2",
                "heading_2": { "rich_text": [{ "text": { "content": &line[3..] } }] }
            }));
        } else if line.starts_with("### ") {
            blocks.push(json!({
                "type": "heading_3",
                "heading_3": { "rich_text": [{ "text": { "content": &line[4..] } }] }
            }));
        } 
        // Bulleted list
        else if line.starts_with("- ") || line.starts_with("* ") {
            let content = &line[2..];
            blocks.push(json!({
                "type": "bulleted_list_item",
                "bulleted_list_item": { "rich_text": [{ "text": { "content": content } }] }
            }));
        }
        // Numbered list
        else if Regex::new(r"^\d+\.\s").unwrap().is_match(line) {
            let content = Regex::new(r"^\d+\.\s").unwrap().replace(line, "");
            blocks.push(json!({
                "type": "numbered_list_item",
                "numbered_list_item": { "rich_text": [{ "text": { "content": content } }] }
            }));
        }
        // Regular paragraphs
        else {
            blocks.push(json!({
                "type": "paragraph",
                "paragraph": { "rich_text": [{ "text": { "content": line } }] }
            }));
        }
    }
    
    // Close any remaining code block
    if let Some(code_block) = current_code_block {
        blocks.push(code_block);
    }
    
    blocks
}

fn get_valid_notion_language(language: &str) -> &str {
    // List of languages supported by the Notion API
    let valid_languages = [
        "abap", "agda", "arduino", "assembly", "bash", "basic", "c", "c#", "c++", 
        "clojure", "coffeescript", "css", "dart", "diff", "docker", "elixir", 
        "elm", "erlang", "f#", "flow", "fortran", "go", "graphql", "groovy", 
        "haskell", "html", "java", "javascript", "json", "julia", "kotlin", "latex", 
        "less", "lisp", "lua", "makefile", "markdown", "matlab", "mermaid", 
        "nix", "objective-c", "ocaml", "pascal", "perl", "php", "python", 
        "r", "ruby", "rust", "scala", "scheme", "scss", "shell", "sql", 
        "swift", "typescript", "vb.net", "verilog", "vhdl", "xml", "yaml"
    ];
    
    // Normalize the language name
    let normalized = language.trim().to_lowercase();
    
    if valid_languages.contains(&normalized.as_str()) {
        for &valid in &valid_languages {
            if valid == normalized {
                return valid;
            }
        }
    }
    
    if normalized.is_empty() {
        return "plain text";
    } else {
        // Try to find a close match
        match normalized.as_str() {
            "js" | "jsx" => "javascript",
            "py" => "python",
            "ts" | "tsx" => "typescript",
            "sh" | "zsh" => "shell",
            "md" => "markdown",
            _ => "plain text"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_split_content_small_text() {
        let text = "This is a small text.";
        let chunks = split_content(text, 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], text);
    }
    
    #[test]
    fn test_split_content_large_text_no_headers() {
        let text = "A".repeat(3000); // Large text without headers
        let chunks = split_content(&text, 1000);
        assert!(chunks.len() > 1);
        for chunk in chunks {
            assert!(chunk.len() <= 1000);
        }
    }
    
    #[test]
    fn test_split_content_with_headers() {
        let text = format!(
            "# Title 1\n{}\n\n## Title 2\n{}\n\n### Title 3\n{}",
            "A".repeat(900),
            "B".repeat(900),
            "C".repeat(900)
        );
        let chunks = split_content(&text, 1000);
        assert!(chunks.len() > 1);
        for chunk in chunks {
            assert!(chunk.len() <= 1000);
        }
    }
    
    #[test]
    fn test_format_for_notion_heading() {
        let text = "# Main Title";
        let blocks = format_for_notion(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["type"], "heading_1");
        assert_eq!(blocks[0]["heading_1"]["rich_text"][0]["text"]["content"], "Main Title");
    }
    
    #[test]
    fn test_format_for_notion_paragraph() {
        let text = "This is a normal paragraph.";
        let blocks = format_for_notion(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0]["type"], "paragraph");
        assert_eq!(blocks[0]["paragraph"]["rich_text"][0]["text"]["content"], text);
    }
}