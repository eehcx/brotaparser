use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DiagnosticCode {
    SyntaxUnexpectedToken,
    ProfileUnsupported,
    SymbolDuplicate,
    SymbolUnresolved,
    DependencyForbidden,
    DependencyCycle,
    ContractContradiction,
    ContractMissingRoot,
    ContractMissingLayer,
}

impl DiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SyntaxUnexpectedToken => "syntax.unexpected_token",
            Self::ProfileUnsupported => "profile.unsupported",
            Self::SymbolDuplicate => "symbol.duplicate",
            Self::SymbolUnresolved => "symbol.unresolved",
            Self::DependencyForbidden => "dependency.forbidden",
            Self::DependencyCycle => "dependency.cycle",
            Self::ContractContradiction => "contract.contradiction",
            Self::ContractMissingRoot => "contract.missing_root",
            Self::ContractMissingLayer => "contract.missing_layer",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Location {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Location {
    pub fn from_source(source: &str, start: usize, end: usize) -> Self {
        let prefix = &source[..start.min(source.len())];
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
        let column = prefix
            .rsplit_once('\n')
            .map_or(prefix.len() + 1, |(_, current_line)| current_line.len() + 1);

        Self {
            start,
            end,
            line,
            column,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub context: Option<String>,
    pub locations: Vec<Location>,
}

impl Diagnostic {
    pub fn error(
        code: DiagnosticCode,
        message: impl Into<String>,
        context: Option<String>,
        locations: Vec<Location>,
    ) -> Self {
        Self {
            code: code.as_str().to_owned(),
            severity: Severity::Error,
            message: message.into(),
            context,
            locations,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostics {
    pub diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn single(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostics: vec![diagnostic],
        }
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn has_code(&self, code: DiagnosticCode) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == code.as_str())
    }
}

impl std::fmt::Display for Diagnostics {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, diagnostic) in self.diagnostics.iter().enumerate() {
            if index > 0 {
                formatter.write_str("\n")?;
            }
            write!(formatter, "{}: {}", diagnostic.code, diagnostic.message)?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostics {}
