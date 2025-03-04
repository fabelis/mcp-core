<p align="center">
    <img src="imgs/mcp_logo.png" alt="mcp_logo" style="width: 15%; margin-right:3%;" />
    <img src="imgs/plus.svg" alt="plus_svg" style="width: 10%; margin-bottom: 2%;" />
    <img src="imgs/rust_logo.png" alt="rust_logo" style="width: 15%; margin-left:3%;" />
</p>
<p align="center">
<h1 align="center">MCP Core</h1>
<p align="center">
A Rust library implementing the <a href="https://modelcontextprotocol.io/introduction">Modern Context Protocol (MCP)</a>
</p>
<p align="center">
<a href="https://github.com/stevohuncho/mcp-core"><img src="https://img.shields.io/github/stars/stevohuncho/mcp-core?style=social" alt="stars" /></a>
&nbsp;
<a href="https://crates.io/crates/mcp-core"><img src="https://img.shields.io/crates/v/mcp-core" alt="Crates.io" /></a>
&nbsp;
</p>

## Project Goals
Combine efforts with [Offical MCP Rust SDK](https://github.com/modelcontextprotocol/rust-sdk). The offical SDK repo is new and collaborations are in works to bring these features to the adopted platform.
- **Efficiency & Scalability**
  - Handles many concurrent connections with low overhead.
  - Scales easily across multiple nodes.
- **Security**
  - Strong authentication and authorization.
  - Built-in rate limiting and quota management.
- **Rust Advantages**
  - High performance and predictable latency.
  - Memory safety with no runtime overhead.

## Installation

Use the `cargo add` command to automatically add it to your `Cargo.toml`
```bash
cargo add mcp-core
```
Or add `mcp-core` to your `Cargo.toml` dependencies directly
```toml
[dependencies]
mcp-core = "0.1.0"
```

## Server Implementation
Easily start your own local SSE MCP Servers with tooling capabilities
```rs
sse-server.rs
use mcp_core::{
    run_http_server,
    server::Server,
    sse::http_server::Host,
    tool_error_response, tool_text_response,
    types::{CallToolRequest, CallToolResponse, ServerCapabilities, Tool, ToolResponseContent},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
enum PostDmError {
    #[error("Missing data")]
    MissingData,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(std::io::stderr)
        .init();

    run_http_server(
        Host {
            host: "127.0.0.1".to_string(),
            port: 8080,
            public_url: None,
        },
        None,
        |transport| async move {
            let mut server_builder = Server::builder(transport)
                .capabilities(ServerCapabilities {
                    tools: Some(json!({
                        "listChanged": false,
                    })),
                    ..ServerCapabilities::default()
                })
                .version("0.1.0")
                .name("Example SSE Server");

            server_builder.register_tool(
                Tool {
                    name: "test".to_string(),
                    description: Some("Test Tool".to_string()),
                    input_schema: json!({
                       "type":"object",
                       "properties":{
                          "test_data":{
                             "type": "string",
                             "description": "Test data",
                          }
                       },
                       "required":["test_data"]
                    }),
                },
                move |req: CallToolRequest| {
                    Box::pin(async move {
                        let args = req.arguments.unwrap_or_default();
                        let data = args.get("test_data");

                        if data.is_none() {
                            return tool_error_response!(PostDmError::MissingData);
                        };

                        tool_text_response!(json!(data).to_string())
                    })
                },
            );

            Ok(server_builder.build())
        },
    )
    .await?;

    Ok(())
}
```
### Public SSE MCP Servers
If you want to expose this MCP Server publicly via a reverse proxy make sure you set the `public_url` to your domain.
```rs
Host {
    host: "127.0.0.1".to_string(),
    port: 8080,
    public_url: Some("https://my.mcpdomain.com".to_string()),
}
```
### SSE Server Authentication
Client connections are authenticated using **JWT** middleware via `mcp_core::sse::middleware`. Enable this by setting the `auth_config` of your server to include the secret to verify claims.
```rs
pub struct AuthConfig {
    pub jwt_secret: String,
}
```
## SSE Client Connection
Connect to an SSE MCP Server using the `ClientSseTransport`. Here is an example of connecting to one and listing the tools from that server.
```rs
let transport = ClientSseTransport::builder("https://my.mcpdomain.com".to_string()).build();
transport.open().await?;

let mcp_client = Arc::new(McpClient::builder(transport).use_strict().build());

let mcp_client_clone = mcp_client.clone();
tokio::spawn(async move { mcp_client_clone.start().await });

mcp_client
    .initialize(Implementation {
        name: "mcp-client".to_string(),
        version: "0.1.0".to_string(),
    })
    .await?;

let tools = mcp_client.list_tools(None, None).await?.tools;

println!("Tools: {:?}", tools);
```

### Setting `SecureValues` to your SSE MCP Client
Have API Keys or Secrets needed to be passed to MCP Tool Calls, but you don't want to pass this information to the LLM you are prompting? Use `mcp_core::client::SecureValue`!
```rs
McpClient::builder(transport)
    .with_secure_value(
        "discord_token",
        mcp_core::client::SecureValue::Static(discord_token),
    )
    .with_secure_value(
        "anthropic_api_key",
        mcp_core::client::SecureValue::Env("ANTHROPIC_API_KEY".to_string()),
    )
    .use_strict()
    .build()
```
#### mcp_core::client::SecureValue::Static
Automatically have **MCP Tool Call Parameters** be replaced by the string value set to it.
#### mcp_core::client::SecureValue::Env
Automatically have **MCP Tool Call Parameters** be replaced by the value in your `.env` from the string set to it.

### SSE MCP Client Tool Call
```rs
let response = mcp_client
    .call_tool(
        "PostMessage",
        Some(json!({
            "content": "Hello, world!"
        })),
    )
    .await?;

println!("Response: {:?}", response);
```
