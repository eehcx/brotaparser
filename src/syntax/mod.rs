use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

use crate::ast::{
    AreaDecl, ContractDocument, ContractMetadata, DependencyRuleDecl, DependencyRuleKind,
    DimensionDecl, ExceptionDecl, ForbiddenPathDecl, PathRuleDecl, ProfileDecl,
    RequiredArtifactDecl, RequiredArtifactKind, RootDecl, RuleMetadataDecl, Span, SymbolDecl,
    VersionDecl,
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
        metadata: ContractMetadata::default(),
    };

    let mut statements = contract.into_inner();
    let contract_name = statements.next().expect("contract name");
    document.name = match contract_name.as_rule() {
        Rule::identifier => contract_name.as_str().to_owned(),
        Rule::quoted => unquote(&contract_name, source)?,
        _ => unreachable!("contract names are identifiers or quoted strings"),
    };
    for statement in statements {
        match statement.as_rule() {
            Rule::language_decl => document.languages.push(parse_language(&statement, source)?),
            Rule::schema_decl => document
                .metadata
                .schemas
                .push(parse_language(&statement, source)?),
            Rule::profile_decl => document.profiles.push(parse_profile(&statement, source)?),
            Rule::root_decl => document.roots.push(parse_root(&statement, source)?),
            Rule::area_decl => document.areas.push(parse_area(&statement)),
            Rule::dimension_decl => document
                .metadata
                .dimensions
                .push(parse_dimension(&statement, source)?),
            Rule::symbols_explicit_decl => document.metadata.symbols_explicit = true,
            Rule::symbol_decl => document
                .metadata
                .symbols
                .push(parse_symbol(&statement, source)?),
            Rule::dependency_decl => document.dependencies.push(parse_dependency(&statement)),
            Rule::exception_decl => document
                .exceptions
                .push(parse_exception(&statement, source)?),
            Rule::path_decl => document.paths.push(parse_path(&statement, source)?),
            Rule::forbidden_path_decl => document
                .forbidden_paths
                .push(parse_forbidden_path(&statement, source)?),
            Rule::artifact_decl => document
                .required_artifacts
                .push(parse_artifact(&statement, source)?),
            Rule::forbidden_paths_state_decl => {
                document.metadata.forbidden_paths_present =
                    Some(statement.as_str().trim_end().ends_with("present"));
            }
            Rule::rules_explicit_decl => document.metadata.rules_explicit = true,
            Rule::rule_decl => document
                .metadata
                .rules
                .push(parse_rule(&statement, source)?),
            _ => unreachable!("statements are silent and expose declaration rules"),
        }
    }

    Ok(document)
}

fn parse_language(pair: &Pair<'_, Rule>, source: &str) -> Result<VersionDecl, Diagnostics> {
    let value = pair.clone().into_inner().next().expect("language version");
    Ok(VersionDecl {
        value: unquote(&value, source)?,
        span: span(pair),
    })
}

fn parse_profile(pair: &Pair<'_, Rule>, source: &str) -> Result<ProfileDecl, Diagnostics> {
    let mut inner = pair.clone().into_inner();
    let name = inner.next().expect("profile name");
    let version = inner.next().expect("profile version");
    Ok(ProfileDecl {
        name: name.as_str().to_owned(),
        version: unquote(&version, source)?,
        span: span(pair),
    })
}

fn parse_root(pair: &Pair<'_, Rule>, source: &str) -> Result<RootDecl, Diagnostics> {
    let value = pair.clone().into_inner().next().expect("root path");
    Ok(RootDecl {
        path: unquote(&value, source)?,
        span: span(pair),
    })
}

fn parse_area(pair: &Pair<'_, Rule>) -> AreaDecl {
    let name = pair.clone().into_inner().next().expect("area name");
    AreaDecl {
        name: name.as_str().to_owned(),
        span: span(pair),
    }
}

fn parse_dimension(pair: &Pair<'_, Rule>, source: &str) -> Result<DimensionDecl, Diagnostics> {
    let mut inner = pair.clone().into_inner();
    let name = inner.next().expect("dimension name");
    let value = inner.next().expect("dimension value");
    Ok(DimensionDecl {
        name: name.as_str().to_owned(),
        value: unquote(&value, source)?,
        span: span(pair),
    })
}

fn parse_symbol(pair: &Pair<'_, Rule>, source: &str) -> Result<SymbolDecl, Diagnostics> {
    let mut inner = pair.clone().into_inner();
    let id = inner.next().expect("symbol id");
    let name = inner.next().expect("symbol name");
    let kind = inner.next().expect("symbol kind");
    Ok(SymbolDecl {
        id: unquote(&id, source)?,
        name: unquote(&name, source)?,
        kind: unquote(&kind, source)?,
        span: span(pair),
    })
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

fn parse_exception(pair: &Pair<'_, Rule>, source_text: &str) -> Result<ExceptionDecl, Diagnostics> {
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
    Ok(ExceptionDecl {
        kind,
        source: source.as_str().to_owned(),
        target: target.as_str().to_owned(),
        through: through.as_str().to_owned(),
        reason: unquote(&reason, source_text)?,
        span: span(pair),
    })
}

fn parse_path(pair: &Pair<'_, Rule>, source_text: &str) -> Result<PathRuleDecl, Diagnostics> {
    let mut inner = pair.clone().into_inner();
    let pattern = inner.next().expect("path pattern");
    let owner = inner.next().expect("path owner");
    Ok(PathRuleDecl {
        pattern: unquote(&pattern, source_text)?,
        owner: owner.as_str().to_owned(),
        span: span(pair),
    })
}

fn parse_forbidden_path(
    pair: &Pair<'_, Rule>,
    source_text: &str,
) -> Result<ForbiddenPathDecl, Diagnostics> {
    let pattern = pair
        .clone()
        .into_inner()
        .next()
        .expect("forbidden path pattern");
    Ok(ForbiddenPathDecl {
        pattern: unquote(&pattern, source_text)?,
        span: span(pair),
    })
}

fn parse_artifact(
    pair: &Pair<'_, Rule>,
    source_text: &str,
) -> Result<RequiredArtifactDecl, Diagnostics> {
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
    Ok(RequiredArtifactDecl {
        kind,
        path: unquote(&path, source_text)?,
        scope,
        span: span(pair),
    })
}

fn parse_rule(pair: &Pair<'_, Rule>, source_text: &str) -> Result<RuleMetadataDecl, Diagnostics> {
    let mut inner = pair.clone().into_inner();
    let id = inner.next().expect("rule id");
    let origin = inner.next().expect("rule origin");
    let verification = inner.next().expect("rule verification level");
    let sources = inner
        .next()
        .expect("rule sources")
        .into_inner()
        .map(|source| unquote(&source, source_text))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(RuleMetadataDecl {
        id: unquote(&id, source_text)?,
        origin: origin.as_str().to_owned(),
        verification: verification.as_str().to_owned(),
        sources,
        span: span(pair),
    })
}

fn span(pair: &Pair<'_, Rule>) -> Span {
    let source_span = pair.as_span();
    Span {
        start: source_span.start(),
        end: source_span.end(),
    }
}

fn unquote(value: &Pair<'_, Rule>, source: &str) -> Result<String, Diagnostics> {
    serde_json::from_str(value.as_str()).map_err(|error| {
        let value_span = value.as_span();
        Diagnostics::single(Diagnostic::error(
            DiagnosticCode::SyntaxUnexpectedToken,
            format!("invalid quoted string: {error}"),
            None,
            vec![Location::from_source(
                source,
                value_span.start(),
                value_span.end(),
            )],
        ))
    })
}
