use brotaparser::{DiagnosticCode, Severity, compile_source, parse, serialize_json};

const VALID_SOURCE: &str = r#"
profile feature-clean@0.1
root "src/app/features/{feature}"

layer domain
layer application
layer infrastructure
layer presentation

depends application on domain
depends infrastructure on application
depends infrastructure on domain
depends presentation on application
depends presentation on domain
"#;

#[test]
fn parses_a_valid_feature_clean_document_and_preserves_spans() {
    let document = parse(VALID_SOURCE).expect("valid source should parse");

    assert_eq!(document.profile.name, "feature-clean");
    assert_eq!(document.profile.version, "0.1");
    assert_eq!(document.root.as_deref(), Some("src/app/features/{feature}"));
    assert_eq!(document.layers.len(), 4);
    assert_eq!(document.layers[0].name, "domain");
    assert!(document.layers[0].span.start < document.layers[0].span.end);
    assert_eq!(document.dependencies.len(), 5);
}

#[test]
fn malformed_syntax_returns_a_syntax_diagnostic() {
    let error = parse("profile feature-clean@0.1\nlayer").expect_err("source is malformed");

    assert!(error.has_code(DiagnosticCode::SyntaxUnexpectedToken));
    assert!(error.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == Severity::Error && !diagnostic.locations.is_empty()
    }));
}

#[test]
fn parsing_does_not_run_semantic_validation() {
    let source = "profile feature-clean@0.1\nlayer domain\ndepends domain on missing";

    let document = parse(source).expect("the syntax is valid");

    assert_eq!(document.dependencies[0].target, "missing");
}

#[test]
fn compiles_a_valid_document() {
    let contract = compile_source(VALID_SOURCE).expect("valid source should compile");

    assert_eq!(contract.profile.name, "feature-clean");
    assert_eq!(contract.profile.version, "0.1");
    assert_eq!(contract.dimensions.decomposition, "feature");
    assert_eq!(contract.symbols.len(), 4);
    assert_eq!(contract.dependencies.len(), 5);
}

#[test]
fn rejects_unsupported_profiles() {
    let error = compile_source("profile vertical-slice@0.1\nroot \"src\"\n")
        .expect_err("unsupported profiles must not be downgraded");

    assert!(error.has_code(DiagnosticCode::ProfileUnsupported));
}

#[test]
fn rejects_duplicate_and_unresolved_symbols() {
    let source = "profile feature-clean@0.1\nroot \"src\"\nlayer domain\nlayer domain\ndepends application on domain\n";
    let error = compile_source(source).expect_err("invalid symbols must fail");

    assert!(error.has_code(DiagnosticCode::SymbolDuplicate));
    assert!(error.has_code(DiagnosticCode::SymbolUnresolved));
}

#[test]
fn rejects_forbidden_dependencies_and_cycles() {
    let source = "profile feature-clean@0.1\nroot \"src\"\nlayer domain\nlayer application\ndepends application on domain\ndepends domain on application\n";
    let error = compile_source(source).expect_err("invalid dependency graph must fail");

    assert!(error.has_code(DiagnosticCode::DependencyForbidden));
    assert!(error.has_code(DiagnosticCode::DependencyCycle));
}

#[test]
fn normalizes_declaration_order_deterministically() {
    let first = compile_source(VALID_SOURCE).expect("first source should compile");
    let second = compile_source(
        "profile feature-clean@0.1\nroot \"src/app/features/{feature}\"\nlayer presentation\nlayer infrastructure\nlayer application\nlayer domain\ndepends presentation on domain\ndepends presentation on application\ndepends infrastructure on domain\ndepends infrastructure on application\ndepends application on domain\n",
    )
    .expect("second source should compile");

    assert_eq!(first, second);
}

#[test]
fn serializes_and_deserializes_canonically() {
    let contract = compile_source(VALID_SOURCE).expect("source should compile");
    let first = serialize_json(&contract).expect("contract should serialize");
    let decoded: brotaparser::ArchitectureContract =
        serde_json::from_str(&first).expect("contract should deserialize");
    let second = serialize_json(&decoded).expect("decoded contract should serialize");

    assert_eq!(first, second);
}
