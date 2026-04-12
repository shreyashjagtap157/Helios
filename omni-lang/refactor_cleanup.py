import os
import re

files_to_fix = [
    "compiler/src/parser/mod.rs",
    "compiler/src/runtime/bytecode_compiler.rs",
    "compiler/src/runtime/interpreter.rs",
    "compiler/src/semantic/monomorphization.rs",
    "compiler/src/semantic/properties.rs"
]

span_str = " span: crate::parser::ast::Span::default(), "

def cleanup(filepath):
    if not os.path.exists(filepath): return
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    # Revert `-> Node { span...` back to `-> Node {`
    # Match `->\s*\w+\s*\{\s*span:\s*crate::parser::ast::Span::default\(\),\s*`
    patt = re.compile(r'(->\s*(?:Block|Function|Module|StructDef|MacroDef|ModuleDecl)\s*\{)(\s*span:\s*crate::parser::ast::Span::default\(\),\s*)')
    content = patt.sub(r'\1', content)
    
    with open(filepath, 'w', encoding='utf-8') as f:
        f.write(content)

for f in files_to_fix:
    cleanup(f)
print("Cleaned up function return types.")
