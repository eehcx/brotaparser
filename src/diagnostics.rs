use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DiagnosticCode {
    SyntaxUnexpectedToken,
    LanguageUnsupported,
    ProfileUnsupported,
    ContractMissingDeclaration,
    ContractDuplicateDeclaration,
    ContractMetadataInvalid,
    SymbolDuplicate,
    SymbolUnresolved,
    DependencyContradiction,
    DependencyDuplicate,
    ExceptionInvalid,
    PathInvalid,
    ArtifactInvalid,
    PathDuplicate,
    PathAmbiguous,
    ArtifactDuplicate,
    BlueprintIncompatible,
    ConformanceOwnership,
    ConformanceArtifactMissing,
    ConformanceDependency,
    ConformanceUnresolved,
}

impl DiagnosticCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SyntaxUnexpectedToken => "BRT-SYN-001",
            Self::LanguageUnsupported => "BRT-PRO-001",
            Self::ProfileUnsupported => "BRT-PRO-002",
            Self::ContractMissingDeclaration => "BRT-CON-001",
            Self::ContractDuplicateDeclaration => "BRT-CON-002",
            Self::ContractMetadataInvalid => "BRT-CON-003",
            Self::SymbolDuplicate => "BRT-SEM-001",
            Self::SymbolUnresolved => "BRT-SEM-002",
            Self::DependencyContradiction => "BRT-DEP-001",
            Self::DependencyDuplicate => "BRT-DEP-002",
            Self::ExceptionInvalid => "BRT-DEP-003",
            Self::PathInvalid => "BRT-PATH-001",
            Self::ArtifactInvalid => "BRT-PATH-002",
            Self::PathDuplicate => "BRT-PATH-003",
            Self::PathAmbiguous => "BRT-PATH-004",
            Self::ArtifactDuplicate => "BRT-PATH-005",
            Self::BlueprintIncompatible => "BRT-BLU-001",
            Self::ConformanceOwnership => "BRT-CONF-001",
            Self::ConformanceArtifactMissing => "BRT-CONF-002",
            Self::ConformanceDependency => "BRT-CONF-003",
            Self::ConformanceUnresolved => "BRT-CONF-004",
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
        let safe_start = safe_boundary(source, start);
        let safe_end = safe_boundary(source, end).max(safe_start);
        let prefix = &source[..safe_start];
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
        let column = prefix.rsplit_once('\n').map_or_else(
            || prefix.chars().count() + 1,
            |(_, current_line)| current_line.chars().count() + 1,
        );

        Self {
            start: safe_start,
            end: safe_end,
            line,
            column,
        }
    }
}

fn safe_boundary(source: &str, offset: usize) -> usize {
    let mut boundary = offset.min(source.len());
    while boundary > 0 && !source.is_char_boundary(boundary) {
        boundary -= 1;
    }
    boundary
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

    pub fn sort_stably(&mut self) {
        self.diagnostics.sort_by(|left, right| {
            let left_location = left.locations.first();
            let right_location = right.locations.first();
            (
                left_location.map_or(usize::MAX, |location| location.start),
                left.code.as_str(),
                left.message.as_str(),
            )
                .cmp(&(
                    right_location.map_or(usize::MAX, |location| location.start),
                    right.code.as_str(),
                    right.message.as_str(),
                ))
        });
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
