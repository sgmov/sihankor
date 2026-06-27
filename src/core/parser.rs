use std::path::Path;

use super::models::{DocStatus, Document, Frontmatter, Stage};
use chrono::Utc;
use serde_yaml;

/// 解析错误
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Missing required frontmatter field: {0}")]
    MissingField(String),
    #[error("Invalid stage: {0}")]
    InvalidStage(String),
    #[error("No frontmatter found in document")]
    NoFrontmatter,
    #[error("Frontmatter not closed: missing closing ---")]
    UnclosedFrontmatter,
}

/// 从 .sih.md 文件解析文档
pub fn parse_file(path: &Path) -> Result<Document, ParseError> {
    let content = std::fs::read_to_string(path)?;
    parse_content(&content)
}

/// 从字符串内容解析文档
pub fn parse_content(content: &str) -> Result<Document, ParseError> {
    let (frontmatter_raw, body) = extract_frontmatter(content)?;
    let frontmatter = parse_frontmatter(&frontmatter_raw)?;

    let title = extract_title(body).unwrap_or_else(|| frontmatter.id.clone());
    let doc_content = body.to_string();

    Ok(Document {
        id: frontmatter.id.clone(),
        stage: frontmatter.stage.clone(),
        title,
        upstream: frontmatter.upstream.clone(),
        frontmatter,
        content: doc_content,
        status: DocStatus::Ok,
        indexed_at: Utc::now(),
        nature: String::new(), // set by indexer after parsing
    })
}

/// 提取 frontmatter 和正文
fn extract_frontmatter(content: &str) -> Result<(String, &str), ParseError> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err(ParseError::NoFrontmatter);
    }

    // 跳过开头的 ---
    let after_first = &trimmed[3..];
    let after_first = after_first.trim_start_matches(['\r', '\n']);

    // 找到闭合的 ---
    if let Some(end_pos) = after_first.find("\n---") {
        let yaml_content = &after_first[..end_pos];
        let body_start = end_pos + 4; // 跳过 \n---
        let body = after_first[body_start..].trim_start_matches(['\r', '\n']);
        Ok((yaml_content.to_string(), body))
    } else if let Some(end_pos) = after_first.find("---") {
        let yaml_content = &after_first[..end_pos];
        let body_start = end_pos + 3;
        let body = after_first[body_start..].trim_start_matches(['\r', '\n']);
        Ok((yaml_content.to_string(), body))
    } else {
        Err(ParseError::UnclosedFrontmatter)
    }
}

/// 解析 YAML frontmatter
fn parse_frontmatter(yaml_str: &str) -> Result<Frontmatter, ParseError> {
    let raw_value: serde_yaml::Value = serde_yaml::from_str(yaml_str)?;

    // 提取必填字段: id 和 stage
    let id = raw_value
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ParseError::MissingField("id".to_string()))?
        .to_string();

    // type 字段已废除，不在 frontmatter 中解析
    // document nature 由所在目录推断

    let stage_str = raw_value
        .get("stage")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ParseError::MissingField("stage".to_string()))?;

    let stage = match Stage::from_str(stage_str) {
        Some(s) => s,
        None => return Err(ParseError::InvalidStage(stage_str.to_string())),
    };

    let upstream = raw_value
        .get("upstream")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let decided_by = raw_value
        .get("decided-by")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // 将 YAML 转为 JSON Value 作为 extra 兜底
    let json_value = yaml_value_to_json(&raw_value);

    // 从 extra 中移除已展开的字段
    let extra = remove_known_fields(json_value);

    Ok(Frontmatter {
        id,
        stage,
        upstream,
        decided_by,
        extra,
    })
}

/// 从正文中提取标题（第一个 # 开头的行）
fn extract_title(body: &str) -> Option<String> {
    body.lines()
        .find(|line| line.trim().starts_with("# "))
        .map(|line| line.trim()[2..].trim().to_string())
}

/// YAML Value → JSON Value
fn yaml_value_to_json(yaml: &serde_yaml::Value) -> serde_json::Value {
    serde_json::to_value(yaml).unwrap_or(serde_json::Value::Null)
}

/// 从 JSON 对象中移除已知字段，保留 extra
fn remove_known_fields(mut json: serde_json::Value) -> serde_json::Value {
    if let serde_json::Value::Object(map) = &mut json {
        map.remove("id");
        map.remove("stage");
        map.remove("upstream");
        map.remove("decided-by");
    }
    json
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_document() {
        let content = r#"---
id: 260613-1800-test-doc
stage: 1/3
upstream: 240602-0900-on-sihankor
---
# Test Document
This is the body.
"#;
        let doc = parse_content(content).unwrap();
        assert_eq!(doc.id, "260613-1800-test-doc");
        assert_eq!(doc.stage.to_display(), "1/3");
        assert_eq!(doc.upstream, Some("240602-0900-on-sihankor".to_string()));
        assert_eq!(doc.title, "Test Document");
        assert!(doc.content.contains("This is the body."));
    }

    #[test]
    fn test_parse_missing_id() {
        let content = r#"---
stage: 1/3
---
# No ID
"#;
        let result = parse_content(content);
        assert!(result.is_err());
        match result.unwrap_err() {
            ParseError::MissingField(field) => assert_eq!(field, "id"),
            _ => panic!("Expected MissingField error"),
        }
    }

    #[test]
    fn test_parse_invalid_stage() {
        let content = r#"---
id: 260613-1800-test
stage: invalid
---
# Test
"#;
        let result = parse_content(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_no_frontmatter() {
        let content = "Just some text without frontmatter";
        let result = parse_content(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_note_without_upstream() {
        let content = r#"---
id: 260613-1800-test-note
stage: 1/3
---
# A Note
Some content.
"#;
        let doc = parse_content(content).unwrap();
        assert_eq!(doc.upstream, None);
    }

    #[test]
    fn test_parse_extra_fields() {
        let content = r#"---
id: 260613-1800-test
stage: 2/3
upstream: 240602-0900-on-sihankor
custom-field: some-value
---
# Test
"#;
        let doc = parse_content(content).unwrap();
        assert!(doc.frontmatter.extra.get("custom-field").is_some());
    }
}
