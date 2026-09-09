use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

use crate::ast::{
    AreaDecl, ContractDocument, DependencyRuleDecl, DependencyRuleKind, ExceptionDecl,
    ForbiddenPathDecl, PathRuleDecl, ProfileDecl, RequiredArtifactDecl, RequiredArtifactKind,
    RootDecl, Span, VersionDecl,
};
use crate::diagnostics::{Diagnostic, DiagnosticCode, Diagnostics, Location};

#[derive(Parser)]
#[grammar = "syntax/grammar.pest"]
struct ArchitectureParser;

pub fn parse(source: &str) -> Result<ContractDocument, Diagnostics> {
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

    let contract = parsed
        .next()
        .and_then(|pair| pair.into_inner().next())
        .expect("the contract rule always produces one pair");
    let mut document = ContractDocument {
        name: String::new(),
        span: span(&contract),
        languages: Vec::new(),
        profiles: Vec::new(),
        roots: Vec::new(),
        areas: Vec::new(),
        dependencies: Vec::new(),
        exceptions: Vec::new(),
        paths: Vec::new(),
        forbidden_paths: Vec::new(),
        required_artifacts: Vec::new(),
    };

    let mut statements = contract.into_inner();
    document.name = statements
        .next()
        .expect("contract name")
        .as_str()
        .to_owned();
    for statement in statements {
        match statement.as_rule() {
            Rule::language_decl => document.languages.push(parse_language(&statement)),
            Rule::profile_decl => document.profiles.push(parse_profile(&statement)),
            Rule::root_decl => document.roots.push(parse_root(&statement)),
            Rule::area_decl => document.areas.push(parse_area(&statement)),
            Rule::dependency_decl => document.dependencies.push(parse_dependency(&statement)),
            Rule::exception_decl => document.exceptions.push(parse_exception(&statement)),
            Rule::path_decl => document.paths.push(parse_path(&statement)),
            Rule::forbidden_path_decl => document
                .forbidden_paths
                .push(parse_forbidden_path(&statement)),
            Rule::artifact_decl => document.required_artifacts.push(parse_artifact(&statement)),
            _ => unreachable!("statements are silent and expose declaration rules"),
        }
    }

    Ok(document)
}

fn parse_language(pair: &Pair<'_, Rule>) -> VersionDecl {
    let value = pair.clone().into_inner().next().expect("language version");
    VersionDecl {
        value: unquote(value.as_str()),
        span: span(pair),
    }
}

fn parse_profile(pair: &Pair<'_, Rule>) -> ProfileDecl {
    let mut inner = pair.clone().into_inner();
    let name = inner.next().expect("profile name");
    let version = inner.next().expect("profile version");
    ProfileDecl {
        name: name.as_str().to_owned(),
        version: unquote(version.as_str()),
        span: span(pair),
    }
}

fn parse_root(pair: &Pair<'_, Rule>) -> RootDecl {
    let value = pair.clone().into_inner().next().expect("root path");
    RootDecl {
        path: unquote(value.as_str()),
        span: span(pair),
    }
}

fn parse_area(pair: &Pair<'_, Rule>) -> AreaDecl {
    let name = pair.clone().into_inner().next().expect("area name");
    AreaDecl {
        name: name.as_str().to_owned(),
        span: span(pair),
    }
}

fn parse_dependency(pair: &Pair<'_, Rule>) -> DependencyRuleDecl {
    let mut inner = pair.clone().into_inner();
    let kind = match inner.next().expect("dependency kind").as_str() {
        "allow" => DependencyRuleKind::Allow,
        "forbid" => DependencyRuleKind::Forbid,
        "require" => DependencyRuleKind::Require,
        _ => unreachable!("grammar restricts dependency kinds"),
    };
    let source = inner.next().expect("dependency source");
    let target = inner.next().expect("dependency target");
    DependencyRuleDecl {
        kind,
        source: source.as_str().to_owned(),
        target: target.as_str().to_owned(),
        span: span(pair),
    }
}

fn parse_exception(pair: &Pair<'_, Rule>) -> ExceptionDecl {
    let mut inner = pair.clone().into_inner();
    let kind = match inner.next().expect("exception kind").as_str() {
        "allow" => DependencyRuleKind::Allow,
        "forbid" => DependencyRuleKind::Forbid,
        "require" => DependencyRuleKind::Require,
        _ => unreachable!("grammar restricts dependency kinds"),
    };
    let source = inner.next().expect("exception source");
    let target = inner.next().expect("exception target");
    let through = inner
        .next()
        .expect("exception through clause")
        .into_inner()
        .next()
        .expect("exception channel");
    let reason = inner
        .next()
        .expect("exception reason clause")
        .into_inner()
        .next()
        .expect("exception reason");
    ExceptionDecl {
        kind,
        source: source.as_str().to_owned(),
        target: target.as_str().to_owned(),
        through: through.as_str().to_owned(),
        reason: unquote(reason.as_str()),
        span: span(pair),
    }
}

fn parse_path(pair: &Pair<'_, Rule>) -> PathRuleDecl {
    let mut inner = pair.clone().into_inner();
    let pattern = inner.next().expect("path pattern");
    let owner = inner.next().expect("path owner");
    PathRuleDecl {
        pattern: unquote(pattern.as_str()),
        owner: owner.as_str().to_owned(),
        span: span(pair),
    }
}

fn parse_forbidden_path(pair: &Pair<'_, Rule>) -> ForbiddenPathDecl {
    let pattern = pair
        .clone()
        .into_inner()
        .next()
        .expect("forbidden path pattern");
    ForbiddenPathDecl {
        pattern: unquote(pattern.as_str()),
        span: span(pair),
    }
}

fn parse_artifact(pair: &Pair<'_, Rule>) -> RequiredArtifactDecl {
    let mut inner = pair.clone().into_inner();
    let kind = match inner.next().expect("artifact kind").as_str() {
        "file" => RequiredArtifactKind::File,
        "directory" => RequiredArtifactKind::Directory,
        "public-api" => RequiredArtifactKind::PublicApi,
        "export" => RequiredArtifactKind::Export,
        "declaration" => RequiredArtifactKind::Declaration,
        "configuration" => RequiredArtifactKind::Configuration,
        _ => unreachable!("grammar restricts artifact kinds"),
    };
    let path = inner.next().expect("artifact path");
    let scope = inner.next().map(|scope| {
        scope
            .into_inner()
            .next()
            .expect("area scope")
            .as_str()
            .to_owned()
    });
    RequiredArtifactDecl {
        kind,
        path: unquote(path.as_str()),
        scope,
        span: span(pair),
    }
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
