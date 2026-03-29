/// Variadic Functions Implementation
/// Supports both C-style variadics and type-safe Omni variadics
/// 
/// C-style: extern fn printf(fmt: *const c_char, ...) -> i32
/// Omni-style: fn sum_all(values: ...i32) -> i32 { ... }

use crate::ir::{Instruction, Type, Function};
use std::collections::HashMap;

/// Represents a variadic parameter in a function signature
#[derive(Debug, Clone, PartialEq)]
pub struct VariadicParam {
    /// The base type being repeated
    pub element_type: Type,
    /// Whether this is a C-style variadic (uses va_list)
    pub is_c_style: bool,
    /// Minimum number of arguments (for optimization)
    pub min_count: usize,
}

/// Represents a variadic function with flexible argument handling
#[derive(Debug, Clone)]
pub struct VariadicFunction {
    /// Regular parameters
    pub regular_params: Vec<(String, Type)>,
    /// Variadic parameter (only one allowed per function)
    pub variadic_param: Option<VariadicParam>,
    /// Return type
    pub return_type: Type,
    /// Function body
    pub body: Vec<Instruction>,
    /// Whether this uses C-style varargs
    pub is_extern: bool,
}

impl VariadicFunction {
    /// Create a new variadic function
    pub fn new(return_type: Type, is_extern: bool) -> Self {
        VariadicFunction {
            regular_params: Vec::new(),
            variadic_param: None,
            return_type,
            body: Vec::new(),
            is_extern,
        }
    }

    /// Add a regular parameter
    pub fn add_param(mut self, name: String, ty: Type) -> Self {
        self.regular_params.push((name, ty));
        self
    }

    /// Set the variadic parameter
    pub fn set_variadic(mut self, element_type: Type, min_count: usize) -> Self {
        self.variadic_param = Some(VariadicParam {
            element_type,
            is_c_style: self.is_extern,
            min_count,
        });
        self
    }

    /// Generate code for calling this variadic function
    pub fn generate_call(&self, args: Vec<(String, Type)>) -> Result<Vec<Instruction>, String> {
        let mut instructions = Vec::new();
        
        // Validate argument count
        let min_required = self.regular_params.len();
        if args.len() < min_required {
            return Err(format!(
                "Function requires at least {} arguments, got {}",
                min_required,
                args.len()
            ));
        }

        // If C-style variadic, generate va_list handling
        if self.is_extern && self.variadic_param.is_some() {
            instructions.push(Instruction::SetupVaList {
                arg_count: args.len() as u32,
            });
        }

        // Push arguments in order
        for (name, ty) in &args {
            instructions.push(Instruction::PushArg {
                name: name.clone(),
                ty: ty.clone(),
            });
        }

        // If Omni-style variadic, pass count as hidden argument
        if !self.is_extern && self.variadic_param.is_some() {
            let variadic_count = (args.len() - min_required) as u32;
            instructions.push(Instruction::PushArg {
                name: "__variadic_count".to_string(),
                ty: Type::U32,
            });
        }

        Ok(instructions)
    }

    /// Validate variadic argument types
    pub fn validate_variadic_args(&self, args: &[(String, Type)]) -> Result<(), String> {
        if let Some(ref variadic) = self.variadic_param {
            let start_idx = self.regular_params.len();
            
            for (idx, (_, arg_type)) in args.iter().enumerate().skip(start_idx) {
                if arg_type != &variadic.element_type {
                    return Err(format!(
                        "Variadic argument {} has type {:?}, expected {:?}",
                        idx, arg_type, variadic.element_type
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Macros for variadic function declaration in Omni
pub mod macros {
    /// Declare a C-style variadic function
    /// # Example
    /// ```
    /// extern_variadic!(printf, fmt: *c_char, ... -> i32);
    /// ```
    #[macro_export]
    macro_rules! extern_variadic {
        ($name:ident, $($param:ident: $ty:ty),+, ... -> $ret:ty) => {
            extern "C" {
                pub fn $name($($param: $ty),+, ...) -> $ret;
            }
        };
    }

    /// Declare an Omni-style type-safe variadic function
    /// # Example
    /// ```
    /// variadic_fn!(sum_all(values: ...i32) -> i32 { ... });
    /// ```
    #[macro_export]
    macro_rules! variadic_fn {
        ($name:ident($($param:ident: $ty:ty),*, values: ...$elem_ty:ty) -> $ret:ty { $($body:tt)* }) => {
            fn $name($($param: $ty),*) -> $ret {
                // Generated code with variadic handling
                $($body)*
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variadic_function_creation() {
        let vf = VariadicFunction::new(Type::I32, false)
            .add_param("format".to_string(), Type::String)
            .set_variadic(Type::I32, 0);

        assert_eq!(vf.regular_params.len(), 1);
        assert!(vf.variadic_param.is_some());
        assert!(!vf.is_extern);
    }

    #[test]
    fn test_variadic_argument_validation() {
        let vf = VariadicFunction::new(Type::I32, false)
            .add_param("format".to_string(), Type::String)
            .set_variadic(Type::I32, 0);

        let args = vec![
            ("format".to_string(), Type::String),
            ("arg1".to_string(), Type::I32),
            ("arg2".to_string(), Type::I32),
        ];

        assert!(vf.validate_variadic_args(&args).is_ok());
    }

    #[test]
    fn test_variadic_type_mismatch() {
        let vf = VariadicFunction::new(Type::I32, false)
            .add_param("format".to_string(), Type::String)
            .set_variadic(Type::I32, 0);

        let args = vec![
            ("format".to_string(), Type::String),
            ("arg1".to_string(), Type::F64), // Wrong type!
        ];

        assert!(vf.validate_variadic_args(&args).is_err());
    }
}
