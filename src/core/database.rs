use async_trait::async_trait;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::sync::Mutex;

use super::models::{ChainNode, DocStatus, Document, SearchResult, Stage};

/// 数据库抽象 trait：契约不泄漏实现细节
#[async_trait]
pub trait SihDatabase: Send + Sync {
    async fn upsert_document(&self, doc: Document) -> Result<(), DatabaseError>;
    async fn get_document(&self, id: &str) -> Result<Option<Document>, DatabaseError>;
    async fn search_by_nature(&self, _nature: &str) -> Result<Vec<Document>, DatabaseError>;
    async fn search_content(&self, query: &str) -> Result<Vec<SearchResult>, DatabaseError>;
    async fn resolve_chain(&self, id: &str, depth: u32) -> Result<Vec<ChainNode>, DatabaseError>;
    async fn delete_document(&self, id: &str) -> Result<(), DatabaseError>;
    async fn count_documents(&self) -> Result<usize, DatabaseError>;
    async fn count_by_stage(&self) -> Result<Vec<(String, usize)>, DatabaseError>;
    async fn count_by_nature(&self) -> Result<Vec<(String, usize)>, DatabaseError>;
    async fn get_documents_by_status(&self, status: &DocStatus) -> Result<Vec<Document>, DatabaseError>;
    async fn get_all_documents(&self) -> Result<Vec<Document>, DatabaseError>;
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Database not initialized")]
    NotInitialized,
    #[error("Document not found: {0}")]
    NotFound(String),
}

/// SQLite 后端实现
pub struct SqliteBackend {
    conn: Mutex<Connection>,
}

impl SqliteBackend {
    /// 打开或创建数据库
    pub fn open(path: &Path) -> Result<Self, DatabaseError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(path)?;
        let backend = Self {
            conn: Mutex::new(conn),
        };
        backend.initialize_schema()?;
        Ok(backend)
    }

    /// 在内存中创建数据库（用于测试）
    pub fn open_in_memory() -> Result<Self, DatabaseError> {
        let conn = Connection::open_in_memory()?;
        let backend = Self {
            conn: Mutex::new(conn),
        };
        backend.initialize_schema()?;
        Ok(backend)
    }

    fn initialize_schema(&self) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|_| DatabaseError::NotInitialized)?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS documents (
                id              TEXT PRIMARY KEY,
                stage           TEXT NOT NULL,
                title           TEXT NOT NULL,
                upstream        TEXT,
                frontmatter_json TEXT NOT NULL,
                content         TEXT NOT NULL,
                status          TEXT NOT NULL DEFAULT 'Ok',
                indexed_at      TEXT NOT NULL,
                nature          TEXT NOT NULL DEFAULT ''
            );

            CREATE INDEX IF NOT EXISTS idx_documents_stage ON documents(stage);
            CREATE INDEX IF NOT EXISTS idx_documents_upstream ON documents(upstream);
            CREATE INDEX IF NOT EXISTS idx_documents_nature ON documents(nature);
            CREATE INDEX IF NOT EXISTS idx_documents_status ON documents(status);
            ",
        )?;
        Ok(())
    }
}

#[async_trait]
impl SihDatabase for SqliteBackend {
    async fn upsert_document(&self, doc: Document) -> Result<(), DatabaseError> {
        let frontmatter_json = serde_json::to_string(&doc.frontmatter)?;
        let indexed_at = doc.indexed_at.to_rfc3339();
        let stage = &doc.stage.0;
        let status = doc.status.as_str();

        let conn = self.conn.lock().map_err(|_| DatabaseError::NotInitialized)?;
        conn.execute(
            "INSERT OR REPLACE INTO documents (id, stage, title, upstream, frontmatter_json, content, status, indexed_at, nature)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                doc.id,
                stage,
                doc.title,
                doc.upstream,
                frontmatter_json,
                doc.content,
                status,
                indexed_at,
                doc.nature,
            ],
        )?;
        Ok(())
    }

    async fn get_document(&self, id: &str) -> Result<Option<Document>, DatabaseError> {
        let conn = self.conn.lock().map_err(|_| DatabaseError::NotInitialized)?;
        let mut stmt = conn.prepare(
            "SELECT id, stage, title, upstream, frontmatter_json, content, status, indexed_at, nature
             FROM documents WHERE id = ?1",
        )?;

        let result = stmt
            .query_row(params![id], |row| {
                Ok(row_to_document(row))
            })
            .optional()?;

        match result {
            Some(doc) => Ok(Some(doc?)),
            None => Ok(None),
        }
    }

    async fn search_by_nature(&self, nature: &str) -> Result<Vec<Document>, DatabaseError> {
        let conn = self.conn.lock().map_err(|_| DatabaseError::NotInitialized)?;
        let mut stmt = conn.prepare(
            "SELECT id, stage, title, upstream, frontmatter_json, content, status, indexed_at, nature
             FROM documents WHERE nature = ?1",
        )?;

        let docs = stmt
            .query_map(params![nature], |row| Ok(row_to_document(row)))?
            .filter_map(|r| r.ok())
            .filter_map(|r| r.ok())
            .collect();

        Ok(docs)
    }

    async fn search_content(&self, query: &str) -> Result<Vec<SearchResult>, DatabaseError> {
        let pattern = format!("%{}%", query);
        let conn = self.conn.lock().map_err(|_| DatabaseError::NotInitialized)?;
        let mut stmt = conn.prepare(
            "SELECT id, stage, title, content FROM documents WHERE content LIKE ?1 OR title LIKE ?1",
        )?;

        let results = stmt
            .query_map(params![pattern], |row| {
                let id: String = row.get(0)?;
                let stage_str: String = row.get(1)?;
                let title: String = row.get(2)?;
                let content: String = row.get(3)?;
                Ok((id, stage_str, title, content))
            })?
            .filter_map(|r| r.ok())
            .map(|(id, stage_str, title, content)| {
                let snippet = extract_snippet(&content, query, 80);
                SearchResult {
                    id,
                    stage: Stage(stage_str),
                    title,
                    snippet,
                    relevance: 1.0,
                }
            })
            .collect();

        Ok(results)
    }

    async fn resolve_chain(&self, id: &str, depth: u32) -> Result<Vec<ChainNode>, DatabaseError> {
        let conn = self.conn.lock().map_err(|_| DatabaseError::NotInitialized)?;

        // 使用递归 CTE 追溯 upstream 链
        let mut stmt = conn.prepare(
            "
            WITH RECURSIVE chain AS (
                SELECT id, stage, title, upstream, 0 as depth
                FROM documents WHERE id = ?1
                UNION ALL
                SELECT d.id, d.stage, d.title, d.upstream, c.depth + 1
                FROM documents d
                INNER JOIN chain c ON d.id = c.upstream
                WHERE c.depth < ?2 AND c.upstream IS NOT NULL
            )
            SELECT id, stage, title, upstream, depth FROM chain ORDER BY depth
            ",
        )?;

        let nodes = stmt
            .query_map(params![id, depth], |row| {
                let id: String = row.get(0)?;
                let stage_str: String = row.get(1)?;
                let title: String = row.get(2)?;
                let upstream: Option<String> = row.get(3)?;
                let d: u32 = row.get(4)?;
                Ok(ChainNode {
                    id,
                    stage: Stage(stage_str),
                    title,
                    upstream,
                    depth: d,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(nodes)
    }

    async fn delete_document(&self, id: &str) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().map_err(|_| DatabaseError::NotInitialized)?;
        conn.execute("DELETE FROM documents WHERE id = ?1", params![id])?;
        Ok(())
    }

    async fn count_documents(&self) -> Result<usize, DatabaseError> {
        let conn = self.conn.lock().map_err(|_| DatabaseError::NotInitialized)?;
        let count: usize = conn
            .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;
        Ok(count)
    }

    async fn count_by_stage(&self) -> Result<Vec<(String, usize)>, DatabaseError> {
        let conn = self.conn.lock().map_err(|_| DatabaseError::NotInitialized)?;
        let mut stmt = conn.prepare(
            "SELECT stage, COUNT(*) as cnt FROM documents GROUP BY stage ORDER BY stage",
        )?;
        let results = stmt
            .query_map([], |row| {
                let stage: String = row.get(0)?;
                let cnt: usize = row.get(1)?;
                Ok((stage, cnt))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(results)
    }

    async fn count_by_nature(&self) -> Result<Vec<(String, usize)>, DatabaseError> {
        let conn = self.conn.lock().map_err(|_| DatabaseError::NotInitialized)?;
        let mut stmt = conn.prepare(
            "SELECT nature, COUNT(*) as cnt FROM documents WHERE nature != '' GROUP BY nature ORDER BY cnt DESC",
        )?;
        let counts: Vec<(String, usize)> = stmt
            .query_map([], |row| {
                let nature: String = row.get(0)?;
                let cnt: usize = row.get(1)?;
                Ok((nature, cnt))
            })?
            .filter_map(|r| r.ok())
            .collect();
        Ok(counts)
    }

    async fn get_documents_by_status(&self, status: &DocStatus) -> Result<Vec<Document>, DatabaseError> {
        let status_str = status.as_str();
        let conn = self.conn.lock().map_err(|_| DatabaseError::NotInitialized)?;
        let mut stmt = conn.prepare(
            "SELECT id, stage, title, upstream, frontmatter_json, content, status, indexed_at, nature
             FROM documents WHERE status = ?1",
        )?;

        let docs = stmt
            .query_map(params![status_str], |row| Ok(row_to_document(row)))?
            .filter_map(|r| r.ok())
            .filter_map(|r| r.ok())
            .collect();

        Ok(docs)
    }

    async fn get_all_documents(&self) -> Result<Vec<Document>, DatabaseError> {
        let conn = self.conn.lock().map_err(|_| DatabaseError::NotInitialized)?;
        let mut stmt = conn.prepare(
            "SELECT id, stage, title, upstream, frontmatter_json, content, status, indexed_at, nature
             FROM documents ORDER BY nature, stage DESC",
        )?;

        let docs = stmt
            .query_map([], |row| Ok(row_to_document(row)))?
            .filter_map(|r| r.ok())
            .filter_map(|r| r.ok())
            .collect();

        Ok(docs)
    }
}

fn row_to_document(
    row: &rusqlite::Row<'_>,
) -> Result<Document, DatabaseError> {
    let id: String = row.get(0)?;
    let stage_str: String = row.get(1)?;
    let title: String = row.get(2)?;
    let upstream: Option<String> = row.get(3)?;
    let frontmatter_json: String = row.get(4)?;
    let content: String = row.get(5)?;
    let status_str: String = row.get(6)?;
    let indexed_at_str: String = row.get(7)?;
    let nature: String = row.get(8)?;

    let frontmatter: super::models::Frontmatter = serde_json::from_str(&frontmatter_json)?;
    let indexed_at = indexed_at_str.parse::<chrono::DateTime<chrono::Utc>>().unwrap_or_else(|_| chrono::Utc::now());

    Ok(Document {
        id,
        stage: Stage(stage_str),
        title,
        upstream,
        frontmatter,
        content,
        status: DocStatus::from_str(&status_str).unwrap_or(DocStatus::Error),
        indexed_at,
        nature,
    })
}

/// 从内容中提取搜索片段（字符安全）
fn extract_snippet(content: &str, query: &str, max_chars: usize) -> String {
    let query_lower = query.to_lowercase();
    let content_lower = content.to_lowercase();
    if let Some(pos) = content_lower.find(&query_lower) {
        // 将字节位置转为字符索引
        let char_pos = content[..pos].chars().count();
        let chars: Vec<char> = content.chars().collect();
        let start = char_pos.saturating_sub(max_chars / 2);
        let end = (char_pos + query.chars().count() + max_chars / 2).min(chars.len());
        let mut snippet = String::new();
        if start > 0 {
            snippet.push_str("...");
        }
        snippet.extend(&chars[start..end]);
        if end < chars.len() {
            snippet.push_str("...");
        }
        snippet
    } else {
        content.chars().take(max_chars).collect()
    }
}
