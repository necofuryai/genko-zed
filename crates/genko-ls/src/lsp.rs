use std::collections::HashMap;

use tokio::sync::RwLock;
use tower_lsp_server::{
    LanguageServer, LspService, Server,
    jsonrpc::Result,
    ls_types::{
        DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, Hover,
        HoverContents, HoverParams, HoverProviderCapability, InitializeParams, InitializeResult,
        MarkupContent, MarkupKind, ServerCapabilities, ServerInfo, TextDocumentSyncCapability,
        TextDocumentSyncKind, Uri,
    },
};

use crate::novel_metrics;

#[derive(Debug, Default)]
struct Backend {
    documents: RwLock<HashMap<Uri, String>>,
}

impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "genko-ls".to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
            offset_encoding: None,
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let document = params.text_document;
        self.documents
            .write()
            .await
            .insert(document.uri, document.text);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Some(change) = params.content_changes.into_iter().last() else {
            return;
        };

        let mut documents = self.documents.write().await;
        if let Some(document) = documents.get_mut(&params.text_document.uri) {
            *document = change.text;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents
            .write()
            .await
            .remove(&params.text_document.uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let source = self.documents.read().await.get(uri).cloned();
        let Some(source) = source else {
            return Ok(None);
        };

        let metrics = novel_metrics(&source);
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**本文文字数:** {} 文字", metrics.body_character_count()),
            }),
            range: None,
        }))
    }
}

/// Runs genko-ls over standard input and standard output until the client exits.
pub async fn run_stdio() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|_| Backend::default());

    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tower_lsp_server::ls_types::{
        DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        HoverContents, HoverParams, HoverProviderCapability, InitializeParams, MarkupKind,
        TextDocumentSyncCapability, TextDocumentSyncKind,
    };

    use super::*;

    #[tokio::test]
    async fn advertises_full_sync_and_hover() {
        let backend = Backend::default();
        let initialize: InitializeParams = serde_json::from_value(json!({
            "processId": null,
            "capabilities": {}
        }))
        .expect("initialize payload should be valid");

        let result = backend
            .initialize(initialize)
            .await
            .expect("initialize should succeed");

        assert_eq!(
            result.capabilities.text_document_sync,
            Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL))
        );
        assert_eq!(
            result.capabilities.hover_provider,
            Some(HoverProviderCapability::Simple(true))
        );
    }

    #[tokio::test]
    async fn full_sync_lifecycle_updates_hover_and_close_removes_document() {
        let backend = Backend::default();

        let open: DidOpenTextDocumentParams = serde_json::from_value(json!({
            "textDocument": {
                "uri": "file:///story.genko",
                "languageId": "genko-novel",
                "version": 1,
                "text": "前｜猫《ねこ》後"
            }
        }))
        .expect("didOpen payload should be valid");
        backend.did_open(open).await;

        assert_eq!(
            hover_markdown(&backend).await.as_deref(),
            Some("**本文文字数:** 3 文字")
        );

        let change: DidChangeTextDocumentParams = serde_json::from_value(json!({
            "textDocument": {
                "uri": "file:///story.genko",
                "version": 2
            },
            "contentChanges": [{
                "text": "一\n二"
            }]
        }))
        .expect("didChange payload should be valid");
        backend.did_change(change).await;

        assert_eq!(
            hover_markdown(&backend).await.as_deref(),
            Some("**本文文字数:** 2 文字")
        );

        let close: DidCloseTextDocumentParams = serde_json::from_value(json!({
            "textDocument": {
                "uri": "file:///story.genko"
            }
        }))
        .expect("didClose payload should be valid");
        backend.did_close(close).await;

        assert_eq!(hover_markdown(&backend).await, None);
    }

    async fn hover_markdown(backend: &Backend) -> Option<String> {
        let hover: HoverParams = serde_json::from_value(json!({
            "textDocument": {
                "uri": "file:///story.genko"
            },
            "position": {
                "line": 999,
                "character": 999
            }
        }))
        .expect("hover payload should be valid");

        let response = backend.hover(hover).await.expect("hover should succeed")?;
        match response.contents {
            HoverContents::Markup(content) => {
                assert_eq!(content.kind, MarkupKind::Markdown);
                Some(content.value)
            }
            HoverContents::Scalar(_) | HoverContents::Array(_) => None,
        }
    }
}
