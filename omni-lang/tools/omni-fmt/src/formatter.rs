//! Omni Code Formatter Module
//! 
//! Handles parsing and reformatting of Omni source code.

use anyhow::Result;
use log::trace;

/// Formatter configuration options
#[derive(Clone, Debug)]
pub struct FormatConfig {
    /// Number of spaces per indentation level
    pub indent_spaces: usize,
    /// Maximum line width before wrapping
    pub max_line_width: usize,
    /// Align colons in struct/enum fields
    pub align_colons: bool,
    /// Sort import statements alphabetically
    pub sort_imports: bool,
    /// Blank lines after import block
    pub blank_lines_after_imports: usize,
    /// Blank lines between top-level functions
    pub blank_lines_between_functions: usize,
    /// Ensure file ends with newline
    pub trailing_newline: bool,
}

impl Default for FormatConfig {
    fn default() -> Self {
        Self {
            indent_spaces: 4,
            max_line_width: 100,
            align_colons: true,
            sort_imports: true,
            blank_lines_after_imports: 1,
            blank_lines_between_functions: 2,
            trailing_newline: true,
        }
    }
}

/// Omni language formatter
pub struct OmniFormatter {
    config: FormatConfig,
}

impl OmniFormatter {
    pub fn new(config: FormatConfig) -> Self {
        Self { config }
    }
    
    /// Format Omni source code
    pub fn format(&self, source: &str) -> Result<String> {
        let mut result = String::with_capacity(source.len());
        let lines: Vec<&str> = source.lines().collect();
        
        let mut in_string = false;
        let mut in_multiline_string = false;
        let mut prev_line_type = LineType::Empty;
        let mut import_block: Vec<String> = Vec::new();
        let mut collecting_imports = false;
        
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let line_type = self.classify_line(trimmed);
            
            // Handle imports - collect and sort if configured
            if self.config.sort_imports {
                if line_type == LineType::Import {
                    import_block.push(trimmed.to_string());
                    collecting_imports = true;
                    continue;
                } else if collecting_imports && !import_block.is_empty() {
                    // Flush sorted imports
                    import_block.sort();
                    for import in &import_block {
                        result.push_str(import);
                        result.push('\n');
                    }
                    for _ in 0..self.config.blank_lines_after_imports {
                        result.push('\n');
                    }
                    import_block.clear();
                    collecting_imports = false;
                }
            }
            
            // Handle blank lines between sections
            let should_add_blank = self.should_add_blank_line(prev_line_type, line_type, i);
            if should_add_blank && !result.is_empty() && !result.ends_with("\n\n") {
                result.push('\n');
            }
            
            // Skip duplicate blank lines
            if line_type == LineType::Empty {
                if prev_line_type == LineType::Empty && result.ends_with("\n\n") {
                    continue;
                }
            }
            
            // Format the line
            let formatted = self.format_line(line, trimmed, line_type)?;
            result.push_str(&formatted);
            result.push('\n');
            
            prev_line_type = line_type;
        }
        
        // Flush any remaining imports
        if !import_block.is_empty() {
            import_block.sort();
            for import in &import_block {
                result.push_str(import);
                result.push('\n');
            }
        }
        
        // Ensure trailing newline
        if self.config.trailing_newline {
            while result.ends_with("\n\n") {
                result.pop();
            }
            if !result.ends_with('\n') {
                result.push('\n');
            }
        }
        
        Ok(result)
    }
    
    /// Classify type of line for formatting decisions
    fn classify_line(&self, trimmed: &str) -> LineType {
        if trimmed.is_empty() {
            LineType::Empty
        } else if trimmed.starts_with('#') {
            LineType::Comment
        } else if trimmed.starts_with("import ") {
            LineType::Import
        } else if trimmed.starts_with("module ") {
            LineType::Module
        } else if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
            LineType::Function
        } else if trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ") {
            LineType::Struct
        } else if trimmed.starts_with("enum ") || trimmed.starts_with("pub enum ") {
            LineType::Enum
        } else if trimmed.starts_with("trait ") || trimmed.starts_with("pub trait ") {
            LineType::Trait
        } else if trimmed.starts_with("impl ") {
            LineType::Impl
        } else if trimmed.starts_with("const ") || trimmed.starts_with("pub const ") {
            LineType::Const
        } else if trimmed.starts_with("type ") || trimmed.starts_with("pub type ") {
            LineType::TypeAlias
        } else if trimmed.starts_with("@") {
            LineType::Attribute
        } else {
            LineType::Code
        }
    }
    
    /// Determine if blank line should be added between line types
    fn should_add_blank_line(&self, prev: LineType, current: LineType, line_num: usize) -> bool {
        if line_num == 0 {
            return false;
        }
        
        match (prev, current) {
            // Add blank lines before top-level definitions
            (LineType::Code | LineType::Comment, LineType::Function) => true,
            (LineType::Code | LineType::Comment, LineType::Struct) => true,
            (LineType::Code | LineType::Comment, LineType::Enum) => true,
            (LineType::Code | LineType::Comment, LineType::Trait) => true,
            (LineType::Code | LineType::Comment, LineType::Impl) => true,
            
            // After imports
            (LineType::Import, LineType::Code) => true,
            (LineType::Import, LineType::Function) => true,
            (LineType::Import, LineType::Struct) => true,
            
            // After module declaration
            (LineType::Module, _) if current != LineType::Empty => true,
            
            // Between functions
            (LineType::Code, LineType::Function) => true,
            
            _ => false,
        }
    }
    
    /// Format a single line
    fn format_line(&self, original: &str, trimmed: &str, line_type: LineType) -> Result<String> {
        if trimmed.is_empty() {
            return Ok(String::new());
        }
        
        // Calculate indentation level
        let indent_level = self.calculate_indent(original);
        let indent = " ".repeat(indent_level * self.config.indent_spaces);
        
        // Apply formatting rules based on line type
        let formatted_content = match line_type {
            LineType::Comment => self.format_comment(trimmed),
            LineType::Import => self.format_import(trimmed),
            LineType::Function => self.format_function_sig(trimmed),
            LineType::Struct | LineType::Enum => self.format_type_def(trimmed),
            _ => self.format_code(trimmed),
        };
        
        Ok(format!("{}{}", indent, formatted_content))
    }
    
    /// Calculate indentation level from original line
    fn calculate_indent(&self, line: &str) -> usize {
        let spaces = line.len() - line.trim_start().len();
        spaces / self.config.indent_spaces
    }
    
    /// Format comment lines
    fn format_comment(&self, line: &str) -> String {
        if line.starts_with("# ") || line == "#" {
            line.to_string()
        } else if line.starts_with('#') {
            format!("# {}", &line[1..])
        } else {
            line.to_string()
        }
    }
    
    /// Format import statements
    fn format_import(&self, line: &str) -> String {
        // Normalize spacing around ::
        let normalized = line
            .replace(" :: ", "::")
            .replace(":: ", "::")
            .replace(" ::", "::");
        
        // Normalize spacing around braces in grouped imports
        let normalized = normalized
            .replace("::{ ", "::{")
            .replace(" }", "}")
            .replace(" ,", ",")
            .replace(",  ", ", ")
            .replace(", ", ", ");
        
        normalized
    }
    
    /// Format function signature
    fn format_function_sig(&self, line: &str) -> String {
        // Normalize spacing around arrows and colons
        let formatted = line
            .replace(" ->", " ->")
            .replace("-> ", "-> ")
            .replace(" :", ": ")
            .replace(":  ", ": ")
            .replace(" ,", ",")
            .replace(",  ", ", ");
        
        // Ensure space after fn keyword
        if formatted.starts_with("fn") && !formatted.starts_with("fn ") {
            format!("fn {}", &formatted[2..])
        } else {
            formatted
        }
    }
    
    /// Format struct/enum definitions
    fn format_type_def(&self, line: &str) -> String {
        let formatted = line
            .replace(" :", ":")
            .replace(":  ", ": ");
        formatted
    }
    
    /// Format general code
    fn format_code(&self, line: &str) -> String {
        let mut result = String::with_capacity(line.len());
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;
        let mut in_string = false;
        let mut string_char = '"';
        
        while i < chars.len() {
            let c = chars[i];
            
            // Track string state
            if (c == '"' || c == '\'') && (i == 0 || chars[i-1] != '\\') {
                if !in_string {
                    in_string = true;
                    string_char = c;
                } else if c == string_char {
                    in_string = false;
                }
                result.push(c);
                i += 1;
                continue;
            }
            
            // Don't format inside strings
            if in_string {
                result.push(c);
                i += 1;
                continue;
            }
            
            // Normalize operators and spacing
            match c {
                '=' => {
                    // Handle ==, =>, etc
                    if i + 1 < chars.len() && (chars[i+1] == '=' || chars[i+1] == '>') {
                        // Ensure space before
                        if !result.ends_with(' ') && !result.is_empty() {
                            result.push(' ');
                        }
                        result.push(c);
                        result.push(chars[i+1]);
                        // Ensure space after
                        if i + 2 < chars.len() && chars[i+2] != ' ' {
                            result.push(' ');
                        }
                        i += 2;
                    } else {
                        // Single = assignment
                        if !result.ends_with(' ') && !result.is_empty() {
                            result.push(' ');
                        }
                        result.push(c);
                        if i + 1 < chars.len() && chars[i+1] != ' ' {
                            result.push(' ');
                        }
                        i += 1;
                    }
                }
                ',' => {
                    result.push(c);
                    if i + 1 < chars.len() && chars[i+1] != ' ' && chars[i+1] != '\n' {
                        result.push(' ');
                    }
                    i += 1;
                }
                ':' => {
                    // Handle :: path separator
                    if i + 1 < chars.len() && chars[i+1] == ':' {
                        result.push(':');
                        result.push(':');
                        i += 2;
                    } else {
                        // Type annotation colon
                        result.push(c);
                        if i + 1 < chars.len() && chars[i+1] != ' ' && chars[i+1] != ':' {
                            result.push(' ');
                        }
                        i += 1;
                    }
                }
                _ => {
                    result.push(c);
                    i += 1;
                }
            }
        }
        
        // Remove trailing whitespace
        result.trim_end().to_string()
    }
}

/// Types of lines for formatting decisions
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineType {
    Empty,
    Comment,
    Module,
    Import,
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Const,
    TypeAlias,
    Attribute,
    Code,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_import() {
        let formatter = OmniFormatter::new(FormatConfig::default());
        
        let input = "import std :: collections :: { HashMap,Vec }";
        let expected = "import std::collections::{HashMap, Vec}";
        assert_eq!(formatter.format_import(input), expected);
    }
    
    #[test]
    fn test_format_comment() {
        let formatter = OmniFormatter::new(FormatConfig::default());
        
        assert_eq!(formatter.format_comment("#comment"), "# comment");
        assert_eq!(formatter.format_comment("# comment"), "# comment");
    }
    
    #[test]
    fn test_sort_imports() {
        let formatter = OmniFormatter::new(FormatConfig::default());
        
        let input = "import std::net\nimport std::async\nimport std::collections\n\nfn main():\n    pass\n";
        let result = formatter.format(input).unwrap();
        
        assert!(result.contains("import std::async\nimport std::collections\nimport std::net"));
    }
}
