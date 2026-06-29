use std::path::Path;

use super::database::SihDatabase;
use super::indexer::{self, IndexReport};
use super::validator::ValidationConfig;

/// 管道配置
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub docs_dir: String,
    pub db_path: String,
    pub validation: ValidationConfig,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            docs_dir: std::env::var("SIHANKOR_DOCS_DIR").unwrap_or_else(|_| "docs/".to_string()),
            db_path: std::env::var("SIHANKOR_DB_PATH")
                .unwrap_or_else(|_| ".sih/index.db".to_string()),
            validation: ValidationConfig::default(),
        }
    }
}

impl PipelineConfig {
    /// 以显式项目根目录构造配置，所有路径解析为绝对路径，不依赖 cwd。
    pub fn with_root(root: &Path) -> Self {
        Self {
            docs_dir: root.join("docs").to_string_lossy().into_owned(),
            db_path: root.join(".sih/index.db").to_string_lossy().into_owned(),
            validation: ValidationConfig::default(),
        }
    }
}

/// 管道执行报告
#[derive(Debug, Clone)]
pub struct PipelineReport {
    pub index: IndexReport,
}

/// 执行完整管道：discover → parse → validate → index
pub async fn run_pipeline(
    db: &dyn SihDatabase,
    docs_dir: &Path,
    config: &ValidationConfig,
) -> PipelineReport {
    let index_report = indexer::rebuild_index(db, docs_dir, config).await;
    PipelineReport {
        index: index_report,
    }
}
