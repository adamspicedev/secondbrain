#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod db;
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

#[derive(Serialize, Deserialize, Debug)]
pub struct DocumentDetail {
    id: String,
    title: String,
    content: String,
}

#[tauri::command]
async fn search(
    query: String,
    state: State<'_, db::DbPool>,
) -> Result<Vec<SearchResult>, String> {
    let results = db::keyword_search(&state, &query, 50).await?;

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
async fn create_document(
    title: String,
    content: String,
    state: State<'_, db::DbPool>,
) -> Result<DocumentDetail, String> {
    let doc = db::create_document(&state, &title, &content).await?;
    Ok(DocumentDetail {
        id: doc.id,
        title: doc.title,
        content: doc.content,
    })
}

#[tauri::command]
async fn get_document_detail(
    id: String,
    state: State<'_, db::DbPool>,
) -> Result<DocumentDetail, String> {
    let doc = db::get_document_detail(&state, &id).await?;
    Ok(DocumentDetail {
        id: doc.id,
        title: doc.title,
        content: doc.content,
    })
}

#[tauri::command]
async fn update_document(
    id: String,
    title: String,
    content: String,
    state: State<'_, db::DbPool>,
) -> Result<(), String> {
    db::update_document(&state, &id, &title, &content).await
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let db_pool = db::init_pool()
        .await
        .expect("Failed to initialize database pool");

    tauri::Builder::default()
        .manage(db_pool)
        .invoke_handler(tauri::generate_handler![
            search,
            create_document,
            get_document_detail,
            update_document
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
