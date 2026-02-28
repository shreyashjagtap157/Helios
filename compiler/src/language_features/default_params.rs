/// Default Parameters and Named Arguments Support
/// Enables ergonomic function signatures with defaults and keyword arguments
/// 
/// Syntax:
/// fn greet(name: String, greeting: String = "Hello") { ... }
/// greet(greeting: "Hi", name: "Alice");

use crate::ir::{Type, Function};
use std::collections::HashMap;

/// Represents a parameter with an optional default value
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
    pub default: Option<String>, // Expression as string, evaluated at call site
}

impl Parameter {
    pub fn new(name: String, ty: Type) -> Self {
        Parameter {
            name,
            ty,
            default: None,
        }
    }

    pub fn with_default(name: String, ty: Type, default: String) -> Self {
        Parameter {
            name,
            ty,
            default: Some(default),
        }
    }

    pub fn has_default(&self) -> bool {
        self.default.is_some()
    }
}

/// Represents an argument in a function call
#[derive(Debug, Clone)]
pub struct Argument {
    pub name: Option<String>, // None for positional, Some for named
    pub value: String,        // Expression as string
}

impl Argument {
    pub fn positional(value: String) -> Self {
        Argument { name: None, value }
    }

    pub fn named(name: String, value: String) -> Self {
        Argument {
            name: Some(name),
            value,
        }
    }

    pub fn is_named(&self) -> bool {
        self.name.is_some()
    }
}

/// Enhanced function signature with default parameters support
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
}

impl FunctionSignature {
    pub fn new(name: String, return_type: Type) -> Self {
        FunctionSignature {
            name,
            parameters: Vec::new(),
            return_type,
        }
    }

    /// Add a parameter
    pub fn add_param(mut self, param: Parameter) -> Self {
        self.parameters.push(param);
        self
    }

    /// Get the number of required (non-default) parameters
    pub fn required_params(&self) -> usize {
        self.parameters.iter().filter(|p| !p.has_default()).count()
    }

    /// Get the total number of parameters
    pub fn total_params(&self) -> usize {
        self.parameters.len()
    }

    /// Validate that all defaults come after required params
    pub fn validate(&self) -> Result<(), String> {
        let mut seen_default = false;
        for param in &self.parameters {
            if param.has_default() {
                seen_default = true;
            } else if seen_default {
                return Err(format!(
                    "Required parameter '{}' cannot follow default parameters",
                    param.name
                ));
            }
        }
        Ok(())
    }
}

/// Call resolver that matches arguments to parameters
pub struct CallResolver {
    signature: FunctionSignature,
}

impl CallResolver {
    pub fn new(signature: FunctionSignature) -> Self {
        CallResolver { signature }
    }

    /// Resolve a call with mixed positional and named arguments
    pub fn resolve_call(&self, args: Vec<Argument>) -> Result<Vec<(String, String)>, String> {
        let mut resolved: Vec<(String, String)> = Vec::new();
        let mut named_seen = false;
        let mut positional_idx = 0;

        for arg in args {
            if let Some(name) = arg.name {
                // Named argument
                named_seen = true;

                // Find matching parameter
                if let Some(param) = self.signature.parameters.iter().find(|p| p.name == name) {
                    resolved.push((param.name.clone(), arg.value));
                } else {
                    return Err(format!("Unknown parameter: {}", name));
                }
            } else {
                // Positional argument
                if named_seen {
                    return Err("Positional arguments cannot follow named arguments".to_string());
                }

                if positional_idx >= self.signature.parameters.len() {
                    return Err(format!(
                        "Too many positional arguments (expected max {})",
                        self.signature.parameters.len()
                    ));
                }

                let param = &self.signature.parameters[positional_idx];
                resolved.push((param.name.clone(), arg.value));
                positional_idx += 1;
            }
        }

        // Fill in defaults for missing parameters
        for param in &self.signature.parameters {
            if !resolved.iter().any(|(n, _)| n == &param.name) {
                if let Some(default) = &param.default {
                    resolved.push((param.name.clone(), default.clone()));
                } else {
                    return Err(format!("Missing required parameter: {}", param.name));
                }
            }
        }

        Ok(resolved)
    }

    /// Check if a call with given number of positional args is valid
    pub fn is_valid_call_count(&self, count: usize) -> bool {
        let required = self.signature.required_params();
        let total = self.signature.total_params();
        count >= required && count <= total
    }
}

/// Builder for function calls with named arguments
pub struct CallBuilder {
    function_name: String,
    args: Vec<Argument>,
}

impl CallBuilder {
    pub fn new(function_name: String) -> Self {
        CallBuilder {
            function_name,
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, value: String) -> Self {
        self.args.push(Argument::positional(value));
        self
    }

    pub fn named_arg(mut self, name: String, value: String) -> Self {
        self.args.push(Argument::named(name, value));
        self
    }

    pub fn build(self) -> (String, Vec<Argument>) {
        (self.function_name, self.args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_signature_defaults() {
        let sig = FunctionSignature::new("greet".to_string(), Type::String)
            .add_param(Parameter::new("name".to_string(), Type::String))
            .add_param(Parameter::with_default(
                "greeting".to_string(),
                Type::String,
                "\"Hello\"".to_string(),
            ));

        assert_eq!(sig.required_params(), 1);
        assert_eq!(sig.total_params(), 2);
        assert!(sig.validate().is_ok());
    }

    #[test]
    fn test_invalid_default_ordering() {
        let sig = FunctionSignature::new("test".to_string(), Type::I32)
            .add_param(Parameter::with_default(
                "a".to_string(),
                Type::I32,
                "5".to_string(),
            ))
            .add_param(Parameter::new("b".to_string(), Type::I32));

        assert!(sig.validate().is_err());
    }

    #[test]
    fn test_call_resolution_positional() {
        let sig = FunctionSignature::new("greet".to_string(), Type::String)
            .add_param(Parameter::new("name".to_string(), Type::String))
            .add_param(Parameter::with_default(
                "greeting".to_string(),
                Type::String,
                "\"Hello\"".to_string(),
            ));

        let resolver = CallResolver::new(sig);
        let args = vec![Argument::positional("\"Alice\"".to_string())];

        let resolved = resolver.resolve_call(args).unwrap();
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[1].1, "\"Hello\"");
    }

    #[test]
    fn test_call_resolution_named() {
        let sig = FunctionSignature::new("greet".to_string(), Type::String)
            .add_param(Parameter::new("name".to_string(), Type::String))
            .add_param(Parameter::new("greeting".to_string(), Type::String));

        let resolver = CallResolver::new(sig);
        let args = vec![
            Argument::named("greeting".to_string(), "\"Hi\"".to_string()),
            Argument::named("name".to_string(), "\"Bob\"".to_string()),
        ];

        let resolved = resolver.resolve_call(args).unwrap();
        assert_eq!(resolved.len(), 2);
    }

    #[test]
    fn test_call_builder() {
        let (fn_name, args) = CallBuilder::new("foo".to_string())
            .arg("1".to_string())
            .named_arg("x".to_string(), "2".to_string())
            .build();

        assert_eq!(fn_name, "foo");
        assert_eq!(args.len(), 2);
        assert!(!args[0].is_named());
        assert!(args[1].is_named());
    }
}
