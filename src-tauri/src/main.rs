#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod db;
mod vector;

use serde::{Deserialize, Serialize};
use tauri::State;
use std::collections::HashMap;

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
    #[serde(default)]
    completed: bool,
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
fn sync_apple_reminder_item(
    date: &str,
    habit_name: &str,
    scheduled_time: &str,
    completed: bool,
) -> Result<(bool, bool), String> {
    const APPLESCRIPT_TIMEOUT_SECONDS: u64 = 120;

    let title = format!("Habit due: {} ({})", habit_name, scheduled_time);
    let body = format!("Scheduled for {} at {}", date, scheduled_time);
    let due_date_string = format!("{} {}:00", date, scheduled_time);
    let completed_literal = if completed { "true" } else { "false" };

    let script = format!(
        r#"
with timeout of {apple_timeout} seconds
set dueDateString to "{due_date_string}"
set dueEpoch to (do shell script "date -j -f '%Y-%m-%d %H:%M:%S' " & quoted form of dueDateString & " '+%s'") as integer
set nowEpoch to (do shell script "date '+%s'") as integer
set dueDate to (current date) + (dueEpoch - nowEpoch)
set targetCompleted to {completed_literal}

tell application "Reminders"
    if not (exists list "Second Brain") then
        make new list with properties {{name:"Second Brain"}}
    end if
    set targetList to list "Second Brain"
    set existingReminders to (every reminder of targetList whose name is "{title}" and body is "{body}")
    set createdCount to 0
    set completedChangedCount to 0
    if (count of existingReminders) is 0 then
        if targetCompleted is false then
            make new reminder at end of reminders of targetList with properties {{name:"{title}", body:"{body}", remind me date:dueDate, completed:false}}
            set createdCount to 1
        end if
    else
        repeat with r in existingReminders
            if completed of r is not targetCompleted then
                set completed of r to targetCompleted
                set completedChangedCount to completedChangedCount + 1
            end if
            if targetCompleted is false then
                set remind me date of r to dueDate
            end if
        end repeat
    end if
    return (createdCount as text) & "," & (completedChangedCount as text)
end tell
end timeout
"#,
        apple_timeout = APPLESCRIPT_TIMEOUT_SECONDS,
        due_date_string = escape_applescript_string(&due_date_string),
        completed_literal = completed_literal,
        title = escape_applescript_string(&title),
        body = escape_applescript_string(&body)
    );

    let output = match run_osascript_with_timeout(&script, Duration::from_secs(APPLESCRIPT_TIMEOUT_SECONDS)) {
        Ok(output) => output,
        Err(error) if error.contains("timed out") => {
            // Best effort: wake Reminders and retry once. First launch or permission prompts can stall.
            let _ = Command::new("open")
                .arg("-a")
                .arg("Reminders")
                .output();

            run_osascript_with_timeout(&script, Duration::from_secs(APPLESCRIPT_TIMEOUT_SECONDS)).map_err(
                |retry_error| {
                    format!(
                        "{retry_error}. If this persists, open Reminders and allow automation for Second Brain in System Settings > Privacy & Security > Automation."
                    )
                },
            )?
        }
        Err(error) => return Err(error),
    };

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let mut parts = stdout.split(',');
        let created = parts
            .next()
            .unwrap_or("0")
            .trim()
            .parse::<usize>()
            .unwrap_or(0);
        let completed_changed = parts
            .next()
            .unwrap_or("0")
            .trim()
            .parse::<usize>()
            .unwrap_or(0);
        Ok((created > 0, completed_changed > 0))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

#[cfg(target_os = "macos")]
fn remove_apple_reminders_not_in_schedule(
    date: &str,
    items: &[AppleReminderItemDto],
) -> Result<usize, String> {
    const APPLESCRIPT_TIMEOUT_SECONDS: u64 = 60;

    let allowed_bodies = items
        .iter()
        .map(|item| {
            format!(
                "\"{}\"",
                escape_applescript_string(&format!("Scheduled for {} at {}", date, item.scheduled_time))
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let allowed_bodies_list = if allowed_bodies.is_empty() {
        "{}".to_string()
    } else {
        format!("{{{allowed_bodies}}}")
    };

    let script = format!(
        r#"
with timeout of {apple_timeout} seconds
set targetDate to "{target_date}"
set targetBodyPrefix to "Scheduled for " & targetDate & " at "
set allowedBodies to {allowed_bodies}

tell application "Reminders"
    if not (exists list "Second Brain") then
        return "0"
    end if

    set targetList to list "Second Brain"
    set removedCount to 0
    set reminderCount to count of reminders of targetList

    repeat with idx from reminderCount to 1 by -1
        try
            set r to reminder idx of targetList
            set reminderBody to body of r
            if reminderBody starts with "Scheduled for " then
                if reminderBody does not contain targetBodyPrefix then
                    delete r
                    set removedCount to removedCount + 1
                else
                    if allowedBodies does not contain reminderBody then
                        delete r
                        set removedCount to removedCount + 1
                    end if
                end if
            end if
        on error
            -- Skip reminders that became unavailable while iterating.
        end try
    end repeat

    return removedCount as text
end tell
end timeout
"#,
        apple_timeout = APPLESCRIPT_TIMEOUT_SECONDS,
    target_date = escape_applescript_string(date),
    allowed_bodies = allowed_bodies_list
    );

    let output = run_osascript_with_timeout(&script, Duration::from_secs(APPLESCRIPT_TIMEOUT_SECONDS))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(stdout.parse::<usize>().unwrap_or(0))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

#[cfg(target_os = "macos")]
fn list_apple_reminders_for_date(date: &str) -> Result<Vec<AppleReminderItemDto>, String> {
    const APPLESCRIPT_TIMEOUT_SECONDS: u64 = 60;

    let script = format!(
        r#"
with timeout of {apple_timeout} seconds
set targetDate to "{target_date}"
set targetBodyPrefix to "Scheduled for " & targetDate & " at "

tell application "Reminders"
    if not (exists list "Second Brain") then
        return ""
    end if

    set targetList to list "Second Brain"
    set serializedRows to {{}}

    repeat with r in (every reminder of targetList)
        try
            set reminderBody to body of r
            if reminderBody starts with targetBodyPrefix then
                set reminderName to name of r
                set isDone to completed of r

                set habitName to reminderName
                if reminderName starts with "Habit due: " then
                    set remainderName to text 12 thru -1 of reminderName
                    if remainderName contains " (" then
                        set AppleScript's text item delimiters to " ("
                        set nameParts to text items of remainderName
                        set habitName to item 1 of nameParts
                        set AppleScript's text item delimiters to ""
                    else
                        set habitName to remainderName
                    end if
                end if

                set scheduledTime to text ((length of targetBodyPrefix) + 1) thru -1 of reminderBody
                set row to habitName & tab & scheduledTime & tab & (isDone as text)
                set end of serializedRows to row
            end if
        on error
            -- ignore malformed reminders
        end try
    end repeat

    set AppleScript's text item delimiters to linefeed
    set outputText to serializedRows as text
    set AppleScript's text item delimiters to ""
    return outputText
end tell
end timeout
"#,
        apple_timeout = APPLESCRIPT_TIMEOUT_SECONDS,
        target_date = escape_applescript_string(date)
    );

    let output = run_osascript_with_timeout(&script, Duration::from_secs(APPLESCRIPT_TIMEOUT_SECONDS))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut items = Vec::new();
    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let mut parts = line.split('\t');
        let habit_name = parts.next().unwrap_or("").trim().to_string();
        let scheduled_time = parts.next().unwrap_or("").trim().to_string();
        let completed_raw = parts.next().unwrap_or("false").trim().to_lowercase();

        if habit_name.is_empty() || scheduled_time.len() != 5 {
            continue;
        }

        let completed = matches!(completed_raw.as_str(), "true" | "yes");
        items.push(AppleReminderItemDto {
            habit_name,
            scheduled_time,
            completed,
        });
    }

    Ok(items)
}

#[cfg(target_os = "macos")]
async fn sync_apple_reminders_to_habits_internal(
    date: &str,
    state: &State<'_, db::DbPool>,
) -> Result<usize, String> {
    let reminders = list_apple_reminders_for_date(date)?;
    if reminders.is_empty() {
        return Ok(0);
    }

    let occurrences = db::list_habit_occurrences_for_date(state, date).await?;
    let mut by_name_time: HashMap<(String, String), Vec<db::HabitOccurrence>> = HashMap::new();
    for occurrence in occurrences {
        by_name_time
            .entry((occurrence.habit_name.clone(), occurrence.scheduled_time.clone()))
            .or_default()
            .push(occurrence);
    }

    let mut updated = 0usize;
    for reminder in reminders {
        if let Some(matching) = by_name_time.get(&(reminder.habit_name.clone(), reminder.scheduled_time.clone())) {
            for occurrence in matching {
                if occurrence.completed != reminder.completed {
                    db::set_habit_occurrence_completed(
                        state,
                        &occurrence.habit_id,
                        date,
                        &occurrence.scheduled_time,
                        reminder.completed,
                    )
                    .await?;
                    updated += 1;
                }
            }
        }
    }

    Ok(updated)
}

#[tauri::command]
async fn sync_apple_reminders_to_habits(
    date: String,
    state: State<'_, db::DbPool>,
) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let updated = sync_apple_reminders_to_habits_internal(&date, &state).await?;
        return Ok(format!("Synced {} reminder completion update(s) into habits.", updated));
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (date, state);
        Err("Apple Reminders sync is only available on macOS.".to_string())
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
    state: State<'_, db::DbPool>,
) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let pulled_updates = sync_apple_reminders_to_habits_internal(&date, &state).await?;
        let mut created = 0usize;
        let mut completed_updated = 0usize;
        let removed_obsolete = remove_apple_reminders_not_in_schedule(&date, &items)?;

        for item in items {
            if item.scheduled_time.len() != 5 {
                continue;
            }

            let (was_created, completion_changed) =
                sync_apple_reminder_item(&date, &item.habit_name, &item.scheduled_time, item.completed)?;
            if was_created {
                created += 1;
            }
            if completion_changed {
                completed_updated += 1;
            }
        }

        return Ok(format!(
            "Synced Apple Reminders: {} pulled into habits, {} created, {} completion state updates, {} obsolete reminders removed in list 'Second Brain'.",
            pulled_updates, created, completed_updated, removed_obsolete
        ));
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (date, items, state);
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
            sync_habits_to_apple_reminders,
            sync_apple_reminders_to_habits
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
