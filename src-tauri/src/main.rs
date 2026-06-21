#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod db;
mod vector;

use serde::{Deserialize, Serialize};
use tauri::State;

#[cfg(target_os = "macos")]
use std::process::{Command, Output, Stdio};
#[cfg(target_os = "macos")]
use std::thread;
#[cfg(target_os = "macos")]
use std::time::{Duration, Instant};

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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppleReminderItemDto {
    habit_name: String,
    scheduled_time: String,
}

#[cfg(target_os = "macos")]
fn escape_applescript_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(target_os = "macos")]
fn run_osascript_with_timeout(script: &str, timeout: Duration) -> Result<Output, String> {
    let mut child = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Failed to launch osascript: {error}"))?;

    let started = Instant::now();
    loop {
        match child
            .try_wait()
            .map_err(|error| format!("Failed to monitor osascript process: {error}"))?
        {
            Some(_) => {
                return child
                    .wait_with_output()
                    .map_err(|error| format!("Failed to collect osascript output: {error}"));
            }
            None => {
                if started.elapsed() >= timeout {
                    child
                        .kill()
                        .map_err(|error| format!("Failed to terminate osascript process: {error}"))?;
                    let _ = child.wait_with_output();
                    return Err(format!(
                        "AppleScript execution timed out after {} seconds",
                        timeout.as_secs()
                    ));
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn create_apple_reminder(date: &str, habit_name: &str, scheduled_time: &str) -> Result<(), String> {
    let title = format!("Habit due: {} ({})", habit_name, scheduled_time);
    let body = format!("Scheduled for {} at {}", date, scheduled_time);
    let due_date_string = format!("{} {}:00", date, scheduled_time);

    let script = format!(
        r#"
with timeout of 15 seconds
set dueDateString to "{due_date_string}"
set dueDate to date (do shell script "date -j -f '%Y-%m-%d %H:%M:%S' " & quoted form of dueDateString & " '+%m/%d/%Y %H:%M:%S'")

tell application "Reminders"
    if not (exists list "Second Brain") then
        make new list with properties {{name:"Second Brain"}}
    end if
    set targetList to list "Second Brain"
    make new reminder at end of reminders of targetList with properties {{name:"{title}", body:"{body}", remind me date:dueDate}}
end tell
end timeout
"#,
        due_date_string = escape_applescript_string(&due_date_string),
        title = escape_applescript_string(&title),
        body = escape_applescript_string(&body)
    );

    let output = run_osascript_with_timeout(&script, Duration::from_secs(15))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
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

#[tauri::command]
async fn sync_habits_to_apple_reminders(
    date: String,
    items: Vec<AppleReminderItemDto>,
) -> Result<String, String> {
    if items.is_empty() {
        return Ok("No reminders were created.".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        let mut created = 0usize;

        for item in items {
            if item.scheduled_time.len() != 5 {
                continue;
            }

            create_apple_reminder(&date, &item.habit_name, &item.scheduled_time)?;
            created += 1;
        }

        return Ok(format!(
            "Created {} reminder(s) in Apple Reminders list 'Second Brain'.",
            created
        ));
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (date, items);
        Err("Apple Reminders sync is only available on macOS.".to_string())
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let db_pool = match db::init_pool().await {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!(
                "Failed to initialize database pool at startup: {error}. Starting with lazy pool."
            );
            db::init_pool_lazy().expect("Failed to initialize lazy database pool")
        }
    };

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
            set_habit_occurrence_completed,
            sync_habits_to_apple_reminders
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
