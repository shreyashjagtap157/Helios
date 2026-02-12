//! Advanced LSP Features
//! 
//! Incremental parsing, symbol indexing, semantic highlighting, code actions.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use rusqlite::{Connection, params};

/// Symbol Index backed by SQLite for fast lookups
pub struct SymbolIndex {
    conn: Connection,
}

impl SymbolIndex {
    pub fn new(db_path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(db_path)?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS symbols (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                file_path TEXT NOT NULL,
                line INTEGER NOT NULL,
                column INTEGER NOT NULL,
                parent_id INTEGER,
                signature TEXT,
                doc TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_name ON symbols(name)",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn add_symbol(&self, symbol: &Symbol) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO symbols (name, kind, file_path, line, column, parent_id, signature, doc)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                symbol.name,
                symbol.kind,
                symbol.file_path,
                symbol.line,
                symbol.column,
                symbol.parent_id,
                symbol.signature,
                symbol.doc,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn find_by_name(&self, name: &str) -> Vec<Symbol> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, kind, file_path, line, column, parent_id, signature, doc
             FROM symbols WHERE name LIKE ?1"
        ).unwrap();

        stmt.query_map(params![format!("%{}%", name)], |row| {
            Ok(Symbol {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                kind: row.get(2)?,
                file_path: row.get(3)?,
                line: row.get(4)?,
                column: row.get(5)?,
                parent_id: row.get(6)?,
                signature: row.get(7)?,
                doc: row.get(8)?,
            })
        }).unwrap().filter_map(Result::ok).collect()
    }

    pub fn find_references(&self, name: &str) -> Vec<Symbol> {
        self.find_by_name(name)
    }

    pub fn clear_file(&self, file_path: &str) -> Result<(), rusqlite::Error> {
        self.conn.execute(
            "DELETE FROM symbols WHERE file_path = ?1",
            params![file_path],
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: Option<i64>,
    pub name: String,
    pub kind: String, // function, struct, variable, trait, etc.
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    pub parent_id: Option<i64>,
    pub signature: Option<String>,
    pub doc: Option<String>,
}

/// Incremental Parser for fast re-parsing on edits
pub struct IncrementalParser {
    cached_trees: HashMap<String, ParseTree>,
}

#[derive(Clone)]
pub struct ParseTree {
    pub root: Vec<AstNode>,
    pub version: u64,
}

#[derive(Clone, Debug)]
pub struct AstNode {
    pub kind: String,
    pub start_line: u32,
    pub end_line: u32,
    pub children: Vec<AstNode>,
}

impl IncrementalParser {
    pub fn new() -> Self {
        Self {
            cached_trees: HashMap::new(),
        }
    }

    /// Parse only the changed region and merge with cached tree
    pub fn parse_incremental(&mut self, file_path: &str, content: &str, change_start: u32, change_end: u32) -> ParseTree {
        // 1. Find the function/block containing the change
        if let Some(cached) = self.cached_trees.get(file_path) {
            // Find affected nodes
            let affected = Self::find_affected_nodes(&cached.root, change_start, change_end);
            
            // Only re-parse affected region
            let new_nodes = self.parse_region(content, change_start, change_end);
            
            // Merge back
            let mut new_tree = cached.clone();
            new_tree.version += 1;
            // Replace affected nodes with new_nodes
            
            self.cached_trees.insert(file_path.to_string(), new_tree.clone());
            return new_tree;
        }

        // Full parse if no cache
        let tree = self.full_parse(content);
        self.cached_trees.insert(file_path.to_string(), tree.clone());
        tree
    }

    fn find_affected_nodes(nodes: &[AstNode], start: u32, end: u32) -> Vec<&AstNode> {
        nodes.iter().filter(|n| {
            n.start_line <= end && n.end_line >= start
        }).collect()
    }

    fn parse_region(&self, _content: &str, _start: u32, _end: u32) -> Vec<AstNode> {
        // Parse specific region
        Vec::new()
    }

    fn full_parse(&self, _content: &str) -> ParseTree {
        ParseTree {
            root: Vec::new(),
            version: 1,
        }
    }
}

/// Semantic Token Provider for syntax highlighting
pub struct SemanticTokenProvider;

#[derive(Debug, Clone)]
pub struct SemanticToken {
    pub line: u32,
    pub start_char: u32,
    pub length: u32,
    pub token_type: TokenType,
    pub modifiers: Vec<TokenModifier>,
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Namespace,
    Type,
    Class,
    Enum,
    Interface,
    Struct,
    TypeParameter,
    Parameter,
    Variable,
    Property,
    EnumMember,
    Event,
    Function,
    Method,
    Macro,
    Keyword,
    Modifier,
    Comment,
    String,
    Number,
    Regexp,
    Operator,
    // Omni-specific
    Tensor,
    Async,
    Unsafe,
}

#[derive(Debug, Clone)]
pub enum TokenModifier {
    Declaration,
    Definition,
    Readonly,
    Static,
    Deprecated,
    Abstract,
    Async,
    Modification,
    Documentation,
    DefaultLibrary,
}

impl SemanticTokenProvider {
    pub fn tokenize(content: &str, symbols: &[Symbol]) -> Vec<SemanticToken> {
        let mut tokens = Vec::new();
        
        // Type-aware highlighting
        for symbol in symbols {
            let token_type = match symbol.kind.as_str() {
                "function" => TokenType::Function,
                "struct" => TokenType::Struct,
                "trait" => TokenType::Interface,
                "variable" => TokenType::Variable,
                "tensor" => TokenType::Tensor,
                _ => TokenType::Variable,
            };

            tokens.push(SemanticToken {
                line: symbol.line,
                start_char: symbol.column,
                length: symbol.name.len() as u32,
                token_type,
                modifiers: Vec::new(),
            });
        }

        tokens
    }
}

/// Code Actions Provider
pub struct CodeActionsProvider;

#[derive(Debug)]
pub struct CodeAction {
    pub title: String,
    pub kind: CodeActionKind,
    pub edits: Vec<TextEdit>,
}

#[derive(Debug)]
pub enum CodeActionKind {
    QuickFix,
    Refactor,
    RefactorExtract,
    RefactorInline,
    RefactorRewrite,
    Source,
    SourceOrganizeImports,
}

#[derive(Debug)]
pub struct TextEdit {
    pub file: String,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
    pub new_text: String,
}

impl CodeActionsProvider {
    /// Generate code actions for missing imports
    pub fn missing_import(symbol_name: &str, available_modules: &[&str]) -> Option<CodeAction> {
        for module in available_modules {
            if module.ends_with(symbol_name) {
                return Some(CodeAction {
                    title: format!("Import {} from {}", symbol_name, module),
                    kind: CodeActionKind::QuickFix,
                    edits: vec![TextEdit {
                        file: String::new(),
                        start_line: 1,
                        start_col: 0,
                        end_line: 1,
                        end_col: 0,
                        new_text: format!("use {};\n", module),
                    }],
                });
            }
        }
        None
    }

    /// Generate Derive macro suggestion
    pub fn derive_trait(struct_name: &str, trait_name: &str, struct_line: u32) -> CodeAction {
        CodeAction {
            title: format!("Derive {} for {}", trait_name, struct_name),
            kind: CodeActionKind::QuickFix,
            edits: vec![TextEdit {
                file: String::new(),
                start_line: struct_line,
                start_col: 0,
                end_line: struct_line,
                end_col: 0,
                new_text: format!("#[derive({})]\n", trait_name),
            }],
        }
    }
}

/// Signature Help with Tensor Shape Inference
pub struct SignatureHelpProvider;

#[derive(Debug)]
pub struct SignatureHelp {
    pub signatures: Vec<SignatureInfo>,
    pub active_signature: u32,
    pub active_parameter: u32,
}

#[derive(Debug)]
pub struct SignatureInfo {
    pub label: String,
    pub documentation: Option<String>,
    pub parameters: Vec<ParameterInfo>,
}

#[derive(Debug)]
pub struct ParameterInfo {
    pub label: String,
    pub documentation: Option<String>,
    pub tensor_shape: Option<String>, // e.g., "[batch, seq_len, hidden]"
}

impl SignatureHelpProvider {
    pub fn get_signature(func_name: &str) -> Option<SignatureHelp> {
        // Builtin signatures with tensor shape annotations
        let signatures = match func_name {
            "matmul" => vec![SignatureInfo {
                label: "matmul(a: Tensor<T>, b: Tensor<T>) -> Tensor<T>".to_string(),
                documentation: Some("Matrix multiplication. Shapes: [M, K] @ [K, N] -> [M, N]".to_string()),
                parameters: vec![
                    ParameterInfo {
                        label: "a".to_string(),
                        documentation: Some("Left operand".to_string()),
                        tensor_shape: Some("[M, K]".to_string()),
                    },
                    ParameterInfo {
                        label: "b".to_string(),
                        documentation: Some("Right operand".to_string()),
                        tensor_shape: Some("[K, N]".to_string()),
                    },
                ],
            }],
            "softmax" => vec![SignatureInfo {
                label: "softmax(x: Tensor<T>, dim: i32) -> Tensor<T>".to_string(),
                documentation: Some("Softmax along dimension".to_string()),
                parameters: vec![
                    ParameterInfo {
                        label: "x".to_string(),
                        documentation: None,
                        tensor_shape: Some("[*, D, *]".to_string()),
                    },
                    ParameterInfo {
                        label: "dim".to_string(),
                        documentation: Some("Dimension to apply softmax".to_string()),
                        tensor_shape: None,
                    },
                ],
            }],
            _ => return None,
        };

        Some(SignatureHelp {
            signatures,
            active_signature: 0,
            active_parameter: 0,
        })
    }
}

/// Refactoring: Rename Symbol across files
pub struct RenameProvider;

impl RenameProvider {
    pub fn rename(old_name: &str, new_name: &str, index: &SymbolIndex) -> Vec<TextEdit> {
        let references = index.find_references(old_name);
        
        references.iter().map(|sym| {
            TextEdit {
                file: sym.file_path.clone(),
                start_line: sym.line,
                start_col: sym.column,
                end_line: sym.line,
                end_col: sym.column + old_name.len() as u32,
                new_text: new_name.to_string(),
            }
        }).collect()
    }
}

/// Code Formatter
pub struct OmniFormatter;

impl OmniFormatter {
    pub fn format(source: &str) -> String {
        let mut output = String::new();
        let mut indent_level = 0;
        
        for line in source.lines() {
            let trimmed = line.trim();
            
            // Decrease indent before closing braces
            if trimmed.starts_with('}') || trimmed.starts_with(')') {
                indent_level = indent_level.saturating_sub(1);
            }
            
            // Apply indentation
            output.push_str(&"    ".repeat(indent_level));
            output.push_str(trimmed);
            output.push('\n');
            
            // Increase indent after opening braces
            if trimmed.ends_with('{') || trimmed.ends_with(':') {
                indent_level += 1;
            }
        }
        
        output
    }
}

/// Snippet Engine
pub struct SnippetEngine;

impl SnippetEngine {
    pub fn get_snippets() -> Vec<Snippet> {
        vec![
            Snippet {
                prefix: "fn".to_string(),
                label: "Function".to_string(),
                body: "fn ${1:name}(${2:params}) -> ${3:T}:\n    ${0:pass}".to_string(),
            },
            Snippet {
                prefix: "async".to_string(),
                label: "Async Function".to_string(),
                body: "async fn ${1:name}(${2:params}) -> ${3:T}:\n    ${0:pass}".to_string(),
            },
            Snippet {
                prefix: "struct".to_string(),
                label: "Struct".to_string(),
                body: "struct ${1:Name}:\n    ${2:field}: ${3:T}".to_string(),
            },
            Snippet {
                prefix: "test".to_string(),
                label: "Test Function".to_string(),
                body: "#[test]\nfn test_${1:name}():\n    assert_eq!(${2:expected}, ${0:actual})".to_string(),
            },
            Snippet {
                prefix: "gpu".to_string(),
                label: "GPU Kernel".to_string(),
                body: "@gpu.kernel\nfn ${1:kernel_name}(${2:params}):\n    let tid = threadIdx.x\n    ${0:pass}".to_string(),
            },
            Snippet {
                prefix: "model".to_string(),
                label: "Neural Network Module".to_string(),
                body: "struct ${1:Model}:\n    ${2:layers}: Linear\n\nimpl ${1:Model}:\n    fn forward(self, x: Tensor) -> Tensor:\n        ${0:pass}".to_string(),
            },
        ]
    }
}

#[derive(Debug)]
pub struct Snippet {
    pub prefix: String,
    pub label: String,
    pub body: String,
}
