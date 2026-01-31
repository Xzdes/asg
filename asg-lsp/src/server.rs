//! LSP Server implementation.

use std::collections::HashMap;
use std::sync::RwLock;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use asg_lang::asg::ASG;
use asg_lang::parser;

use crate::completion::get_completions;
use crate::definition::{find_definition, find_references};
use crate::diagnostics::get_diagnostics;
use crate::hover::get_hover_info;

/// Документ в редакторе.
#[derive(Debug, Clone)]
pub struct Document {
    /// Содержимое файла
    pub content: String,
    /// Распарсенный ASG (если парсинг успешен)
    pub asg: Option<ASG>,
    /// URI документа
    pub uri: Url,
}

/// ASG Language Server.
pub struct ASGLanguageServer {
    /// LSP клиент
    client: Client,
    /// Открытые документы
    documents: RwLock<HashMap<Url, Document>>,
}

impl ASGLanguageServer {
    /// Создать новый сервер.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: RwLock::new(HashMap::new()),
        }
    }

    /// Обновить документ и получить диагностику.
    async fn update_document(&self, uri: Url, content: String) {
        // Парсим документ
        let parse_result = parser::parse(&content);

        let (asg, diagnostics) = match parse_result {
            Ok((asg, _)) => (Some(asg), vec![]),
            Err(e) => (None, get_diagnostics(&e.to_string(), &content)),
        };

        // Сохраняем документ
        {
            let mut docs = self.documents.write().unwrap();
            docs.insert(
                uri.clone(),
                Document {
                    content,
                    asg,
                    uri: uri.clone(),
                },
            );
        }

        // Публикуем диагностику
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    /// Получить документ.
    fn get_document(&self, uri: &Url) -> Option<Document> {
        let docs = self.documents.read().unwrap();
        docs.get(uri).cloned()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for ASGLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["(".to_string(), " ".to_string()]),
                    resolve_provider: Some(false),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "asg-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "ASG LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.update_document(
            params.text_document.uri,
            params.text_document.text,
        )
        .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().last() {
            self.update_document(params.text_document.uri, change.text)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut docs = self.documents.write().unwrap();
        docs.remove(&params.text_document.uri);
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let doc = match self.get_document(&uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        let items = get_completions(&doc.content, position, doc.asg.as_ref());
        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let doc = match self.get_document(&uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        Ok(get_hover_info(&doc.content, position, doc.asg.as_ref()))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let doc = match self.get_document(&uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        Ok(find_definition(&doc.content, position, doc.asg.as_ref(), &uri))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let doc = match self.get_document(&uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        Ok(find_references(&doc.content, position, doc.asg.as_ref(), &uri))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        let doc = match self.get_document(&uri) {
            Some(d) => d,
            None => return Ok(None),
        };

        let symbols = get_document_symbols(&doc.content, doc.asg.as_ref());
        Ok(Some(DocumentSymbolResponse::Flat(symbols)))
    }
}

/// Получить символы документа.
fn get_document_symbols(content: &str, asg: Option<&ASG>) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();

    if let Some(asg) = asg {
        for node in &asg.nodes {
            use asg_lang::nodecodes::NodeType;

            let (name, kind) = match node.node_type {
                NodeType::Function => {
                    let name = node.get_name().unwrap_or_else(|| "<anonymous>".to_string());
                    (name, SymbolKind::FUNCTION)
                }
                NodeType::Variable => {
                    let name = node.get_name().unwrap_or_else(|| "<var>".to_string());
                    (name, SymbolKind::VARIABLE)
                }
                NodeType::Module => {
                    let name = node.get_name().unwrap_or_else(|| "<module>".to_string());
                    (name, SymbolKind::MODULE)
                }
                _ => continue,
            };

            // Вычисляем позицию из span
            let range = if let Some(span) = node.span {
                let start = offset_to_position(content, span.start);
                let end = offset_to_position(content, span.end);
                Range { start, end }
            } else {
                Range::default()
            };

            #[allow(deprecated)]
            symbols.push(SymbolInformation {
                name,
                kind,
                tags: None,
                deprecated: None,
                location: Location {
                    uri: Url::parse("file:///").unwrap(),
                    range,
                },
                container_name: None,
            });
        }
    }

    symbols
}

/// Конвертировать offset в Position.
fn offset_to_position(content: &str, offset: usize) -> Position {
    let mut line = 0u32;
    let mut col = 0u32;

    for (i, ch) in content.chars().enumerate() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    Position { line, character: col }
}
