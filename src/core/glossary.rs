// GlossaryLoader — 加载 glossary/zh.yml，为术语提供首次/后续文案
//
// 统一采用中文（English）格式。首次出现附定义，后续仅用中文。
// 启动时加载一次，全局共享。

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

// ---------------------------------------------------------------------------
// 数据结构
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct GlossaryEntry {
    #[serde(rename = "derives-from")]
    #[allow(dead_code)]
    pub derives_from: String,
    pub term: String,
    pub definition: String,
    #[allow(dead_code)]
    pub verified: String,
}

#[derive(Debug, Clone)]
pub struct Glossary {
    entries: HashMap<String, GlossaryEntry>,
}

// ---------------------------------------------------------------------------
// Glossary 实现
// ---------------------------------------------------------------------------

impl Glossary {
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("read glossary: {}", e))?;
        let raw: HashMap<String, GlossaryEntry> =
            serde_yaml::from_str(&content).map_err(|e| format!("parse glossary: {}", e))?;
        Ok(Self { entries: raw })
    }

    /// 获取术语的简要标注：中文（English）
    pub fn label(&self, key: &str) -> String {
        if let Some(entry) = self.entries.get(key) {
            format!("{}（{}）", entry.term, key)
        } else {
            key.to_string()
        }
    }

    /// 获取术语的首次出现文案：中文（English）+ 定义
    pub fn first_use(&self, key: &str) -> String {
        if let Some(entry) = self.entries.get(key) {
            format!("{}（{}）：{}", entry.term, key, entry.definition)
        } else {
            key.to_string()
        }
    }

    /// 为道法术语生成简短溯源标注
    pub fn dao_hint(&self, key: &str) -> String {
        if let Some(entry) = self.entries.get(key) {
            format!("{}（{}）：{}", entry.term, key, entry.definition)
        } else {
            format!("{}：未在 glossary 中定义", key)
        }
    }

    /// 为自然术语（nature/stage/upstream）生成带选项的首次定义
    pub fn nature_help(&self) -> String {
        "spec（系统定义）/ proposal（变更提案）/ decision（架构决策）/ note（实践笔记）".into()
    }

    pub fn stage_help(&self) -> String {
        "1/3 = 初稿待讨论, 2/3 = 方案已收敛可推进（推荐追问后选择）, 3/3 = 已定稿".into()
    }

    pub fn upstream_help(&self) -> String {
        "上游文档（upstream）：授权本文档变更的已有文档 id。上游文档必须 stage ≥ 2/3。根文档可以不填".into()
    }
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_glossary() -> Glossary {
        let yaml = r#"
顺因:
  derives-from: "test"
  term: "顺因"
  definition: "尊重因果方向"
  verified: "240610"
有度:
  derives-from: "test"
  term: "有度"
  definition: "规约不多不少"
  verified: "240610"
"#;
        let entries: HashMap<String, GlossaryEntry> = serde_yaml::from_str(yaml).unwrap();
        Glossary { entries }
    }

    #[test]
    fn test_label() {
        let g = sample_glossary();
        assert_eq!(g.label("顺因"), "顺因（顺因）");
    }

    #[test]
    fn test_first_use() {
        let g = sample_glossary();
        assert!(g.first_use("顺因").contains("尊重因果方向"));
    }

    #[test]
    fn test_dao_hint() {
        let g = sample_glossary();
        let hint = g.dao_hint("有度");
        assert!(hint.contains("有度"));
        assert!(hint.contains("规约不多不少"));
    }

    #[test]
    fn test_unknown_term() {
        let g = sample_glossary();
        assert_eq!(g.label("unknown"), "unknown");
        assert_eq!(g.dao_hint("unknown"), "unknown：未在 glossary 中定义");
    }

    #[test]
    fn test_nature_help() {
        let g = sample_glossary();
        let help = g.nature_help();
        assert!(help.contains("spec"));
        assert!(help.contains("proposal"));
    }
}
