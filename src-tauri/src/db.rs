use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres, Row};
use uuid::Uuid;

pub type DbPool = Pool<Postgres>;

#[derive(Debug, Clone)]
pub struct DocumentRecord {
    pub id: String,
    pub filename: String,
    #[allow(dead_code)]
    pub extracted_text: String,
    pub similarity: f32,
    pub content_preview: String,
}

#[derive(Debug, Clone)]
pub struct DocumentDetail {
    pub id: String,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Habit {
    pub id: String,
    pub name: String,
    pub times_per_day: i32,
    pub days_of_week: Vec<i32>,
    pub times_of_day: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HabitOccurrence {
    pub occurrence_id: String,
    pub habit_id: String,
    pub habit_name: String,
    pub scheduled_date: String,
    pub scheduled_time: String,
    pub completed: bool,
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

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS habits (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name TEXT NOT NULL,
            times_per_day INTEGER NOT NULL CHECK (times_per_day > 0),
            days_of_week INTEGER[] NOT NULL,
            times_of_day TIME[] NOT NULL,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at TIMESTAMP NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMP NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS habit_occurrences (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            habit_id UUID NOT NULL REFERENCES habits(id) ON DELETE CASCADE,
            scheduled_date DATE NOT NULL,
            scheduled_time TIME NOT NULL,
            completed_at TIMESTAMP NULL,
            created_at TIMESTAMP NOT NULL DEFAULT NOW(),
            UNIQUE (habit_id, scheduled_date, scheduled_time)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_habit_occurrences_date
        ON habit_occurrences (scheduled_date, scheduled_time)
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

pub async fn keyword_search(
    pool: &DbPool,
    query: &str,
    limit: i64,
) -> Result<Vec<DocumentRecord>, String> {
    let like_query = format!("%{}%", query);
    let rows = if query.trim().is_empty() {
        sqlx::query(
            r#"
            SELECT id::text AS id, filename, extracted_text
            FROM documents
            ORDER BY updated_at DESC, created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query(
            r#"
            SELECT id::text AS id, filename, extracted_text
            FROM documents
            WHERE filename ILIKE $1 OR extracted_text ILIKE $1
            ORDER BY updated_at DESC, created_at DESC
            LIMIT $2
            "#,
        )
        .bind(like_query)
        .bind(limit)
        .fetch_all(pool)
        .await
    }
    .map_err(|e| format!("Search failed: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let text: String = row.get("extracted_text");
            let content_preview = if text.len() > 200 {
                format!("{}...", &text[..200])
            } else {
                text.clone()
            };

            DocumentRecord {
                id: row.get("id"),
                filename: row.get("filename"),
                extracted_text: text,
                similarity: 1.0,
                content_preview,
            }
        })
        .collect())
}

pub async fn create_document(
    pool: &DbPool,
    title: &str,
    content: &str,
) -> Result<DocumentDetail, String> {
    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO documents (id, filename, file_type, extracted_text, embedding)
        VALUES ($1, $2, $3, $4, NULL)
        "#,
    )
    .bind(id)
    .bind(title)
    .bind("note")
    .bind(content)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create document: {}", e))?;

    Ok(DocumentDetail {
        id: id.to_string(),
        title: title.to_string(),
        content: content.to_string(),
    })
}

pub async fn update_document(
    pool: &DbPool,
    id: &str,
    title: &str,
    content: &str,
) -> Result<(), String> {
    let rows_affected = sqlx::query(
        r#"
        UPDATE documents
        SET filename = $1,
            extracted_text = $2,
            embedding = NULL,
            updated_at = NOW()
        WHERE id = $3::uuid
        "#,
    )
    .bind(title)
    .bind(content)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update document: {}", e))?
    .rows_affected();

    if rows_affected == 0 {
        return Err("Document not found".to_string());
    }

    Ok(())
}

pub async fn get_document_detail(
    pool: &DbPool,
    id: &str,
) -> Result<DocumentDetail, String> {
    let row = sqlx::query(
        r#"
        SELECT id::text AS id, filename, extracted_text
        FROM documents
        WHERE id = $1::uuid
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Failed to fetch document: {}", e))?
    .ok_or_else(|| "Document not found".to_string())?;

    Ok(DocumentDetail {
        id: row.get("id"),
        title: row.get("filename"),
        content: row.get("extracted_text"),
    })
}

pub async fn create_habit(
    pool: &DbPool,
    name: &str,
    times_per_day: i32,
    days_of_week: &[i32],
    times_of_day: &[String],
) -> Result<Habit, String> {
    if times_per_day <= 0 {
        return Err("times_per_day must be greater than 0".to_string());
    }

    if days_of_week.is_empty() {
        return Err("At least one day of week is required".to_string());
    }

    if times_of_day.is_empty() {
        return Err("At least one time of day is required".to_string());
    }

    if times_of_day.len() as i32 != times_per_day {
        return Err("times_per_day must match the number of times_of_day entries".to_string());
    }

    let id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO habits (id, name, times_per_day, days_of_week, times_of_day)
        VALUES ($1, $2, $3, $4, $5::time[])
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(times_per_day)
    .bind(days_of_week)
    .bind(times_of_day)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create habit: {}", e))?;

    Ok(Habit {
        id: id.to_string(),
        name: name.to_string(),
        times_per_day,
        days_of_week: days_of_week.to_vec(),
        times_of_day: times_of_day.to_vec(),
    })
}

pub async fn list_habits(pool: &DbPool) -> Result<Vec<Habit>, String> {
    let rows = sqlx::query(
        r#"
        SELECT id::text AS id, name, times_per_day,
               days_of_week,
               ARRAY(SELECT to_char(t, 'HH24:MI') FROM unnest(times_of_day) AS t) AS times_of_day
        FROM habits
        WHERE is_active = TRUE
        ORDER BY name ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to list habits: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|row| Habit {
            id: row.get("id"),
            name: row.get("name"),
            times_per_day: row.get("times_per_day"),
            days_of_week: row.get("days_of_week"),
            times_of_day: row.get("times_of_day"),
        })
        .collect())
}

pub async fn list_habit_occurrences_for_date(
    pool: &DbPool,
    date: &str,
) -> Result<Vec<HabitOccurrence>, String> {
    let rows = sqlx::query(
        r#"
        WITH target_day AS (
            SELECT EXTRACT(DOW FROM $1::date)::int AS dow
        ),
        scheduled AS (
            SELECT h.id::text AS habit_id,
                   h.name AS habit_name,
                   $1::date AS scheduled_date,
                   to_char(t, 'HH24:MI') AS scheduled_time
            FROM habits h
            JOIN target_day td ON td.dow = ANY(h.days_of_week)
            CROSS JOIN LATERAL unnest(h.times_of_day) AS t
            WHERE h.is_active = TRUE
        )
        SELECT 
            COALESCE(o.id::text, gen_random_uuid()::text) AS occurrence_id,
            s.habit_id,
            s.habit_name,
            s.scheduled_date::text AS scheduled_date,
            s.scheduled_time,
            (o.completed_at IS NOT NULL) AS completed
        FROM scheduled s
        LEFT JOIN habit_occurrences o
          ON o.habit_id = s.habit_id::uuid
         AND o.scheduled_date = s.scheduled_date
         AND to_char(o.scheduled_time, 'HH24:MI') = s.scheduled_time
        ORDER BY s.scheduled_time ASC, s.habit_name ASC
        "#,
    )
    .bind(date)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to load habit occurrences: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|row| HabitOccurrence {
            occurrence_id: row.get("occurrence_id"),
            habit_id: row.get("habit_id"),
            habit_name: row.get("habit_name"),
            scheduled_date: row.get("scheduled_date"),
            scheduled_time: row.get("scheduled_time"),
            completed: row.get("completed"),
        })
        .collect())
}

pub async fn list_habit_occurrences_for_range(
    pool: &DbPool,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<HabitOccurrence>, String> {
    let rows = sqlx::query(
        r#"
        WITH dates AS (
            SELECT generate_series($1::date, $2::date, interval '1 day')::date AS scheduled_date
        ),
        scheduled AS (
            SELECT
                h.id::text AS habit_id,
                h.name AS habit_name,
                d.scheduled_date,
                to_char(t, 'HH24:MI') AS scheduled_time
            FROM habits h
            JOIN dates d
              ON EXTRACT(DOW FROM d.scheduled_date)::int = ANY(h.days_of_week)
            CROSS JOIN LATERAL unnest(h.times_of_day) AS t
            WHERE h.is_active = TRUE
        )
        SELECT
            COALESCE(o.id::text, gen_random_uuid()::text) AS occurrence_id,
            s.habit_id,
            s.habit_name,
            s.scheduled_date::text AS scheduled_date,
            s.scheduled_time,
            (o.completed_at IS NOT NULL) AS completed
        FROM scheduled s
        LEFT JOIN habit_occurrences o
          ON o.habit_id = s.habit_id::uuid
         AND o.scheduled_date = s.scheduled_date
         AND to_char(o.scheduled_time, 'HH24:MI') = s.scheduled_time
        ORDER BY s.scheduled_date ASC, s.scheduled_time ASC, s.habit_name ASC
        "#,
    )
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to load habit occurrences for range: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|row| HabitOccurrence {
            occurrence_id: row.get("occurrence_id"),
            habit_id: row.get("habit_id"),
            habit_name: row.get("habit_name"),
            scheduled_date: row.get("scheduled_date"),
            scheduled_time: row.get("scheduled_time"),
            completed: row.get("completed"),
        })
        .collect())
}

pub async fn update_habit(
    pool: &DbPool,
    id: &str,
    name: &str,
    times_per_day: i32,
    days_of_week: &[i32],
    times_of_day: &[String],
) -> Result<Habit, String> {
    if name.trim().is_empty() {
        return Err("Habit name is required".to_string());
    }

    if times_per_day <= 0 {
        return Err("times_per_day must be greater than 0".to_string());
    }

    if days_of_week.is_empty() {
        return Err("At least one day of week is required".to_string());
    }

    if times_of_day.is_empty() {
        return Err("At least one time of day is required".to_string());
    }

    if times_of_day.len() as i32 != times_per_day {
        return Err("times_per_day must match the number of times_of_day entries".to_string());
    }

    let rows_affected = sqlx::query(
        r#"
        UPDATE habits
        SET name = $1,
            times_per_day = $2,
            days_of_week = $3,
            times_of_day = $4::time[],
            updated_at = NOW()
        WHERE id = $5::uuid
          AND is_active = TRUE
        "#,
    )
    .bind(name.trim())
    .bind(times_per_day)
    .bind(days_of_week)
    .bind(times_of_day)
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update habit: {}", e))?
    .rows_affected();

    if rows_affected == 0 {
        return Err("Habit not found".to_string());
    }

    Ok(Habit {
        id: id.to_string(),
        name: name.trim().to_string(),
        times_per_day,
        days_of_week: days_of_week.to_vec(),
        times_of_day: times_of_day.to_vec(),
    })
}

pub async fn delete_habit(
    pool: &DbPool,
    id: &str,
) -> Result<(), String> {
    let rows_affected = sqlx::query(
        r#"
        UPDATE habits
        SET is_active = FALSE,
            updated_at = NOW()
        WHERE id = $1::uuid
          AND is_active = TRUE
        "#,
    )
    .bind(id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to delete habit: {}", e))?
    .rows_affected();

    if rows_affected == 0 {
        return Err("Habit not found".to_string());
    }

    Ok(())
}

pub async fn set_habit_occurrence_completed(
    pool: &DbPool,
    habit_id: &str,
    scheduled_date: &str,
    scheduled_time: &str,
    completed: bool,
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO habit_occurrences (habit_id, scheduled_date, scheduled_time, completed_at)
        VALUES (
            $1::uuid,
            $2::date,
            $3::time,
            CASE WHEN $4 THEN NOW() ELSE NULL END
        )
        ON CONFLICT (habit_id, scheduled_date, scheduled_time)
        DO UPDATE SET completed_at = CASE WHEN $4 THEN NOW() ELSE NULL END
        "#,
    )
    .bind(habit_id)
    .bind(scheduled_date)
    .bind(scheduled_time)
    .bind(completed)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update habit completion: {}", e))?;

    Ok(())
}
