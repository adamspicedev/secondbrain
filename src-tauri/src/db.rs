use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

pub type DbPool = Pool<Postgres>;

#[derive(Debug, Clone)]
pub struct DocumentRecord {
    pub id: String,
    pub filename: String,
    pub extracted_text: String,
    pub similarity: f32,
    pub content_preview: String,
}

pub async fn init_pool() -> Result<DbPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            "postgres://secondbrain_user:changeme_securepassword@pi.local:5432/secondbrain"
                .to_string()
        });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Ensure pgvector is available before creating columns of type `vector`.
    sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
        .execute(&pool)
        .await?;

    // Run migrations (execute statements separately; Postgres prepared statements
    // cannot contain multiple commands in one query call).
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS documents (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            filename TEXT NOT NULL,
            file_type TEXT NOT NULL,
            extracted_text TEXT NOT NULL,
            embedding vector(1536),
            created_at TIMESTAMP DEFAULT NOW(),
            updated_at TIMESTAMP DEFAULT NOW()
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_embedding
        ON documents USING ivfflat (embedding vector_cosine_ops)
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

pub async fn store_document(
    pool: &DbPool,
    filename: &str,
    extracted_text: &str,
    embedding: &[f32; 1536],
) -> Result<DocumentRecord, String> {
    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO documents (id, filename, file_type, extracted_text, embedding)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(id)
    .bind(filename)
    .bind("mixed")
    .bind(extracted_text)
    .bind(embedding as &[f32])
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to store document: {}", e))?;

    let content_preview = if extracted_text.len() > 200 {
        format!("{}...", &extracted_text[..200])
    } else {
        extracted_text.to_string()
    };

    Ok(DocumentRecord {
        id: id.to_string(),
        filename: filename.to_string(),
        extracted_text: extracted_text.to_string(),
        similarity: 1.0,
        content_preview,
    })
}

pub async fn vector_search(
    pool: &DbPool,
    embedding: &[f32; 1536],
    limit: i64,
) -> Result<Vec<DocumentRecord>, String> {
    let results = sqlx::query(
        r#"
        SELECT id, filename, extracted_text, 
               1 - (embedding <=> $1) as similarity
        FROM documents
        ORDER BY similarity DESC
        LIMIT $2
        "#,
    )
    .bind(embedding as &[f32])
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Vector search failed: {}", e))?;

    Ok(results
        .into_iter()
        .map(|row| {
            let text: String = row.get("extracted_text");
            let content_preview = if text.len() > 200 {
                format!("{}...", &text[..200])
            } else {
                text.clone()
            };

            DocumentRecord {
                id: row.get::<String, _>("id"),
                filename: row.get("filename"),
                extracted_text: text,
                similarity: row.get("similarity"),
                content_preview,
            }
        })
        .collect())
}

pub async fn get_document_content(
    pool: &DbPool,
    id: &str,
) -> Result<String, String> {
    let result = sqlx::query_scalar::<_, String>(
        "SELECT extracted_text FROM documents WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to fetch document: {}", e))?
    .ok_or_else(|| "Document not found".to_string())?;

    Ok(result)
}
