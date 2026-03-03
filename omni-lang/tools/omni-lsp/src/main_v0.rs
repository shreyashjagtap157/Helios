//! Omni Language Server (LSP)
//! 
//! Implements the Language Server Protocol (v3.17) for the Omni language.
//! Communication: Standard Input/Output (JSON-RPC 2.0).

use std::error::Error;
use std::io::{self, BufRead, Read, Write};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
struct RpcMessage {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    result: Option<Value>,
    error: Option<RpcError>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

/// Symbol information stored in the index
#[derive(Debug, Clone)]
struct SymbolInfo {
    name: String,
    kind: SymbolKind,
    location: Location,
    documentation: Option<String>,
    signature: Option<String>,
    children: Vec<SymbolInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SymbolKind {
    Module = 2,
    Struct = 23,
    Enum = 10,
    Function = 12,
    Method = 6,
    Field = 8,
    Variable = 13,
    Constant = 14,
    Trait = 11,
    Type = 15,
}

#[derive(Debug, Clone)]
struct Location {
    uri: String,
    start_line: u32,
    start_col: u32,
    end_line: u32,
    end_col: u32,
}

/// Document state
struct Document {
    uri: String,
    text: String,
    version: i32,
    symbols: Vec<SymbolInfo>,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone)]
struct Diagnostic {
    range: Range,
    severity: u32,
    code: Option<String>,
    message: String,
}

#[derive(Debug, Clone, Copy)]
struct Range {
    start_line: u32,
    start_col: u32,
    end_line: u32,
    end_col: u32,
}

/// Language server state
struct LanguageServer {
    documents: HashMap<String, Document>,
    symbol_index: HashMap<String, SymbolInfo>,
    root_path: Option<PathBuf>,
    initialized: bool,
}

impl LanguageServer {
    fn new() -> Self {
        LanguageServer {
            documents: HashMap::new(),
            symbol_index: HashMap::new(),
            root_path: None,
            initialized: false,
        }
    }
    
    /// Index symbols from document text
    fn index_document(&mut self, uri: &str, text: &str) -> Vec<Diagnostic> {
        let mut symbols = Vec::new();
        let mut diagnostics = Vec::new();
        
        // Simple parsing for symbol extraction
        let lines: Vec<&str> = text.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            
            // Function definitions
            if trimmed.starts_with("fn ") || trimmed.starts_with("async fn ") {
                if let Some(name) = Self::extract_fn_name(trimmed) {
                    symbols.push(SymbolInfo {
                        name: name.clone(),
                        kind: SymbolKind::Function,
                        location: Location {
                            uri: uri.to_string(),
                            start_line: line_num as u32,
                            start_col: 0,
                            end_line: line_num as u32,
                            end_col: line.len() as u32,
                        },
                        documentation: Self::extract_doc_comment(&lines, line_num),
                        signature: Some(trimmed.to_string()),
                        children: Vec::new(),
                    });
                    self.symbol_index.insert(name, symbols.last().unwrap().clone());
                }
            }
            
            // Struct definitions
            if trimmed.starts_with("struct ") {
                if let Some(name) = Self::extract_name_after("struct ", trimmed) {
                    let fields = Self::extract_struct_fields(&lines, line_num);
                    symbols.push(SymbolInfo {
                        name: name.clone(),
                        kind: SymbolKind::Struct,
                        location: Location {
                            uri: uri.to_string(),
                            start_line: line_num as u32,
                            start_col: 0,
                            end_line: line_num as u32,
                            end_col: line.len() as u32,
                        },
                        documentation: Self::extract_doc_comment(&lines, line_num),
                        signature: None,
                        children: fields,
                    });
                    self.symbol_index.insert(name, symbols.last().unwrap().clone());
                }
            }
            
            // Enum definitions
            if trimmed.starts_with("enum ") {
                if let Some(name) = Self::extract_name_after("enum ", trimmed) {
                    symbols.push(SymbolInfo {
                        name: name.clone(),
                        kind: SymbolKind::Enum,
                        location: Location {
                            uri: uri.to_string(),
                            start_line: line_num as u32,
                            start_col: 0,
                            end_line: line_num as u32,
                            end_col: line.len() as u32,
                        },
                        documentation: Self::extract_doc_comment(&lines, line_num),
                        signature: None,
                        children: Vec::new(),
                    });
                    self.symbol_index.insert(name, symbols.last().unwrap().clone());
                }
            }
            
            // Trait definitions
            if trimmed.starts_with("trait ") {
                if let Some(name) = Self::extract_name_after("trait ", trimmed) {
                    symbols.push(SymbolInfo {
                        name: name.clone(),
                        kind: SymbolKind::Trait,
                        location: Location {
                            uri: uri.to_string(),
                            start_line: line_num as u32,
                            start_col: 0,
                            end_line: line_num as u32,
                            end_col: line.len() as u32,
                        },
                        documentation: Self::extract_doc_comment(&lines, line_num),
                        signature: None,
                        children: Vec::new(),
                    });
                    self.symbol_index.insert(name, symbols.last().unwrap().clone());
                }
            }
            
            // Const definitions  
            if trimmed.starts_with("const ") {
                if let Some(name) = Self::extract_const_name(trimmed) {
                    symbols.push(SymbolInfo {
                        name: name.clone(),
                        kind: SymbolKind::Constant,
                        location: Location {
                            uri: uri.to_string(),
                            start_line: line_num as u32,
                            start_col: 0,
                            end_line: line_num as u32,
                            end_col: line.len() as u32,
                        },
                        documentation: None,
                        signature: Some(trimmed.to_string()),
                        children: Vec::new(),
                    });
                    self.symbol_index.insert(name, symbols.last().unwrap().clone());
                }
            }
            
            // Check for syntax errors (simple heuristics)
            if trimmed.contains("::") && !trimmed.ends_with(":") && trimmed.ends_with("::") {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start_line: line_num as u32,
                        start_col: 0,
                        end_line: line_num as u32,
                        end_col: line.len() as u32,
                    },
                    severity: 1,
                    code: Some("E0001".to_string()),
                    message: "Incomplete path expression".to_string(),
                });
            }
        }
        
        if let Some(doc) = self.documents.get_mut(uri) {
            doc.symbols = symbols;
            doc.diagnostics = diagnostics.clone();
        }
        
        diagnostics
    }
    
    fn extract_fn_name(line: &str) -> Option<String> {
        let after_fn = if line.starts_with("async fn ") {
            &line[9..]
        } else {
            &line[3..]
        };
        
        let end = after_fn.find(&['(', '<', ':'][..])?;
        Some(after_fn[..end].trim().to_string())
    }
    
    fn extract_name_after(prefix: &str, line: &str) -> Option<String> {
        let after = &line[prefix.len()..];
        let end = after.find(&['<', ':', '(', '{', ' '][..]).unwrap_or(after.len());
        let name = after[..end].trim();
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    }
    
    fn extract_const_name(line: &str) -> Option<String> {
        let after = &line[6..]; // Skip "const "
        let end = after.find(&[':', '='][..])?;
        Some(after[..end].trim().to_string())
    }
    
    fn extract_doc_comment(lines: &[&str], current_line: usize) -> Option<String> {
        if current_line == 0 {
            return None;
        }
        
        let mut doc_lines = Vec::new();
        let mut i = current_line - 1;
        
        loop {
            let line = lines[i].trim();
            if line.starts_with("#") || line.starts_with("///") {
                let comment = if line.starts_with("///") {
                    line[3..].trim()
                } else if line.starts_with("# ") {
                    line[2..].trim()
                } else {
                    break;
                };
                doc_lines.insert(0, comment.to_string());
            } else if !line.is_empty() {
                break;
            }
            
            if i == 0 {
                break;
            }
            i -= 1;
        }
        
        if doc_lines.is_empty() {
            None
        } else {
            Some(doc_lines.join("\n"))
        }
    }
    
    fn extract_struct_fields(lines: &[&str], struct_line: usize) -> Vec<SymbolInfo> {
        let mut fields = Vec::new();
        let mut i = struct_line + 1;
        
        while i < lines.len() {
            let line = lines[i].trim();
            if line.is_empty() || line.starts_with("fn ") || line.starts_with("struct ") ||
               line.starts_with("enum ") || line.starts_with("trait ") ||
               line.starts_with("impl ") || line.starts_with("#") ||
               (!line.starts_with(" ") && !line.starts_with("\t") && !line.contains(":")) {
                break;
            }
            
            // Parse field: name: Type
            if let Some(colon_pos) = line.find(':') {
                let name = line[..colon_pos].trim().to_string();
                if !name.is_empty() && !name.starts_with("fn ") {
                    fields.push(SymbolInfo {
                        name,
                        kind: SymbolKind::Field,
                        location: Location {
                            uri: String::new(),
                            start_line: i as u32,
                            start_col: 0,
                            end_line: i as u32,
                            end_col: line.len() as u32,
                        },
                        documentation: None,
                        signature: Some(line.to_string()),
                        children: Vec::new(),
                    });
                }
            }
            
            i += 1;
        }
        
        fields
    }
    
    /// Find definition of symbol at position
    fn find_definition(&self, uri: &str, line: u32, col: u32) -> Option<Location> {
        let doc = self.documents.get(uri)?;
        let text_line = doc.text.lines().nth(line as usize)?;
        
        // Extract word at position
        let word = Self::word_at_position(text_line, col as usize)?;
        
        // Look up in symbol index
        if let Some(symbol) = self.symbol_index.get(&word) {
            return Some(symbol.location.clone());
        }
        
        None
    }
    
    /// Find all references to symbol
    fn find_references(&self, uri: &str, line: u32, col: u32) -> Vec<Location> {
        let mut refs = Vec::new();
        
        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return refs,
        };
        
        let text_line = match doc.text.lines().nth(line as usize) {
            Some(l) => l,
            None => return refs,
        };
        
        let word = match Self::word_at_position(text_line, col as usize) {
            Some(w) => w,
            None => return refs,
        };
        
        // Search all documents for references
        for (doc_uri, document) in &self.documents {
            for (line_num, line_text) in document.text.lines().enumerate() {
                if line_text.contains(&word) {
                    // Simple word boundary check
                    let mut start = 0;
                    while let Some(pos) = line_text[start..].find(&word) {
                        let actual_pos = start + pos;
                        let before_ok = actual_pos == 0 || 
                            !line_text.chars().nth(actual_pos - 1).unwrap().is_alphanumeric();
                        let after_ok = actual_pos + word.len() >= line_text.len() ||
                            !line_text.chars().nth(actual_pos + word.len()).unwrap().is_alphanumeric();
                        
                        if before_ok && after_ok {
                            refs.push(Location {
                                uri: doc_uri.clone(),
                                start_line: line_num as u32,
                                start_col: actual_pos as u32,
                                end_line: line_num as u32,
                                end_col: (actual_pos + word.len()) as u32,
                            });
                        }
                        
                        start = actual_pos + 1;
                        if start >= line_text.len() {
                            break;
                        }
                    }
                }
            }
        }
        
        refs
    }
    
    /// Get completions at position
    fn get_completions(&self, uri: &str, line: u32, col: u32) -> Vec<CompletionItem> {
        let mut items = Vec::new();
        
        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return items,
        };
        
        let text_line = match doc.text.lines().nth(line as usize) {
            Some(l) => l,
            None => return items,
        };
        
        // Get prefix before cursor
        let prefix = &text_line[..col as usize];
        let trigger = if prefix.ends_with('.') {
            CompletionTrigger::Dot
        } else if prefix.ends_with("::") {
            CompletionTrigger::PathSep
        } else {
            CompletionTrigger::Identifier
        };
        
        match trigger {
            CompletionTrigger::Dot => {
                // Method completions - find the type of expression before dot
                // For now, return common methods
                items.push(CompletionItem {
                    label: "len()".to_string(),
                    kind: CompletionItemKind::Method,
                    detail: Some("fn len(&self) -> usize".to_string()),
                    documentation: Some("Returns the length".to_string()),
                });
                items.push(CompletionItem {
                    label: "is_empty()".to_string(),
                    kind: CompletionItemKind::Method,
                    detail: Some("fn is_empty(&self) -> bool".to_string()),
                    documentation: Some("Returns true if empty".to_string()),
                });
            }
            CompletionTrigger::PathSep => {
                // Module/type path completions
                if prefix.ends_with("Option::") {
                    items.push(CompletionItem {
                        label: "Some".to_string(),
                        kind: CompletionItemKind::EnumMember,
                        detail: Some("Option::Some(T)".to_string()),
                        documentation: Some("Some value".to_string()),
                    });
                    items.push(CompletionItem {
                        label: "None".to_string(),
                        kind: CompletionItemKind::EnumMember,
                        detail: Some("Option::None".to_string()),
                        documentation: Some("No value".to_string()),
                    });
                }
            }
            CompletionTrigger::Identifier => {
                // Keywords and symbols
                let partial = Self::word_at_position(text_line, col.saturating_sub(1) as usize)
                    .unwrap_or_default();
                    
                // Keywords
                for kw in &["fn", "let", "if", "else", "for", "while", "match", "struct", 
                           "enum", "trait", "impl", "return", "async", "await", "use", 
                           "import", "pub", "mut", "const", "static"] {
                    if kw.starts_with(&partial) {
                        items.push(CompletionItem {
                            label: kw.to_string(),
                            kind: CompletionItemKind::Keyword,
                            detail: None,
                            documentation: None,
                        });
                    }
                }
                
                // Symbols from index
                for (name, symbol) in &self.symbol_index {
                    if name.starts_with(&partial) {
                        let kind = match symbol.kind {
                            SymbolKind::Function => CompletionItemKind::Function,
                            SymbolKind::Struct => CompletionItemKind::Struct,
                            SymbolKind::Enum => CompletionItemKind::Enum,
                            SymbolKind::Trait => CompletionItemKind::Interface,
                            SymbolKind::Constant => CompletionItemKind::Constant,
                            _ => CompletionItemKind::Variable,
                        };
                        items.push(CompletionItem {
                            label: name.clone(),
                            kind,
                            detail: symbol.signature.clone(),
                            documentation: symbol.documentation.clone(),
                        });
                    }
                }
            }
        }
        
        items
    }
    
    /// Get hover information at position
    fn get_hover(&self, uri: &str, line: u32, col: u32) -> Option<HoverInfo> {
        let doc = self.documents.get(uri)?;
        let text_line = doc.text.lines().nth(line as usize)?;
        let word = Self::word_at_position(text_line, col as usize)?;
        
        // Check symbol index
        if let Some(symbol) = self.symbol_index.get(&word) {
            let mut contents = String::new();
            
            if let Some(sig) = &symbol.signature {
                contents.push_str("```omni\n");
                contents.push_str(sig);
                contents.push_str("\n```\n");
            }
            
            if let Some(doc) = &symbol.documentation {
                contents.push_str("\n---\n");
                contents.push_str(doc);
            }
            
            return Some(HoverInfo {
                contents,
                range: Some(Range {
                    start_line: line,
                    start_col: col,
                    end_line: line,
                    end_col: col + word.len() as u32,
                }),
            });
        }
        
        // Built-in types
        let builtin_docs = match word.as_str() {
            "i8" | "i16" | "i32" | "i64" => Some("Signed integer type"),
            "u8" | "u16" | "u32" | "u64" => Some("Unsigned integer type"),
            "f32" | "f64" => Some("Floating-point type"),
            "bool" => Some("Boolean type (true or false)"),
            "str" => Some("String slice type"),
            "String" => Some("Owned, growable string type"),
            "Vec" => Some("Growable array type"),
            "Option" => Some("Optional value type: Some(T) or None"),
            "Result" => Some("Error handling type: Ok(T) or Err(E)"),
            _ => None,
        };
        
        if let Some(doc) = builtin_docs {
            return Some(HoverInfo {
                contents: format!("```omni\n{}\n```\n\n{}", word, doc),
                range: None,
            });
        }
        
        None
    }
    
    fn word_at_position(line: &str, pos: usize) -> Option<String> {
        if pos >= line.len() {
            return None;
        }
        
        let chars: Vec<char> = line.chars().collect();
        let mut start = pos;
        let mut end = pos;
        
        // Find word start
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }
        
        // Find word end
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }
        
        if start == end {
            None
        } else {
            Some(chars[start..end].iter().collect())
        }
    }
    
    /// Format an Omni document
    fn format_document(&self, source: &str, tab_size: usize, insert_spaces: bool) -> String {
        let indent_char = if insert_spaces {
            " ".repeat(tab_size)
        } else {
            "\t".to_string()
        };
        
        let mut output = String::new();
        let mut indent_level: usize = 0;
        
        for line in source.lines() {
            let trimmed = line.trim();
            
            // Skip empty lines
            if trimmed.is_empty() {
                output.push('\n');
                continue;
            }
            
            // Decrease indent for closing constructs
            if trimmed.starts_with("else:") || trimmed.starts_with("elif ")
                || trimmed.starts_with("except:") || trimmed.starts_with("finally:") {
                indent_level = indent_level.saturating_sub(1);
            }
            
            // Write indented line
            for _ in 0..indent_level {
                output.push_str(&indent_char);
            }
            output.push_str(trimmed);
            output.push('\n');
            
            // Increase indent after block openers
            if trimmed.ends_with(':') && !trimmed.starts_with('#') && !trimmed.starts_with("//") {
                // Check if it's a real block opener (fn, struct, if, for, etc.)
                let is_block = trimmed.starts_with("fn ") || trimmed.starts_with("async fn ")
                    || trimmed.starts_with("struct ") || trimmed.starts_with("enum ")
                    || trimmed.starts_with("trait ") || trimmed.starts_with("impl ")
                    || trimmed.starts_with("if ") || trimmed.starts_with("else:")
                    || trimmed.starts_with("elif ") || trimmed.starts_with("for ")
                    || trimmed.starts_with("while ") || trimmed.starts_with("match ")
                    || trimmed.starts_with("loop:") || trimmed.starts_with("try:")
                    || trimmed.starts_with("except:") || trimmed.starts_with("finally:")
                    || trimmed.starts_with("module ");
                
                if is_block {
                    indent_level += 1;
                }
            }
        }
        
        output
    }
    
    /// Rename a symbol across all open documents
    fn rename_symbol(&self, uri: &str, line: u32, col: u32, new_name: &str) -> Value {
        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return Value::Null,
        };
        
        let text_line = match doc.text.lines().nth(line as usize) {
            Some(l) => l,
            None => return Value::Null,
        };
        
        let old_name = match Self::word_at_position(text_line, col as usize) {
            Some(w) => w,
            None => return Value::Null,
        };
        
        // Find all references and build workspace edit
        let refs = self.find_references(uri, line, col);
        
        let mut changes: HashMap<String, Vec<Value>> = HashMap::new();
        
        for location in &refs {
            let edits = changes.entry(location.uri.clone()).or_default();
            edits.push(serde_json::json!({
                "range": {
                    "start": { "line": location.start_line, "character": location.start_col },
                    "end": { "line": location.end_line, "character": location.end_col }
                },
                "newText": new_name
            }));
        }
        
        serde_json::json!({
            "changes": changes
        })
    }
    
    /// Get signature help at position (inside function call parentheses)
    fn get_signature_help(&self, uri: &str, line: u32, col: u32) -> Value {
        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return Value::Null,
        };
        
        let text_line = match doc.text.lines().nth(line as usize) {
            Some(l) => l,
            None => return Value::Null,
        };
        
        // Walk backwards from cursor to find the function name before '('
        let before_cursor = &text_line[..col as usize];
        
        // Count commas to determine active parameter
        let mut active_param = 0u32;
        let mut paren_depth = 0i32;
        let mut func_name_end = None;
        
        for (i, ch) in before_cursor.chars().rev().enumerate() {
            let pos = before_cursor.len() - 1 - i;
            match ch {
                ')' => paren_depth += 1,
                '(' => {
                    if paren_depth == 0 {
                        func_name_end = Some(pos);
                        break;
                    }
                    paren_depth -= 1;
                }
                ',' if paren_depth == 0 => active_param += 1,
                _ => {}
            }
        }
        
        let func_name = if let Some(end) = func_name_end {
            Self::word_at_position(before_cursor, end.saturating_sub(1))
        } else {
            None
        };
        
        let func_name = match func_name {
            Some(n) => n,
            None => return Value::Null,
        };
        
        // Look up function signature in symbol index
        if let Some(symbol) = self.symbol_index.get(&func_name) {
            if let Some(sig) = &symbol.signature {
                // Parse parameter names from signature
                let params = Self::extract_params_from_signature(sig);
                
                let param_infos: Vec<Value> = params.iter().map(|p| {
                    serde_json::json!({
                        "label": p
                    })
                }).collect();
                
                return serde_json::json!({
                    "signatures": [{
                        "label": sig,
                        "documentation": symbol.documentation.as_ref().map(|d| {
                            serde_json::json!({
                                "kind": "markdown",
                                "value": d
                            })
                        }),
                        "parameters": param_infos
                    }],
                    "activeSignature": 0,
                    "activeParameter": active_param
                });
            }
        }
        
        Value::Null
    }
    
    /// Extract parameter names from a function signature string
    fn extract_params_from_signature(sig: &str) -> Vec<String> {
        // Find content between ( and )
        if let Some(start) = sig.find('(') {
            if let Some(end) = sig.rfind(')') {
                let params_str = &sig[start + 1..end];
                return params_str.split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
                    .collect();
            }
        }
        Vec::new()
    }
    
    /// Get code actions for range
    fn get_code_actions(&self, uri: &str, start_line: u32, end_line: u32) -> Vec<Value> {
        let mut actions = Vec::new();
        
        let doc = match self.documents.get(uri) {
            Some(d) => d,
            None => return actions,
        };
        
        let lines: Vec<&str> = doc.text.lines().collect();
        
        for line_num in start_line..=end_line.min(lines.len() as u32 - 1) {
            let line = lines[line_num as usize].trim();
            
            // Suggest adding type annotation for untyped let bindings
            if line.starts_with("let ") && !line.contains(':') && line.contains('=') {
                actions.push(serde_json::json!({
                    "title": "Add type annotation",
                    "kind": "quickfix",
                    "diagnostics": [],
                    "isPreferred": false
                }));
            }
            
            // Suggest extracting function if selection spans multiple lines
            if end_line - start_line >= 2 {
                actions.push(serde_json::json!({
                    "title": "Extract function",
                    "kind": "refactor.extract",
                    "diagnostics": [],
                    "isPreferred": false
                }));
            }
            
            // Suggest organizing imports if on an import line
            if line.starts_with("import ") {
                actions.push(serde_json::json!({
                    "title": "Organize imports",
                    "kind": "source.organizeImports",
                    "diagnostics": [],
                    "isPreferred": true
                }));
                break; // Only suggest once
            }
        }
        
        // Check diagnostics for quick fixes
        for diag in &doc.diagnostics {
            if diag.range.start_line >= start_line && diag.range.end_line <= end_line {
                if diag.message.contains("Incomplete path") {
                    actions.push(serde_json::json!({
                        "title": format!("Fix: {}", diag.message),
                        "kind": "quickfix",
                        "diagnostics": [serde_json::json!({
                            "range": {
                                "start": { "line": diag.range.start_line, "character": diag.range.start_col },
                                "end": { "line": diag.range.end_line, "character": diag.range.end_col }
                            },
                            "message": diag.message
                        })],
                        "isPreferred": true
                    }));
                }
            }
        }
        
        actions
    }
}

enum CompletionTrigger {
    Dot,
    PathSep,
    Identifier,
}

#[derive(Debug)]
struct CompletionItem {
    label: String,
    kind: CompletionItemKind,
    detail: Option<String>,
    documentation: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum CompletionItemKind {
    Function = 3,
    Method = 2,
    Variable = 6,
    Struct = 22,
    Enum = 13,
    EnumMember = 20,
    Interface = 8,
    Constant = 21,
    Keyword = 14,
}

#[derive(Debug)]
struct HoverInfo {
    contents: String,
    range: Option<Range>,
}

fn main() -> Result<(), Box<dyn Error>> {
    eprintln!("Omni LSP starting...");

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = stdin.lock();
    
    let mut server = LanguageServer::new();

    // LSP Message Loop
    loop {
        // 1. Read Content-Length header
        let mut size = 0;
        let mut header_buffer = String::new();
        
        loop {
            header_buffer.clear();
            if reader.read_line(&mut header_buffer)? == 0 {
                return Ok(()); // EOF
            }
            
            if header_buffer == "\r\n" {
                break; // End of headers
            }
            
            if header_buffer.starts_with("Content-Length: ") {
                let len_str = header_buffer.trim_start_matches("Content-Length: ").trim();
                size = len_str.parse::<usize>()?;
            }
        }

        if size == 0 {
            continue;
        }

        // 2. Read Body
        let mut body_buffer = vec![0; size];
        reader.read_exact(&mut body_buffer)?;
        let body_str = String::from_utf8(body_buffer)?;
        
        // 3. Parse JSON-RPC
        match serde_json::from_str::<RpcMessage>(&body_str) {
            Ok(msg) => handle_message(&mut server, msg, &mut stdout)?,
            Err(e) => eprintln!("Failed to parse JSON: {}", e),
        }
    }
}

fn handle_message(server: &mut LanguageServer, msg: RpcMessage, stdout: &mut io::Stdout) -> Result<(), Box<dyn Error>> {
    eprintln!("Received method: {}", msg.method);

    match msg.method.as_str() {
        "initialize" => {
            let result = serde_json::json!({
                "capabilities": {
                    "textDocumentSync": {
                        "openClose": true,
                        "change": 1,
                        "save": { "includeText": true }
                    },
                    "hoverProvider": true,
                    "completionProvider": {
                        "resolveProvider": true,
                        "triggerCharacters": [".", ":", "<"]
                    },
                    "definitionProvider": true,
                    "referencesProvider": true,
                    "documentSymbolProvider": true,
                    "workspaceSymbolProvider": true,
                    "documentFormattingProvider": true,
                    "renameProvider": {
                        "prepareProvider": true
                    },
                    "signatureHelpProvider": {
                        "triggerCharacters": ["(", ","]
                    },
                    "codeActionProvider": true
                },
                "serverInfo": {
                    "name": "omni-lsp",
                    "version": "0.2.0"
                }
            });
            
            if let Some(params) = &msg.params {
                if let Some(root_uri) = params.get("rootUri").and_then(|v| v.as_str()) {
                    if let Ok(url) = Url::parse(root_uri) {
                        if let Ok(path) = url.to_file_path() {
                            server.root_path = Some(path);
                        }
                    }
                }
            }
            
            send_response(stdout, msg.id, Some(result), None)?;
        },
        
        "initialized" => {
            server.initialized = true;
            eprintln!("Client initialized. Root: {:?}", server.root_path);
        },
        
        "shutdown" => {
            send_response(stdout, msg.id, Some(Value::Null), None)?;
        },
        
        "exit" => {
            std::process::exit(0);
        },
        
        "textDocument/didOpen" => {
            if let Some(params) = msg.params {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                let text = params["textDocument"]["text"].as_str().unwrap_or("").to_string();
                let version = params["textDocument"]["version"].as_i64().unwrap_or(0) as i32;
                
                eprintln!("Opened file: {}", uri);
                
                server.documents.insert(uri.clone(), Document {
                    uri: uri.clone(),
                    text: text.clone(),
                    version,
                    symbols: Vec::new(),
                    diagnostics: Vec::new(),
                });
                
                let diagnostics = server.index_document(&uri, &text);
                publish_diagnostics(&uri, &diagnostics, stdout)?;
            }
        },
        
        "textDocument/didChange" => {
            if let Some(params) = msg.params {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("").to_string();
                let version = params["textDocument"]["version"].as_i64().unwrap_or(0) as i32;
                
                if let Some(changes) = params["contentChanges"].as_array() {
                    if let Some(change) = changes.first() {
                        let text = change["text"].as_str().unwrap_or("").to_string();
                        
                        if let Some(doc) = server.documents.get_mut(&uri) {
                            doc.text = text.clone();
                            doc.version = version;
                        }
                        
                        let diagnostics = server.index_document(&uri, &text);
                        publish_diagnostics(&uri, &diagnostics, stdout)?;
                    }
                }
            }
        },
        
        "textDocument/didClose" => {
            if let Some(params) = msg.params {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                server.documents.remove(uri);
            }
        },
        
        "textDocument/hover" => {
            if let Some(params) = msg.params {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                let line = params["position"]["line"].as_u64().unwrap_or(0) as u32;
                let col = params["position"]["character"].as_u64().unwrap_or(0) as u32;
                
                let result = if let Some(hover) = server.get_hover(uri, line, col) {
                    serde_json::json!({
                        "contents": {
                            "kind": "markdown",
                            "value": hover.contents
                        }
                    })
                } else {
                    Value::Null
                };
                
                send_response(stdout, msg.id, Some(result), None)?;
            }
        },
        
        "textDocument/completion" => {
            if let Some(params) = msg.params {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                let line = params["position"]["line"].as_u64().unwrap_or(0) as u32;
                let col = params["position"]["character"].as_u64().unwrap_or(0) as u32;
                
                let items = server.get_completions(uri, line, col);
                let result: Vec<Value> = items.iter().map(|item| {
                    serde_json::json!({
                        "label": item.label,
                        "kind": item.kind as u32,
                        "detail": item.detail,
                        "documentation": item.documentation.as_ref().map(|d| {
                            serde_json::json!({
                                "kind": "markdown",
                                "value": d
                            })
                        })
                    })
                }).collect();
                
                send_response(stdout, msg.id, Some(serde_json::json!(result)), None)?;
            }
        },
        
        "textDocument/definition" => {
            if let Some(params) = msg.params {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                let line = params["position"]["line"].as_u64().unwrap_or(0) as u32;
                let col = params["position"]["character"].as_u64().unwrap_or(0) as u32;
                
                let result = if let Some(loc) = server.find_definition(uri, line, col) {
                    serde_json::json!({
                        "uri": loc.uri,
                        "range": {
                            "start": { "line": loc.start_line, "character": loc.start_col },
                            "end": { "line": loc.end_line, "character": loc.end_col }
                        }
                    })
                } else {
                    Value::Null
                };
                
                send_response(stdout, msg.id, Some(result), None)?;
            }
        },
        
        "textDocument/references" => {
            if let Some(params) = msg.params {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                let line = params["position"]["line"].as_u64().unwrap_or(0) as u32;
                let col = params["position"]["character"].as_u64().unwrap_or(0) as u32;
                
                let refs = server.find_references(uri, line, col);
                let result: Vec<Value> = refs.iter().map(|loc| {
                    serde_json::json!({
                        "uri": loc.uri,
                        "range": {
                            "start": { "line": loc.start_line, "character": loc.start_col },
                            "end": { "line": loc.end_line, "character": loc.end_col }
                        }
                    })
                }).collect();
                
                send_response(stdout, msg.id, Some(serde_json::json!(result)), None)?;
            }
        },
        
        "textDocument/documentSymbol" => {
            if let Some(params) = msg.params {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                
                let result: Vec<Value> = if let Some(doc) = server.documents.get(uri) {
                    doc.symbols.iter().map(|sym| {
                        serde_json::json!({
                            "name": sym.name,
                            "kind": sym.kind as u32,
                            "range": {
                                "start": { "line": sym.location.start_line, "character": sym.location.start_col },
                                "end": { "line": sym.location.end_line, "character": sym.location.end_col }
                            },
                            "selectionRange": {
                                "start": { "line": sym.location.start_line, "character": sym.location.start_col },
                                "end": { "line": sym.location.end_line, "character": sym.location.end_col }
                            }
                        })
                    }).collect()
                } else {
                    Vec::new()
                };
                
                send_response(stdout, msg.id, Some(serde_json::json!(result)), None)?;
            }
        },
        
        "workspace/symbol" => {
            if let Some(params) = msg.params {
                let query = params["query"].as_str().unwrap_or("");
                
                let result: Vec<Value> = server.symbol_index.iter()
                    .filter(|(name, _)| query.is_empty() || name.contains(query))
                    .map(|(_, sym)| {
                        serde_json::json!({
                            "name": sym.name,
                            "kind": sym.kind as u32,
                            "location": {
                                "uri": sym.location.uri,
                                "range": {
                                    "start": { "line": sym.location.start_line, "character": sym.location.start_col },
                                    "end": { "line": sym.location.end_line, "character": sym.location.end_col }
                                }
                            }
                        })
                    })
                    .collect();
                
                send_response(stdout, msg.id, Some(serde_json::json!(result)), None)?;
            }
        },
        
        "textDocument/formatting" => {
            if let Some(params) = msg.params {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                let tab_size = params["options"]["tabSize"].as_u64().unwrap_or(4) as usize;
                let insert_spaces = params["options"]["insertSpaces"].as_bool().unwrap_or(true);
                
                let result = if let Some(doc) = server.documents.get(uri) {
                    let formatted = server.format_document(&doc.text, tab_size, insert_spaces);
                    let line_count = doc.text.lines().count() as u32;
                    serde_json::json!([{
                        "range": {
                            "start": { "line": 0, "character": 0 },
                            "end": { "line": line_count, "character": 0 }
                        },
                        "newText": formatted
                    }])
                } else {
                    serde_json::json!([])
                };
                
                send_response(stdout, msg.id, Some(result), None)?;
            }
        },
        
        "textDocument/rename" => {
            if let Some(params) = msg.params {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                let line = params["position"]["line"].as_u64().unwrap_or(0) as u32;
                let col = params["position"]["character"].as_u64().unwrap_or(0) as u32;
                let new_name = params["newName"].as_str().unwrap_or("");
                
                let result = server.rename_symbol(uri, line, col, new_name);
                send_response(stdout, msg.id, Some(result), None)?;
            }
        },
        
        "textDocument/prepareRename" => {
            if let Some(params) = msg.params {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                let line = params["position"]["line"].as_u64().unwrap_or(0) as u32;
                let col = params["position"]["character"].as_u64().unwrap_or(0) as u32;
                
                let result = if let Some(doc) = server.documents.get(uri) {
                    if let Some(text_line) = doc.text.lines().nth(line as usize) {
                        if let Some(word) = LanguageServer::word_at_position(text_line, col as usize) {
                            serde_json::json!({
                                "range": {
                                    "start": { "line": line, "character": col.saturating_sub(word.len() as u32 / 2) },
                                    "end": { "line": line, "character": col + word.len() as u32 / 2 }
                                },
                                "placeholder": word
                            })
                        } else {
                            Value::Null
                        }
                    } else {
                        Value::Null
                    }
                } else {
                    Value::Null
                };
                
                send_response(stdout, msg.id, Some(result), None)?;
            }
        },
        
        "textDocument/signatureHelp" => {
            if let Some(params) = msg.params {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                let line = params["position"]["line"].as_u64().unwrap_or(0) as u32;
                let col = params["position"]["character"].as_u64().unwrap_or(0) as u32;
                
                let result = server.get_signature_help(uri, line, col);
                send_response(stdout, msg.id, Some(result), None)?;
            }
        },
        
        "textDocument/codeAction" => {
            if let Some(params) = msg.params {
                let uri = params["textDocument"]["uri"].as_str().unwrap_or("");
                let start_line = params["range"]["start"]["line"].as_u64().unwrap_or(0) as u32;
                let end_line = params["range"]["end"]["line"].as_u64().unwrap_or(0) as u32;
                
                let result = server.get_code_actions(uri, start_line, end_line);
                send_response(stdout, msg.id, Some(serde_json::json!(result)), None)?;
            }
        },
        
        _ => {
            // Unhandled method
            if msg.id.is_some() {
                send_response(stdout, msg.id, None, Some(RpcError {
                    code: -32601,
                    message: format!("Method not found: {}", msg.method),
                    data: None,
                }))?;
            }
        }
    }
    
    Ok(())
}

fn publish_diagnostics(uri: &str, diagnostics: &[Diagnostic], stdout: &mut io::Stdout) -> Result<(), Box<dyn Error>> {
    let diag_array: Vec<Value> = diagnostics.iter().map(|d| {
        serde_json::json!({
            "range": {
                "start": { "line": d.range.start_line, "character": d.range.start_col },
                "end": { "line": d.range.end_line, "character": d.range.end_col }
            },
            "severity": d.severity,
            "code": d.code,
            "source": "omni",
            "message": d.message
        })
    }).collect();

    let params = serde_json::json!({
        "uri": uri,
        "diagnostics": diag_array
    });

    let notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": params
    });

    let body = serde_json::to_string(&notification)?;
    write!(stdout, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    stdout.flush()?;
    Ok(())
}

fn send_response(stdout: &mut io::Stdout, id: Option<Value>, result: Option<Value>, error: Option<RpcError>) -> Result<(), Box<dyn Error>> {
    if id.is_none() { return Ok(()); } // Notification, no response needed

    let response = RpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result,
        error,
    };

    let body = serde_json::to_string(&response)?;
    write!(stdout, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    stdout.flush()?;
    Ok(())
}
