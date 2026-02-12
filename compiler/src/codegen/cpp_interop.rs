#![allow(dead_code)]
//! C++ Interoperability Layer
//! 
//! Handles Itanium C++ ABI name mangling and VTable generation.

use crate::parser::ast::{Type, Ownership};

/// Mangle a function name according to Itanium C++ ABI
pub fn mangle_function(name: &str, params: &[Type], class_name: Option<&str>) -> String {
    let mut mangled = String::from("_Z");

    // Nested name (Class::Method)
    if let Some(cls) = class_name {
        mangled.push('N');
        mangled.push_str(&format!("{}{}", cls.len(), cls));
        mangled.push_str(&format!("{}{}", name.len(), name));
        mangled.push('E');
    } else {
        // Free function
        mangled.push_str(&format!("{}{}", name.len(), name));
    }

    // Parameters
    if params.is_empty() {
        mangled.push('v'); // void
    } else {
        for param in params {
            mangled.push_str(&mangle_type(param));
        }
    }

    mangled
}

/// Mangle a single type
fn mangle_type(ty: &Type) -> String {
    match ty {
        Type::U8 => "h".to_string(),   // unsigned char
        Type::U16 => "t".to_string(),  // unsigned short
        Type::U32 => "j".to_string(),  // unsigned int
        Type::U64 => "m".to_string(),  // unsigned long
        Type::I8 => "a".to_string(),   // signed char
        Type::I16 => "s".to_string(),  // short
        Type::I32 => "i".to_string(),  // int
        Type::I64 => "l".to_string(),  // long
        Type::F32 => "f".to_string(),  // float
        Type::F64 => "d".to_string(),  // double
        Type::Bool => "b".to_string(), // bool
        Type::Str => "Pc".to_string(), // char* (approximation)
        Type::Named(name) => {
            // Class type: 3Mat
            format!("{}{}", name.len(), name)
        },
        Type::WithOwnership(inner, own) => {
            match own {
                Ownership::RawPointer => format!("P{}", mangle_type(inner)), // *T
                Ownership::Borrow | Ownership::BorrowMut => format!("R{}", mangle_type(inner)), // &T
                _ => mangle_type(inner),
            }
        },
        _ => "v".to_string(), // Fallback void
    }
}

/// Generate a VTable structure for a C++ class
pub fn generate_vtable(class_name: &str, methods: &[&str]) -> String {
    let mut vtable = String::new();
    vtable.push_str(&format!("struct {}_VTable {{\n", class_name));
    
    for (i, _method) in methods.iter().enumerate() {
        vtable.push_str(&format!("    fn_{}: usize,\n", i));
    }
    
    vtable.push_str("}");
    vtable
}
