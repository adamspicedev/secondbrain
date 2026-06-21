import { type FormEvent, useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  type AppleReminderItem,
  createHabit,
  deleteHabit,
  type Habit,
  type HabitOccurrence,
  listHabitOccurrencesForDate,
  listHabitOccurrencesForRange,
  listHabits,
  setHabitOccurrenceCompleted,
  syncAppleRemindersToHabits,
  syncHabitsToAppleReminders,
  updateHabit,
} from "../lib/api";

const WEEKDAYS = [
  { value: 0, label: "Sun" },
  { value: 1, label: "Mon" },
  { value: 2, label: "Tue" },
  { value: 3, label: "Wed" },
  { value: 4, label: "Thu" },
  { value: 5, label: "Fri" },
  { value: 6, label: "Sat" },
];

interface HabitTime {
  id: string;
  value: string;
}

interface CalendarCell {
  key: string;
  day: number | null;
}

function createHabitTime(value: string): HabitTime {
  return {
    id: globalThis.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random()}`,
    value,
  };
}

const defaultTimes = [createHabitTime("08:00")];
const MONTH_NAMES = [
  "January",
  "February",
  "March",
  "April",
  "May",
  "June",
  "July",
  "August",
  "September",
  "October",
  "November",
  "December",
];

function todayIsoDate(): string {
  const now = new Date();
  const yyyy = now.getFullYear();
  const mm = String(now.getMonth() + 1).padStart(2, "0");
  const dd = String(now.getDate()).padStart(2, "0");
  return `${yyyy}-${mm}-${dd}`;
}

function toIsoDate(date: Date): string {
  const yyyy = date.getFullYear();
  const mm = String(date.getMonth() + 1).padStart(2, "0");
  const dd = String(date.getDate()).padStart(2, "0");
  return `${yyyy}-${mm}-${dd}`;
}

function syncTimeLabel(date: Date): string {
  return date.toLocaleTimeString();
}

export function Habits() {
  const [habits, setHabits] = useState<Habit[]>([]);
  const [selectedDateItems, setSelectedDateItems] = useState<HabitOccurrence[]>([]);
  const [rangeItems, setRangeItems] = useState<HabitOccurrence[]>([]);
  const [name, setName] = useState("");
  const [days, setDays] = useState<number[]>([0, 1, 2, 3, 4, 5, 6]);
  const [times, setTimes] = useState<HabitTime[]>(defaultTimes);
  const [selectedDate, setSelectedDate] = useState(todayIsoDate());
  const [calendarMonth, setCalendarMonth] = useState(() => {
    const now = new Date();
    return new Date(now.getFullYear(), now.getMonth(), 1);
  });
  const [editingHabitId, setEditingHabitId] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [deletingHabitId, setDeletingHabitId] = useState<string | null>(null);
  const [syncingReminders, setSyncingReminders] = useState(false);
  const [reminderStatus, setReminderStatus] = useState<string | null>(null);
  const [lastSyncedAt, setLastSyncedAt] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const syncingRemindersRef = useRef(false);

  useEffect(() => {
    let midnightTimer: ReturnType<typeof setTimeout>;

    const scheduleNextMidnightTick = () => {
      const now = new Date();
      const nextMidnight = new Date(now);
      nextMidnight.setHours(24, 0, 0, 0);
      const delayMs = Math.max(1000, nextMidnight.getTime() - now.getTime() + 100);

      midnightTimer = setTimeout(() => {
        const nextDate = todayIsoDate();
        const parsed = new Date(`${nextDate}T00:00:00`);
        setSelectedDate(nextDate);
        setCalendarMonth(new Date(parsed.getFullYear(), parsed.getMonth(), 1));
        scheduleNextMidnightTick();
      }, delayMs);
    };

    scheduleNextMidnightTick();

    return () => {
      clearTimeout(midnightTimer);
    };
  }, []);

  const lookbackStartDate = useMemo(() => {
    const base = new Date(`${selectedDate}T00:00:00`);
    base.setDate(base.getDate() - 29);
    const yyyy = base.getFullYear();
    const mm = String(base.getMonth() + 1).padStart(2, "0");
    const dd = String(base.getDate()).padStart(2, "0");
    return `${yyyy}-${mm}-${dd}`;
  }, [selectedDate]);

  const loadData = useCallback(
    async (targetDate: string = selectedDate) => {
      const [habitRows, occurrenceRows, rangeRows] = await Promise.all([
        listHabits(),
        listHabitOccurrencesForDate(targetDate),
        listHabitOccurrencesForRange(lookbackStartDate, targetDate),
      ]);

      setHabits(habitRows);
      setSelectedDateItems(occurrenceRows);
      setRangeItems(rangeRows);
    },
    [lookbackStartDate, selectedDate],
  );

  useEffect(() => {
    loadData().catch((err) => {
      setError(String(err));
    });
  }, [loadData]);

  useEffect(() => {
    const parsed = new Date(`${selectedDate}T00:00:00`);
    setCalendarMonth(new Date(parsed.getFullYear(), parsed.getMonth(), 1));
  }, [selectedDate]);

  const summaries = useMemo(() => {
    const perHabitMap = new Map<string, HabitOccurrence[]>();
    for (const item of rangeItems) {
      const existing = perHabitMap.get(item.habit_id) ?? [];
      existing.push(item);
      perHabitMap.set(item.habit_id, existing);
    }

    const perHabit = habits.map((habit) => {
      const items = perHabitMap.get(habit.id) ?? [];
      const total = items.length;
      const completed = items.filter((i) => i.completed).length;
      const completionRate = total === 0 ? 0 : Math.round((completed / total) * 100);

      const byDate = new Map<string, HabitOccurrence[]>();
      for (const item of items) {
        const existing = byDate.get(item.scheduled_date) ?? [];
        existing.push(item);
        byDate.set(item.scheduled_date, existing);
      }

      const scheduledDates = Array.from(byDate.keys()).sort((a, b) => (a > b ? -1 : 1));
      let streak = 0;
      for (const dateKey of scheduledDates) {
        const dayItems = byDate.get(dateKey) ?? [];
        const dayDone = dayItems.length > 0 && dayItems.every((entry) => entry.completed);
        if (dayDone) {
          streak += 1;
        } else {
          break;
        }
      }

      return {
        habitId: habit.id,
        habitName: habit.name,
        streak,
        completionRate,
      };
    });

    const totalOccurrences = rangeItems.length;
    const totalCompleted = rangeItems.filter((item) => item.completed).length;
    const overallCompletionRate =
      totalOccurrences === 0 ? 0 : Math.round((totalCompleted / totalOccurrences) * 100);

    return {
      perHabit,
      totalOccurrences,
      totalCompleted,
      overallCompletionRate,
    };
  }, [habits, rangeItems]);

  const selectedDateChecklistItems = useMemo(
    () => selectedDateItems.filter((item) => item.scheduled_date === selectedDate),
    [selectedDateItems, selectedDate],
  );

  const toggleDay = (value: number) => {
    setDays((prev) =>
      prev.includes(value)
        ? prev.filter((v) => v !== value)
        : [...prev, value].sort((a, b) => a - b),
    );
  };

  const addTime = () => {
    setTimes((prev) => [...prev, createHabitTime("12:00")]);
  };

  const updateTime = (id: string, value: string) => {
    setTimes((prev) => prev.map((item) => (item.id === id ? { ...item, value } : item)));
  };

  const removeTime = (id: string) => {
    setTimes((prev) => (prev.length === 1 ? prev : prev.filter((item) => item.id !== id)));
  };

  const handleCreateHabit = async (event: FormEvent) => {
    event.preventDefault();

    setError(null);

    if (!name.trim()) {
      setError("Please enter a habit name.");
      return;
    }

    if (days.length === 0) {
      setError("Select at least one day of the week.");
      return;
    }

    const timesOfDay = times.map((item) => item.value);

    if (timesOfDay.some((t) => !/^\d{2}:\d{2}$/.test(t))) {
      setError("Every time must be in HH:MM format.");
      return;
    }

    setSaving(true);
    try {
      if (editingHabitId) {
        await updateHabit(editingHabitId, name.trim(), timesOfDay.length, days, timesOfDay);
      } else {
        await createHabit(name.trim(), timesOfDay.length, days, timesOfDay);
      }
      setName("");
      setDays([0, 1, 2, 3, 4, 5, 6]);
      setTimes(defaultTimes);
      setEditingHabitId(null);
      await loadData();
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  };

  const handleToggleCompleted = async (item: HabitOccurrence) => {
    setError(null);
    try {
      await setHabitOccurrenceCompleted(
        item.habit_id,
        item.scheduled_date,
        item.scheduled_time,
        !item.completed,
      );
      await loadData();
    } catch (err) {
      setError(String(err));
    }
  };

  const handleEditHabit = (habit: Habit) => {
    setEditingHabitId(habit.id);
    setName(habit.name);
    setDays([...habit.days_of_week].sort((a, b) => a - b));
    setTimes(habit.times_of_day.map((time) => createHabitTime(time)));
  };

  const handleDeleteHabit = async (habitId: string) => {
    setError(null);
    setDeletingHabitId(habitId);
    try {
      await deleteHabit(habitId);
      if (editingHabitId === habitId) {
        setEditingHabitId(null);
        setName("");
        setDays([0, 1, 2, 3, 4, 5, 6]);
        setTimes(defaultTimes);
      }
      await loadData();
    } catch (err) {
      setError(String(err));
    } finally {
      setDeletingHabitId(null);
    }
  };

  const handleSyncAppleReminders = useCallback(
    async ({
      silentWhenNoDue = false,
      forceWhenEmpty = false,
    }: {
      silentWhenNoDue?: boolean;
      forceWhenEmpty?: boolean;
    } = {}) => {
      if (syncingReminders || syncingRemindersRef.current) {
        return;
      }

      syncingRemindersRef.current = true;
      setSyncingReminders(true);
      setReminderStatus(null);
      setError(null);

      await syncAppleRemindersToHabits(selectedDate);
      await loadData(selectedDate);

      if (selectedDateChecklistItems.length === 0 && !forceWhenEmpty) {
        if (!silentWhenNoDue) {
          setReminderStatus("No scheduled habits for the selected date.");
        }
        syncingRemindersRef.current = false;
        setSyncingReminders(false);
        return;
      }

      const reminderItems: AppleReminderItem[] = selectedDateChecklistItems.map((item) => ({
        habitName: item.habit_name,
        scheduledTime: item.scheduled_time,
        completed: item.completed,
      }));

      setSyncingReminders(true);
      try {
        const message = await syncHabitsToAppleReminders(selectedDate, reminderItems);
        setReminderStatus(message);
        setLastSyncedAt(syncTimeLabel(new Date()));
      } catch (err) {
        setReminderStatus(String(err));
      } finally {
        syncingRemindersRef.current = false;
        setSyncingReminders(false);
      }
    },
    [selectedDate, selectedDateChecklistItems, syncingReminders, loadData],
  );

  useEffect(() => {
    const intervalId = setInterval(() => {
      handleSyncAppleReminders({ silentWhenNoDue: true, forceWhenEmpty: true }).catch((err) => {
        setReminderStatus(String(err));
      });
    }, 60_000);

    return () => {
      clearInterval(intervalId);
    };
  }, [handleSyncAppleReminders]);

  const cancelEdit = () => {
    setEditingHabitId(null);
    setName("");
    setDays([0, 1, 2, 3, 4, 5, 6]);
    setTimes(defaultTimes);
  };

  const moveMonth = (offset: number) => {
    setCalendarMonth((prev) => new Date(prev.getFullYear(), prev.getMonth() + offset, 1));
  };

  const jumpToToday = () => {
    const today = new Date();
    setSelectedDate(toIsoDate(today));
    setCalendarMonth(new Date(today.getFullYear(), today.getMonth(), 1));
  };

  const year = calendarMonth.getFullYear();
  const month = calendarMonth.getMonth();
  const monthFirstWeekday = new Date(year, month, 1).getDay();
  const daysInMonth = new Date(year, month + 1, 0).getDate();

  const dayCells: CalendarCell[] = [];
  for (let i = 0; i < monthFirstWeekday; i += 1) {
    dayCells.push({ key: `pad-start-${year}-${month}-${i}`, day: null });
  }
  for (let day = 1; day <= daysInMonth; day += 1) {
    dayCells.push({ key: `day-${year}-${month}-${day}`, day });
  }
  let trailingPad = 0;
  while (dayCells.length % 7 !== 0) {
    dayCells.push({ key: `pad-end-${year}-${month}-${trailingPad}`, day: null });
    trailingPad += 1;
  }

  return (
    <div className="flex h-full flex-col gap-4 rounded-3xl bg-[#fffdf8] p-4">
      <div>
        <h2 className="text-lg font-semibold text-[#24314a]">Habit Tracking</h2>
        <p className="mt-1 text-xs text-[#6d788d]">
          Create recurring habits, edit schedules, and track completion across dates.
        </p>
      </div>

      <section className="rounded-2xl border border-[#efe7d7] bg-white p-3">
        <h3 className="text-sm font-semibold text-[#24314a]">Summary (Last 30 Days)</h3>
        <p className="mt-1 text-xs text-[#7e8aa0]">
          Overall: {summaries.overallCompletionRate}% ({summaries.totalCompleted}/
          {summaries.totalOccurrences})
        </p>
        <div className="mt-2 space-y-2">
          {summaries.perHabit.length === 0 ? (
            <p className="text-xs text-[#768299]">No habits to summarize yet.</p>
          ) : (
            summaries.perHabit.map((item) => (
              <div key={item.habitId} className="rounded-xl bg-[#faf6ec] px-3 py-2">
                <p className="text-sm font-semibold text-[#2b3750]">{item.habitName}</p>
                <p className="text-xs text-[#7e8aa0]">
                  Completion: {item.completionRate}% | Streak: {item.streak} days
                </p>
              </div>
            ))
          )}
        </div>
      </section>

      <section className="rounded-2xl border border-[#efe7d7] bg-white p-3">
        <div className="flex items-center justify-between gap-2">
          <h3 className="text-sm font-semibold text-[#24314a]">Checklist for {selectedDate}</h3>
          <button
            type="button"
            onClick={() => {
              handleSyncAppleReminders().catch((err) => {
                setReminderStatus(String(err));
              });
            }}
            disabled={syncingReminders}
            className="rounded-full bg-[#edf0ff] px-3 py-1 text-xs font-semibold text-[#4d5dcf] disabled:cursor-not-allowed disabled:opacity-60"
          >
            {syncingReminders ? "Syncing..." : "Send to Apple Reminders"}
          </button>
        </div>
        <div className="mt-3 space-y-2">
          {selectedDateChecklistItems.length === 0 ? (
            <p className="text-xs text-[#768299]">No scheduled habits for this date.</p>
          ) : (
            selectedDateChecklistItems.map((item) => (
              <label
                key={`${item.habit_id}-${item.scheduled_date}-${item.scheduled_time}`}
                className="flex items-center justify-between rounded-xl bg-[#faf6ec] px-3 py-2"
              >
                <div>
                  <p className="text-sm font-medium text-[#2b3750]">{item.habit_name}</p>
                  <p className="text-xs text-[#7e8aa0]">{item.scheduled_time}</p>
                </div>
                <input
                  type="checkbox"
                  checked={item.completed}
                  onChange={() => handleToggleCompleted(item)}
                  aria-label={`Mark ${item.habit_name} at ${item.scheduled_time} as done`}
                  className="h-4 w-4 accent-[#7e8bff]"
                />
              </label>
            ))
          )}
        </div>
        {reminderStatus ? <p className="mt-3 text-xs text-[#4b5c78]">{reminderStatus}</p> : null}
        {lastSyncedAt ? (
          <p className="mt-1 text-[11px] text-[#7e8aa0]">Last synced: {lastSyncedAt}</p>
        ) : null}
      </section>

      <section className="rounded-2xl border border-[#efe7d7] bg-white p-3">
        <h3 className="text-sm font-semibold text-[#24314a]">Date</h3>
        <div className="mt-2 rounded-xl border border-[#eadfca] bg-[#fffaf0] p-3">
          <div className="mb-3 flex items-center justify-between gap-2">
            <button
              type="button"
              onClick={() => moveMonth(-1)}
              className="rounded-full bg-[#edf0ff] px-3 py-1 text-xs font-semibold text-[#4d5dcf]"
            >
              Prev
            </button>
            <p className="text-base font-semibold text-[#24314a]">
              {MONTH_NAMES[month]} {year}
            </p>
            <button
              type="button"
              onClick={() => moveMonth(1)}
              className="rounded-full bg-[#edf0ff] px-3 py-1 text-xs font-semibold text-[#4d5dcf]"
            >
              Next
            </button>
          </div>

          <div className="mb-2 grid grid-cols-7 gap-1">
            {WEEKDAYS.map((day) => (
              <p
                key={`heading-${day.value}`}
                className="text-center text-[11px] font-semibold uppercase tracking-wide text-[#7c6d56]"
              >
                {day.label}
              </p>
            ))}
          </div>

          <div className="grid grid-cols-7 gap-1">
            {dayCells.map((cell) => {
              if (cell.day === null) {
                return <div key={cell.key} className="h-10 rounded-lg" />;
              }

              const dayDate = new Date(year, month, cell.day);
              const iso = toIsoDate(dayDate);
              const isSelected = iso === selectedDate;

              return (
                <button
                  key={iso}
                  type="button"
                  onClick={() => setSelectedDate(iso)}
                  className={`h-10 rounded-lg text-sm font-semibold transition ${
                    isSelected
                      ? "bg-[#9ea8ff] text-white"
                      : "bg-white text-[#61543f] hover:bg-[#f1e8d6]"
                  }`}
                >
                  {cell.day}
                </button>
              );
            })}
          </div>

          <div className="mt-3 flex items-center justify-between">
            <p className="text-sm font-semibold text-[#2b3750]">Selected: {selectedDate}</p>
            <button
              type="button"
              onClick={jumpToToday}
              className="rounded-full bg-[#f2ebdd] px-3 py-1 text-xs font-semibold text-[#61543f]"
            >
              Today
            </button>
          </div>
        </div>
      </section>

      <form
        onSubmit={handleCreateHabit}
        className="space-y-3 rounded-2xl border border-[#efe7d7] bg-white p-3"
      >
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold text-[#24314a]">
            {editingHabitId ? "Edit Habit" : "Create Habit"}
          </h3>
          {editingHabitId && (
            <button
              type="button"
              onClick={cancelEdit}
              className="rounded-full bg-[#f2ebdd] px-3 py-1 text-xs font-semibold text-[#61543f]"
            >
              Cancel
            </button>
          )}
        </div>

        <label
          htmlFor="habit-name"
          className="block text-xs font-semibold uppercase tracking-wide text-[#7c6d56]"
        >
          Habit Name
        </label>
        <input
          id="habit-name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="Habit name (e.g. Take medication)"
          className="w-full rounded-xl border border-[#eadfca] bg-[#fffaf0] px-3 py-2 text-sm outline-none focus:border-[#c8b18b]"
        />

        <div>
          <p className="mb-2 text-xs font-semibold uppercase tracking-wide text-[#7c6d56]">
            Days of Week
          </p>
          <div className="flex flex-wrap gap-2">
            {WEEKDAYS.map((day) => {
              const active = days.includes(day.value);
              return (
                <button
                  key={day.value}
                  type="button"
                  onClick={() => toggleDay(day.value)}
                  className={`rounded-full px-3 py-1 text-xs font-semibold transition ${
                    active
                      ? "bg-[#9ea8ff] text-white"
                      : "bg-[#f2ebdd] text-[#61543f] hover:bg-[#e7dcc8]"
                  }`}
                >
                  {day.label}
                </button>
              );
            })}
          </div>
        </div>

        <div>
          <div className="mb-2 flex items-center justify-between">
            <p className="text-xs font-semibold uppercase tracking-wide text-[#7c6d56]">
              Times Per Day ({times.length})
            </p>
            <button
              type="button"
              onClick={addTime}
              className="rounded-full bg-[#edf0ff] px-3 py-1 text-xs font-semibold text-[#4d5dcf]"
            >
              + Add Time
            </button>
          </div>

          <div className="space-y-2">
            {times.map((time, index) => (
              <div key={time.id} className="flex items-center gap-2">
                <label htmlFor={`habit-time-${time.id}`} className="sr-only">
                  Time {index + 1}
                </label>
                <input
                  id={`habit-time-${time.id}`}
                  type="time"
                  value={time.value}
                  onChange={(e) => updateTime(time.id, e.target.value)}
                  className="w-full rounded-xl border border-[#eadfca] bg-[#fffaf0] px-3 py-2 text-sm outline-none focus:border-[#c8b18b]"
                />
                <button
                  type="button"
                  onClick={() => removeTime(time.id)}
                  className="rounded-full bg-[#f4e8df] px-3 py-1 text-xs font-semibold text-[#8a5d3a]"
                >
                  Remove
                </button>
              </div>
            ))}
          </div>
        </div>

        <button
          type="submit"
          disabled={saving}
          className="w-full rounded-xl bg-[#93a0ff] px-4 py-2 text-sm font-semibold text-white transition hover:bg-[#7d8cff] disabled:cursor-not-allowed disabled:opacity-60"
        >
          {saving ? "Saving Habit..." : editingHabitId ? "Update Habit" : "Create Habit"}
        </button>
      </form>

      {error && <p className="rounded-xl bg-[#ffe8e8] px-3 py-2 text-xs text-[#9b3b3b]">{error}</p>}

      <section className="rounded-2xl border border-[#efe7d7] bg-white p-3">
        <h3 className="text-sm font-semibold text-[#24314a]">All Habits</h3>
        <div className="mt-2 space-y-2">
          {habits.length === 0 ? (
            <p className="text-xs text-[#768299]">No habits created yet.</p>
          ) : (
            habits.map((habit) => (
              <div key={habit.id} className="rounded-xl bg-[#faf6ec] px-3 py-2">
                <div className="flex items-start justify-between gap-2">
                  <div>
                    <p className="text-sm font-semibold text-[#2b3750]">{habit.name}</p>
                    <p className="text-xs text-[#7e8aa0]">
                      {habit.times_per_day}x/day at {habit.times_of_day.join(", ")}
                    </p>
                  </div>
                  <div className="flex gap-2">
                    <button
                      type="button"
                      onClick={() => handleEditHabit(habit)}
                      className="rounded-full bg-[#edf0ff] px-3 py-1 text-xs font-semibold text-[#4d5dcf]"
                    >
                      Edit
                    </button>
                    <button
                      type="button"
                      onClick={() => handleDeleteHabit(habit.id)}
                      disabled={deletingHabitId === habit.id}
                      className="rounded-full bg-[#f4e8df] px-3 py-1 text-xs font-semibold text-[#8a5d3a] disabled:opacity-60"
                    >
                      {deletingHabitId === habit.id ? "Deleting..." : "Delete"}
                    </button>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      </section>
    </div>
  );
}
