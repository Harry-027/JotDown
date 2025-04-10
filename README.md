# Jotdown - MCP Server for Notion Page Creation and mdBook Generation

Jotdown is a Model Context Protocol (MCP) server that allows large language models (LLMs) to interact with **Notion** and also generate **Markdown Books**. It provides two primary tools for LLMs:

- 👉  **Notion Integration**: Create or update pages in **Notion** with content generated by the LLM.
- 👉  **Mdbook Generation**: Generate a **mdbook** from content and manage the structure.

Jotdown enables LLMs to seamlessly integrate with these systems for various content management and publishing workflows.

---

## Features

- 🌿 **Notion Integration**: Automatically create or update Notion pages with content generated by the LLM.
- 🌿 **mdbook Generation**: Create and manage **mdbooks** directly from content, including generating necessary files like `SUMMARY.md`, `README.md`, and individual chapter markdown files.
- 🌿 **MCP Support**: Leverages the Model Context Protocol to maintain context over interactions, enabling more intelligent and consistent content creation and updates.

---

## Tools Provided by Jotdown

### 1. **Notion Page Tool**
   LLMs can use the Notion tool to create or update pages within Notion, allowing them to store structured content like articles or any other type of document directly in Notion.

   #### Example:
   - Create a new page in Notion with content that the LLM has generated.
   - Update an existing Notion page with new information.

### 2. **mdbook Tool**
   With the mdbook tool, LLMs can automatically generate markdown books, handling the creation of multiple chapters, managing the structure, and adding a `SUMMARY.md` file for navigation.

   #### Example:
   - Generate a new book based on LLM-generated content.
   - Automatically generate chapters with proper links in `SUMMARY.md`.

---

## Installation

### Prerequisites

- **Rust**: Make sure you have Rust installed. You can install it from [rust-lang.org](https://www.rust-lang.org/).
- **Notion API Token**: You will need a Notion API token (`internal integration secret`) to integrate with Notion.
- **Claude Desktop**: Make sure you have Claude desktop or any other MCP client (Cline, Continue etc) installed and configured with a Notion integration token.

### Steps to Install

1. Clone the repository:

    ```bash
    git clone https://github.com/Harry-027/JotDown
    cd jotdown
    ```

2. Install dependencies:

    ```bash
    cargo build --release
    ```

3. Install mdbook cli (required for the book generation to work):

    ```bash
    cargo install mdbook
    ```

4. Notion setup:

    * Setup Notion connection (internal intergation with access to Notion workspace) & copy the `internal intergration secret` for later use.
    * Setup a page with title `Jot It Down` in your workspace and share it with your connection. This is required for the integration to work.

5. Set up Claude desktop (or any other MCP client) configuration file (for Notion integration):
    ```json
      "mcpServers": {
            "Jotdown": {
                "command": "/path_to_repo/Jotdown/target/release/Jotdown",
                "args": [],
                "env": {
                    "NOTION_TOKEN": "your_notion_intergration_token"
                }
            }
      }
    ```

6. Restart Claude desktop and try it out!

---

### Notion Integration Example:

- To create or update a Notion page, the LLM sends a request to the server specifying the content and page details. The server then interacts with the Notion API to either create a new page or update an existing one.

### Mdbook Integration Example:

- LLMs can send structured content to the server to create an entire mdbook, including chapter creation, `README.md`, and `SUMMARY.md` management. The server compiles the content into a complete book.

---

## 🧑‍💻 Demo

### Notion Demo -

![Demo Notion](./demo/demo_1.gif)

### MdBook Demo -

![Demo mdbook](./demo/demo_2.gif)

### MdBook Screenshots -

![Demo mdbook screenshot 1](./demo/demo_s_1.png)
![Demo mdbook screenshot 2](./demo/demo_s_2.png)

---

## 🧑‍💻 Contributing

Feel free to open issues or submit pull requests.

---

## 📜 License

Jotdown is licensed under the **MIT License**. See the LICENSE file for details.

---

## 📧 Contact

For support or inquiries, reach out at [harishmmp@gmail.com](mailto:harishmmp@gmail.com).
