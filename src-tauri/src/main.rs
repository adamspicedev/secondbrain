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

#[derive(Serialize, Deserialize, Debug)]
pub struct HabitDto {
    id: String,
    name: String,
    times_per_day: i32,
    days_of_week: Vec<i32>,
    times_of_day: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HabitOccurrenceDto {
    occurrence_id: String,
    habit_id: String,
    habit_name: String,
    scheduled_date: String,
    scheduled_time: String,
    completed: bool,
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

#[tauri::command]
async fn create_habit(
    name: String,
    times_per_day: i32,
    days_of_week: Vec<i32>,
    times_of_day: Vec<String>,
    state: State<'_, db::DbPool>,
) -> Result<HabitDto, String> {
    let habit = db::create_habit(&state, &name, times_per_day, &days_of_week, &times_of_day).await?;
    Ok(HabitDto {
        id: habit.id,
        name: habit.name,
        times_per_day: habit.times_per_day,
        days_of_week: habit.days_of_week,
        times_of_day: habit.times_of_day,
    })
}

#[tauri::command]
async fn list_habits(state: State<'_, db::DbPool>) -> Result<Vec<HabitDto>, String> {
    let habits = db::list_habits(&state).await?;
    Ok(habits
        .into_iter()
        .map(|h| HabitDto {
            id: h.id,
            name: h.name,
            times_per_day: h.times_per_day,
            days_of_week: h.days_of_week,
            times_of_day: h.times_of_day,
        })
        .collect())
}

#[tauri::command]
async fn list_habit_occurrences_for_date(
    date: String,
    state: State<'_, db::DbPool>,
) -> Result<Vec<HabitOccurrenceDto>, String> {
    let items = db::list_habit_occurrences_for_date(&state, &date).await?;
    Ok(items
        .into_iter()
        .map(|o| HabitOccurrenceDto {
            occurrence_id: o.occurrence_id,
            habit_id: o.habit_id,
            habit_name: o.habit_name,
            scheduled_date: o.scheduled_date,
            scheduled_time: o.scheduled_time,
            completed: o.completed,
        })
        .collect())
}

#[tauri::command]
async fn list_habit_occurrences_for_range(
    start_date: String,
    end_date: String,
    state: State<'_, db::DbPool>,
) -> Result<Vec<HabitOccurrenceDto>, String> {
    let items = db::list_habit_occurrences_for_range(&state, &start_date, &end_date).await?;
    Ok(items
        .into_iter()
        .map(|o| HabitOccurrenceDto {
            occurrence_id: o.occurrence_id,
            habit_id: o.habit_id,
            habit_name: o.habit_name,
            scheduled_date: o.scheduled_date,
            scheduled_time: o.scheduled_time,
            completed: o.completed,
        })
        .collect())
}

#[tauri::command]
async fn update_habit(
    id: String,
    name: String,
    times_per_day: i32,
    days_of_week: Vec<i32>,
    times_of_day: Vec<String>,
    state: State<'_, db::DbPool>,
) -> Result<HabitDto, String> {
    let habit = db::update_habit(&state, &id, &name, times_per_day, &days_of_week, &times_of_day).await?;
    Ok(HabitDto {
        id: habit.id,
        name: habit.name,
        times_per_day: habit.times_per_day,
        days_of_week: habit.days_of_week,
        times_of_day: habit.times_of_day,
    })
}

#[tauri::command]
async fn delete_habit(
    id: String,
    state: State<'_, db::DbPool>,
) -> Result<(), String> {
    db::delete_habit(&state, &id).await
}

#[tauri::command]
async fn set_habit_occurrence_completed(
    habit_id: String,
    scheduled_date: String,
    scheduled_time: String,
    completed: bool,
    state: State<'_, db::DbPool>,
) -> Result<(), String> {
    db::set_habit_occurrence_completed(
        &state,
        &habit_id,
        &scheduled_date,
        &scheduled_time,
        completed,
    )
    .await
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
            update_document,
            create_habit,
            update_habit,
            delete_habit,
            list_habits,
            list_habit_occurrences_for_date,
            list_habit_occurrences_for_range,
            set_habit_occurrence_completed
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
