import { invoke } from "@tauri-apps/api/tauri";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createHabit, setHabitOccurrenceCompleted, syncHabitsToAppleReminders } from "./api";

vi.mock("@tauri-apps/api/tauri", () => ({
  invoke: vi.fn(),
}));

describe("api wrappers", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("maps createHabit args to tauri invoke payload", async () => {
    vi.mocked(invoke).mockResolvedValueOnce({ id: "1" });

    await createHabit("Take meds", 3, [0, 1, 2, 3, 4, 5, 6], ["08:00", "13:00", "20:00"]);

    expect(invoke).toHaveBeenCalledWith("create_habit", {
      name: "Take meds",
      timesPerDay: 3,
      daysOfWeek: [0, 1, 2, 3, 4, 5, 6],
      timesOfDay: ["08:00", "13:00", "20:00"],
    });
  });

  it("maps setHabitOccurrenceCompleted payload keys correctly", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(undefined);

    await setHabitOccurrenceCompleted("habit-1", "2026-06-19", "08:00", true);

    expect(invoke).toHaveBeenCalledWith("set_habit_occurrence_completed", {
      habitId: "habit-1",
      scheduledDate: "2026-06-19",
      scheduledTime: "08:00",
      completed: true,
    });
  });

  it("maps Apple Reminders sync payload keys correctly", async () => {
    vi.mocked(invoke).mockResolvedValueOnce("Created 1 reminder");

    await syncHabitsToAppleReminders("2026-06-21", [
      {
        habitName: "Walk",
        scheduledTime: "18:30",
        completed: false,
      },
    ]);

    expect(invoke).toHaveBeenCalledWith("sync_habits_to_apple_reminders", {
      date: "2026-06-21",
      items: [
        {
          habitName: "Walk",
          scheduledTime: "18:30",
          completed: false,
        },
      ],
    });
  });
});
