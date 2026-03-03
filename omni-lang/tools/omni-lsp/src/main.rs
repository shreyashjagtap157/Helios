//! Omni Language Server (LSP)
//!
//! A Language Server Protocol implementation for the Omni programming language,
//! built on tower-lsp. Provides diagnostics, completion, and document sync.

use log::info;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use std::sync::Mutex;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Omni keywords – returned as completions
// ---------------------------------------------------------------------------
const OMNI_KEYWORDS: &[&str] = &[
    "module", "struct", "fn", "let", "mut", "if", "else", "for", "in",
    "while", "return", "match", "async", "await", "import", "as", "own",
    "shared", "unsafe", "trait", "impl", "implements", "const", "true",
    "false", "extern", "comptime", "enum", "type", "where", "defer", "pub",
    "var", "break", "continue", "loop", "yield", "Self", "None", "Some",
    "Ok", "Err", "dyn", "spawn", "select", "case", "macro", "pass",
    "try", "catch", "finally", "elif", "self",
];

// ---------------------------------------------------------------------------
// Backend – thin wrapper around the omnc compiler
// ---------------------------------------------------------------------------
struct Backend {
    client: Client,
    documents: Mutex<HashMap<Url, String>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: Mutex::new(HashMap::new()),
        }
    }

    /// Run the omnc lexer + parser on `source` and convert errors to LSP diagnostics.
    fn diagnose(&self, source: &str) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // --- Lexer pass ---
        let tokens = match omni_compiler::lexer::tokenize(source) {
            Ok(tokens) => tokens,
            Err(e) => {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 1),
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("omnc-lexer".into()),
                    message: format!("Lexer error: {e:?}"),
                    ..Default::default()
                });
                return diagnostics;
            }
        };

        // --- Parser pass ---
        match omni_compiler::parser::parse(tokens) {
            Ok(_module) => { /* no errors */ }
            Err(e) => {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 1),
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("omnc-parser".into()),
                    message: format!("Parse error: {e:?}"),
                    ..Default::default()
                });
            }
        }

        diagnostics
    }

    /// Publish diagnostics for a document.
    async fn publish_diagnostics(&self, uri: Url) {
        let source = {
            let docs = self.documents.lock().unwrap();
            match docs.get(&uri) {
                Some(s) => s.clone(),
                None => return,
            }
        };
        let diags = self.diagnose(&source);
        self.client
            .publish_diagnostics(uri, diags, None)
            .await;
    }
}

// ---------------------------------------------------------------------------
// LanguageServer trait implementation
// ---------------------------------------------------------------------------
#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        info!("omni-lsp: initialize");
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".into(), ":".into()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "omni-lsp".into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        info!("omni-lsp: server initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        info!("omni-lsp: shutdown");
        Ok(())
    }

    // -- Document synchronization ------------------------------------------

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        {
            let mut docs = self.documents.lock().unwrap();
            docs.insert(uri.clone(), params.text_document.text);
        }
        self.publish_diagnostics(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        if let Some(change) = params.content_changes.into_iter().last() {
            let mut docs = self.documents.lock().unwrap();
            docs.insert(uri.clone(), change.text);
        }
        self.publish_diagnostics(uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        self.publish_diagnostics(uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut docs = self.documents.lock().unwrap();
        docs.remove(&params.text_document.uri);
    }

    // -- Completion --------------------------------------------------------

    async fn completion(
        &self,
        _params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let items: Vec<CompletionItem> = OMNI_KEYWORDS
            .iter()
            .map(|kw| CompletionItem {
                label: kw.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Omni keyword".into()),
                ..Default::default()
            })
            .collect();
        Ok(Some(CompletionResponse::Array(items)))
    }

    // -- Hover (stub) ------------------------------------------------------

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let docs = self.documents.lock().unwrap();
        if let Some(source) = docs.get(&uri) {
            // Find the word under the cursor
            let lines: Vec<&str> = source.lines().collect();
            if let Some(line) = lines.get(pos.line as usize) {
                let col = pos.character as usize;
                let start = line[..col]
                    .rfind(|c: char| !c.is_alphanumeric() && c != '_')
                    .map(|i| i + 1)
                    .unwrap_or(0);
                let end = line[col..]
                    .find(|c: char| !c.is_alphanumeric() && c != '_')
                    .map(|i| i + col)
                    .unwrap_or(line.len());
                let word = &line[start..end];

                if OMNI_KEYWORDS.contains(&word) {
                    return Ok(Some(Hover {
                        contents: HoverContents::Scalar(MarkedString::String(
                            format!("`{word}` — Omni keyword"),
                        )),
                        range: Some(Range {
                            start: Position::new(pos.line, start as u32),
                            end: Position::new(pos.line, end as u32),
                        }),
                    }));
                }
            }
        }
        Ok(None)
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------
#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Starting Omni Language Server…");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
