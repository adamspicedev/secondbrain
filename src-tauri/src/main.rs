#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod db;
mod ai;
mod vector;

use tauri::State;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchResult {
    id: String,
    filename: String,
    content_preview: String,
    similarity: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UploadResponse {
    id: String,
    filename: String,
    extracted_text: String,
}

#[tauri::command]
async fn upload_file(
    file_path: String,
    file_type: String,
    state: State<'_, db::DbPool>,
) -> Result<UploadResponse, String> {
    println!("Uploading file: {} ({})", file_path, file_type);

    // Extract text from file
    let extracted_text = match file_type.as_str() {
        "image" => ai::extract_text_from_image(&file_path).await?,
        "pdf" => ai::extract_text_from_pdf(&file_path).await?,
        "document" => ai::extract_text_from_document(&file_path).await?,
        _ => return Err("Unsupported file type".to_string()),
    };

    // Generate embedding
    let embedding = ai::generate_embedding(&extracted_text).await?;

    // Store in database
    let result = db::store_document(&state, &file_path, &extracted_text, &embedding).await?;

    Ok(UploadResponse {
        id: result.id,
        filename: result.filename,
        extracted_text: result.extracted_text,
    })
}

#[tauri::command]
async fn search(
    query: String,
    state: State<'_, db::DbPool>,
) -> Result<Vec<SearchResult>, String> {
    println!("Searching for: {}", query);

    // Generate embedding for search query
    let query_embedding = ai::generate_embedding(&query).await?;

    // Vector search in database
    let results = db::vector_search(&state, &query_embedding, 10).await?;

    Ok(results
        .into_iter()
        .map(|r| SearchResult {
            id: r.id,
            filename: r.filename,
            content_preview: r.content_preview,
            similarity: r.similarity,
        })
        .collect())
}

#[tauri::command]
async fn get_document(
    id: String,
    state: State<'_, db::DbPool>,
) -> Result<String, String> {
    db::get_document_content(&state, &id).await
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let db_pool = db::init_pool()
        .await
        .expect("Failed to initialize database pool");

    tauri::Builder::default()
        .manage(db_pool)
        .invoke_handler(tauri::generate_handler![upload_file, search, get_document])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
