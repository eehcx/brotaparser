#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileDecl {
    pub name: String,
    pub version: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerDecl {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyDecl {
    pub source: String,
    pub target: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstDocument {
    pub profile: ProfileDecl,
    pub root: Option<String>,
    pub root_span: Option<Span>,
    pub layers: Vec<LayerDecl>,
    pub dependencies: Vec<DependencyDecl>,
}
