use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

use crate::ast::{AstDocument, DependencyDecl, LayerDecl, ProfileDecl, Span};
use crate::diagnostics::{Diagnostic, DiagnosticCode, Diagnostics, Location};

#[derive(Parser)]
#[grammar = "syntax/grammar.pest"]
struct ArchitectureParser;

pub fn parse(source: &str) -> Result<AstDocument, Diagnostics> {
    let mut parsed = ArchitectureParser::parse(Rule::document, source).map_err(|error| {
        let (start, end) = match error.location {
            pest::error::InputLocation::Pos(position) => (position, position + 1),
            pest::error::InputLocation::Span((start, end)) => (start, end),
        };
        Diagnostics::single(Diagnostic::error(
            DiagnosticCode::SyntaxUnexpectedToken,
            error.to_string(),
            None,
            vec![Location::from_source(source, start, end)],
        ))
    })?;

    let document = parsed
        .next()
        .expect("the document rule always produces one pair");
    let mut profile = None;
    let mut root = None;
    let mut root_span = None;
    let mut layers = Vec::new();
    let mut dependencies = Vec::new();

    for pair in document.into_inner() {
        match pair.as_rule() {
            Rule::profile_line => {
                let mut inner = pair.clone().into_inner();
                let name = inner.next().expect("profile name");
                let version = inner.next().expect("profile version");
                profile = Some(ProfileDecl {
                    name: name.as_str().to_owned(),
                    version: version.as_str().to_owned(),
                    span: span(&pair),
                });
            }
            Rule::root_line => {
                let quoted = pair.clone().into_inner().next().expect("root path");
                root = Some(unquote(quoted.as_str()));
                root_span = Some(span(&pair));
            }
            Rule::layer_line => {
                let name = pair.clone().into_inner().next().expect("layer name");
                layers.push(LayerDecl {
                    name: name.as_str().to_owned(),
                    span: span(&pair),
                });
            }
            Rule::dependency_line => {
                let mut inner = pair.clone().into_inner();
                let source_name = inner.next().expect("dependency source");
                let target_name = inner.next().expect("dependency target");
                dependencies.push(DependencyDecl {
                    source: source_name.as_str().to_owned(),
                    target: target_name.as_str().to_owned(),
                    span: span(&pair),
                });
            }
            Rule::blank_line => {}
            Rule::EOI => {}
            _ => unreachable!("line is silent and only exposes declaration rules"),
        }
    }

    let profile = profile.ok_or_else(|| {
        Diagnostics::single(Diagnostic::error(
            DiagnosticCode::SyntaxUnexpectedToken,
            "document must declare a profile",
            None,
            vec![Location::from_source(source, 0, source.len().min(1))],
        ))
    })?;

    Ok(AstDocument {
        profile,
        root,
        root_span,
        layers,
        dependencies,
    })
}

fn span(pair: &Pair<'_, Rule>) -> Span {
    let source_span = pair.as_span();
    Span {
        start: source_span.start(),
        end: source_span.end(),
    }
}

fn unquote(value: &str) -> String {
    value[1..value.len() - 1].to_owned()
}
