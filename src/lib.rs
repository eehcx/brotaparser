mod ast;
mod contract;
mod diagnostics;
mod semantic;
mod syntax;

pub use ast::{AstDocument, DependencyDecl, LayerDecl, ProfileDecl, Span};
pub use contract::{
    ArchitectureContract, DependencyEdge, Dimensions, ProfileVersion, ResolvedSymbol,
};
pub use diagnostics::{Diagnostic, DiagnosticCode, Diagnostics, Location, Severity};

pub fn parse(source: &str) -> Result<AstDocument, Diagnostics> {
    syntax::parse(source)
}

pub fn compile(document: &AstDocument, source: &str) -> Result<ArchitectureContract, Diagnostics> {
    semantic::compile(document, source)
}

pub fn compile_source(source: &str) -> Result<ArchitectureContract, Diagnostics> {
    let document = parse(source)?;
    compile(&document, source)
}

pub fn serialize_json(contract: &ArchitectureContract) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(contract)
}
