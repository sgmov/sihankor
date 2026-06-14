use sihankor::core::database::{SihDatabase, SqliteBackend};
use sihankor::core::indexer;
use sihankor::core::models::DocType;
use sihankor::core::parser;
use sihankor::core::validator::{validate_document, ValidationConfig};

use std::path::Path;

#[tokio::test]
async fn test_index_real_docs() {
    let db = SqliteBackend::open_in_memory().unwrap();
    let docs_dir = Path::new("docs");
    let config = ValidationConfig::default();

    let report = indexer::rebuild_index(&db, docs_dir, &config).await;

    println!("Discovered: {}", report.discovered);
    println!("Parsed: {}", report.parsed);
    println!("Indexed: {}", report.indexed);
    println!("Warnings: {}", report.warnings.len());
    println!("Errors: {}", report.errors.len());

    for (path, error) in &report.errors {
        println!("  ERROR: {} - {}", path, error);
    }

    for (id, warning) in &report.warnings {
        println!("  WARNING: {} - {}", id, warning);
    }

    // 至少应该能解析出一些文档
    assert!(report.discovered > 0, "Should discover documents in docs/");
    assert!(report.parsed > 0, "Should parse at least some documents");

    // 验证数据库中有数据
    let total = db.count_documents().await.unwrap();
    assert_eq!(total, report.indexed);
}

#[tokio::test]
async fn test_search_and_resolve() {
    let db = SqliteBackend::open_in_memory().unwrap();
    let docs_dir = Path::new("docs");
    let config = ValidationConfig::default();

    indexer::rebuild_index(&db, docs_dir, &config).await;

    // 测试搜索
    let results = db.search_content("司衡").await.unwrap();
    println!("Search for '司衡': {} results", results.len());
    assert!(!results.is_empty(), "Should find documents mentioning 司衡");

    // 测试按类型查询
    let treatises = db.search_by_type(&DocType::Treatise).await.unwrap();
    println!("Treatises: {}", treatises.len());

    // 测试授权链
    let by_stage = db.count_by_stage().await.unwrap();
    println!("By stage: {:?}", by_stage);

    let by_type = db.count_by_type().await.unwrap();
    println!("By type: {:?}", by_type);
}

#[test]
fn test_parse_philosophy_docs() {
    let philosophy_dir = Path::new("docs/specs/philosophy");

    if !philosophy_dir.exists() {
        eprintln!("Philosophy dir not found, skipping");
        return;
    }

    let mut parsed = 0;
    let mut errors = Vec::new();

    for entry in std::fs::read_dir(philosophy_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map(|e| e == "md").unwrap_or(false) {
            match parser::parse_file(&path) {
                Ok(doc) => {
                    parsed += 1;
                    println!("Parsed: {} ({}/{}) - {}", doc.id, doc.r#type.as_str(), doc.stage.0, doc.title);

                    // 哲学文档应该都是 3/3
                    assert_eq!(doc.stage.0, "3/3", "Philosophy docs should be at 3/3");
                }
                Err(e) => {
                    errors.push((path.to_string_lossy().to_string(), e.to_string()));
                }
            }
        }
    }

    assert!(parsed > 0, "Should parse at least one philosophy doc");
    for (path, error) in &errors {
        eprintln!("Parse error for {}: {}", path, error);
    }
    assert!(errors.is_empty(), "All philosophy docs should parse successfully");
}

#[test]
fn test_validate_all_docs() {
    let docs_dir = Path::new("docs");

    if !docs_dir.exists() {
        eprintln!("Docs dir not found, skipping");
        return;
    }

    let config = ValidationConfig::default();
    let mut total_violations = 0;

    for entry in walkdir::WalkDir::new(docs_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
            match parser::parse_file(path) {
                Ok(doc) => {
                    let result = validate_document(&doc, Some(path), &config);
                    if !result.violations.is_empty() {
                        println!("\n[{}] {} violations:", doc.id, result.violations.len());
                        for v in &result.violations {
                            println!("  [{}] {} ({}): {}", v.severity.as_str(), v.rule_id, v.location, v.message);
                        }
                        total_violations += result.violations.len();
                    }
                }
                Err(e) => {
                    println!("Parse error for {}: {}", path.display(), e);
                }
            }
        }
    }

    println!("\nTotal violations across all docs: {}", total_violations);
}
