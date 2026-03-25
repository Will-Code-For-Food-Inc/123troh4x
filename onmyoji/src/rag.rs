use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, SearchPointsBuilder,
    UpsertPointsBuilder, VectorParamsBuilder,
};
use qdrant_client::Qdrant;
use uuid::Uuid;

const VECTOR_DIM: u64 = 384; // AllMiniLML6V2
const CHUNK_CHARS: usize = 600;

// ── RagStore ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct RagStore {
    client: Arc<Qdrant>,
    model: Arc<Mutex<TextEmbedding>>,
}

impl RagStore {
    pub fn new(qdrant_url: &str) -> Result<Self> {
        let client = Qdrant::from_url(qdrant_url)
            .build()
            .context("failed to connect to Qdrant")?;
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2),
        )
        .context("failed to load fastembed model")?;
        Ok(Self {
            client: Arc::new(client),
            model: Arc::new(Mutex::new(model)),
        })
    }

    async fn ensure_collection(&self, collection: &str) -> Result<()> {
        if !self.client.collection_exists(collection).await? {
            self.client
                .create_collection(
                    CreateCollectionBuilder::new(collection)
                        .vectors_config(VectorParamsBuilder::new(VECTOR_DIM, Distance::Cosine)),
                )
                .await
                .with_context(|| format!("failed to create collection {collection}"))?;
        }
        Ok(())
    }

    /// Ingest text into `collection`. Returns the number of chunks stored.
    pub async fn ingest(
        &self,
        collection: &str,
        text: &str,
        source: &str,
        platform: Option<&str>,
    ) -> Result<usize> {
        self.ensure_collection(collection).await?;

        let chunks = chunk_text(text, CHUNK_CHARS);
        if chunks.is_empty() {
            return Ok(0);
        }
        let n = chunks.len();

        let model = self.model.clone();
        let chunks_clone = chunks.clone();
        let embeddings = tokio::task::spawn_blocking(move || {
            model.lock().unwrap().embed(chunks_clone, None)
        })
        .await
        .context("embed task panicked")?
        .context("embedding failed")?;

        let points: Vec<PointStruct> = embeddings
            .into_iter()
            .zip(chunks)
            .map(|(vec, chunk)| {
                let mut payload = serde_json::json!({
                    "text":   chunk,
                    "source": source,
                });
                if let Some(p) = platform {
                    payload["platform"] = serde_json::json!(p);
                }
                PointStruct::new(
                    Uuid::new_v4().to_string(),
                    vec,
                    payload
                        .as_object()
                        .unwrap()
                        .iter()
                        .map(|(k, v)| (k.clone(), qdrant_value(v)))
                        .collect::<std::collections::HashMap<_, _>>(),
                )
            })
            .collect();

        self.client
            .upsert_points(UpsertPointsBuilder::new(collection, points))
            .await
            .context("upsert failed")?;

        Ok(n)
    }

    /// Semantic search over `collection`. Returns up to `limit` results.
    pub async fn search(
        &self,
        collection: &str,
        query: &str,
        limit: u64,
    ) -> Result<Vec<SearchHit>> {
        let model = self.model.clone();
        let query_owned = query.to_string();
        let vec = tokio::task::spawn_blocking(move || {
            model
                .lock()
                .unwrap()
                .embed(vec![query_owned], None)
                .map(|mut v| v.pop().unwrap_or_default())
        })
        .await
        .context("embed task panicked")?
        .context("query embedding failed")?;

        let resp = self
            .client
            .search_points(
                SearchPointsBuilder::new(collection, vec, limit).with_payload(true),
            )
            .await
            .context("search failed")?;

        Ok(resp
            .result
            .into_iter()
            .map(|r| {
                let text = r
                    .payload
                    .get("text")
                    .and_then(payload_str)
                    .unwrap_or_default()
                    .to_string();
                let source = r
                    .payload
                    .get("source")
                    .and_then(payload_str)
                    .unwrap_or_default()
                    .to_string();
                SearchHit { score: r.score, text, source }
            })
            .collect())
    }

    pub async fn list_collections(&self) -> Result<Vec<String>> {
        let resp = self.client.list_collections().await?;
        Ok(resp.collections.into_iter().map(|c| c.name).collect())
    }
}

pub struct SearchHit {
    pub score: f32,
    pub text: String,
    pub source: String,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    for para in text.split("\n\n") {
        let para = para.trim();
        if para.is_empty() {
            continue;
        }
        if current.len() + para.len() + 2 > max_chars && !current.is_empty() {
            chunks.push(current.trim().to_string());
            current = String::new();
        }
        if para.len() > max_chars {
            // Hard-split oversized paragraphs at char boundaries.
            let mut i = 0;
            let chars: Vec<char> = para.chars().collect();
            while i < chars.len() {
                let end = (i + max_chars).min(chars.len());
                chunks.push(chars[i..end].iter().collect());
                i = end;
            }
        } else {
            if !current.is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(para);
        }
    }
    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }
    if chunks.is_empty() && !text.trim().is_empty() {
        chunks.push(text.trim().to_string());
    }
    chunks
}

fn qdrant_value(v: &serde_json::Value) -> qdrant_client::qdrant::Value {
    use qdrant_client::qdrant::value::Kind;
    use qdrant_client::qdrant::Value;
    let kind = match v {
        serde_json::Value::String(s) => Some(Kind::StringValue(s.clone())),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(Kind::IntegerValue(i))
            } else {
                Some(Kind::DoubleValue(n.as_f64().unwrap_or(0.0)))
            }
        }
        serde_json::Value::Bool(b) => Some(Kind::BoolValue(*b)),
        _ => Some(Kind::StringValue(v.to_string())),
    };
    Value { kind }
}

fn payload_str(v: &qdrant_client::qdrant::Value) -> Option<&str> {
    use qdrant_client::qdrant::value::Kind;
    match v.kind.as_ref()? {
        Kind::StringValue(s) => Some(s.as_str()),
        _ => None,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_empty_string() {
        assert!(chunk_text("", 100).is_empty());
    }

    #[test]
    fn chunk_short_text_is_single_chunk() {
        let chunks = chunk_text("hello world", 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "hello world");
    }

    #[test]
    fn chunk_splits_on_double_newline() {
        let text = "para one\n\npara two\n\npara three";
        let chunks = chunk_text(text, 100);
        // All fit in one chunk (total < 100 chars).
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn chunk_splits_when_over_limit() {
        let text = "aaaa\n\nbbbb\n\ncccc";
        // Each para is 4 chars; limit 6 — "aaaa" fits, "bbbb" would overflow current.
        let chunks = chunk_text(text, 6);
        assert!(chunks.len() > 1);
    }

    #[test]
    fn chunk_hard_splits_long_para() {
        let long = "x".repeat(1500);
        let chunks = chunk_text(&long, 600);
        assert_eq!(chunks.len(), 3); // 600 + 600 + 300
        for c in &chunks[..2] {
            assert_eq!(c.len(), 600);
        }
    }
}
