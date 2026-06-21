import { invoke } from "@tauri-apps/api/tauri";

export interface SearchResult {
  id: string;
  filename: string;
  content_preview: string;
  similarity: number;
}

export interface UploadResponse {
  id: string;
  filename: string;
  extracted_text: string;
}

export interface DocumentDetail {
  id: string;
  title: string;
  content: string;
}

export interface Habit {
  id: string;
  name: string;
  times_per_day: number;
  days_of_week: number[];
  times_of_day: string[];
}

export interface HabitOccurrence {
  occurrence_id: string;
  habit_id: string;
  habit_name: string;
  scheduled_date: string;
  scheduled_time: string;
  completed: boolean;
}

/**
 * Search documents by keyword
 * @param query - Natural language search query
 */
export async function searchDocuments(query: string): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("search", { query });
}

/**
 * Get full content and title of a document
 * @param id - Document UUID
 */
export async function getDocumentDetail(id: string): Promise<DocumentDetail> {
  return invoke<DocumentDetail>("get_document_detail", { id });
}

/**
 * Create a new document
 */
export async function createDocument(title: string, content: string): Promise<DocumentDetail> {
  return invoke<DocumentDetail>("create_document", { title, content });
}

/**
 * Update an existing document
 */
export async function updateDocument(id: string, title: string, content: string): Promise<void> {
  return invoke<void>("update_document", { id, title, content });
}

export async function createHabit(
  name: string,
  timesPerDay: number,
  daysOfWeek: number[],
  timesOfDay: string[],
): Promise<Habit> {
  return invoke<Habit>("create_habit", {
    name,
    timesPerDay,
    daysOfWeek,
    timesOfDay,
  });
}

export async function updateHabit(
  id: string,
  name: string,
  timesPerDay: number,
  daysOfWeek: number[],
  timesOfDay: string[],
): Promise<Habit> {
  return invoke<Habit>("update_habit", {
    id,
    name,
    timesPerDay,
    daysOfWeek,
    timesOfDay,
  });
}

export async function deleteHabit(id: string): Promise<void> {
  return invoke<void>("delete_habit", { id });
}

export async function listHabits(): Promise<Habit[]> {
  return invoke<Habit[]>("list_habits");
}

export async function listHabitOccurrencesForDate(date: string): Promise<HabitOccurrence[]> {
  return invoke<HabitOccurrence[]>("list_habit_occurrences_for_date", { date });
}

export async function listHabitOccurrencesForRange(
  startDate: string,
  endDate: string,
): Promise<HabitOccurrence[]> {
  return invoke<HabitOccurrence[]>("list_habit_occurrences_for_range", {
    startDate,
    endDate,
  });
}

export async function setHabitOccurrenceCompleted(
  habitId: string,
  scheduledDate: string,
  scheduledTime: string,
  completed: boolean,
): Promise<void> {
  return invoke<void>("set_habit_occurrence_completed", {
    habitId,
    scheduledDate,
    scheduledTime,
    completed,
  });
}

export interface AppleReminderItem {
  habitName: string;
  scheduledTime: string;
}

export async function syncHabitsToAppleReminders(
  date: string,
  items: AppleReminderItem[],
): Promise<string> {
  return invoke<string>("sync_habits_to_apple_reminders", {
    date,
    items,
  });
}
