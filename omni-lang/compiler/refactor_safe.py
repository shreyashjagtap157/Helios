import re

files_to_fix = [
    "src/parser/mod.rs",
    "src/runtime/bytecode_compiler.rs",
    "src/runtime/interpreter.rs",
    "src/semantic/monomorphization.rs",
    "src/semantic/properties.rs"
]

def fix_spans(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    # Avoid matching struct definitions: `pub struct Block {`
    # We want to match instantiation: `Block {`
    # Replace `Block {` with `Block { span: Default::default(), `
    # Note: `span: crate::parser::ast::Span::default(),` is safer across the crate
    span_init = "span: crate::parser::ast::Span::default(),"
    
    # Negative lookbehind to ensure we don't match `struct Name {`
    patt = re.compile(r'(?<!struct\s)(?<!struct)(?<!struct\s\b)\b(Block|Function|Module|StructDef|MacroDef|ModuleDecl)(\s*\{)')
    
    content = patt.sub(r'\1\2 ' + span_init + ' ', content)
    
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)

for f in files_to_fix:
    fix_spans(f)

print("Injected spans safely after braces.")
