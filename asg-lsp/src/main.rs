//! ASG Language Server
//!
//! LSP сервер для языка ASG.

use tower_lsp::{LspService, Server};

mod server;
mod diagnostics;
mod completion;
mod hover;
mod definition;

use server::ASGLanguageServer;

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| ASGLanguageServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
