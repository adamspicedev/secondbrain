import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  createHabit,
  deleteHabit,
  listHabitOccurrencesForDate,
  listHabitOccurrencesForRange,
  listHabits,
  setHabitOccurrenceCompleted,
  syncHabitsToAppleReminders,
  updateHabit,
} from "../lib/api";
import { Habits } from "./Habits";

vi.mock("../lib/api", () => ({
  createHabit: vi.fn(),
  deleteHabit: vi.fn(),
  listHabitOccurrencesForDate: vi.fn(),
  listHabitOccurrencesForRange: vi.fn(),
  listHabits: vi.fn(),
  setHabitOccurrenceCompleted: vi.fn(),
  syncHabitsToAppleReminders: vi.fn(),
  updateHabit: vi.fn(),
}));

function todayIsoDate(): string {
  const now = new Date();
  const yyyy = now.getFullYear();
  const mm = String(now.getMonth() + 1).padStart(2, "0");
  const dd = String(now.getDate()).padStart(2, "0");
  return `${yyyy}-${mm}-${dd}`;
}

describe("Habits notifications", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(createHabit).mockResolvedValue({} as never);
    vi.mocked(updateHabit).mockResolvedValue({} as never);
    vi.mocked(deleteHabit).mockResolvedValue(undefined);
    vi.mocked(setHabitOccurrenceCompleted).mockResolvedValue(undefined);
    vi.mocked(listHabits).mockResolvedValue([]);
    vi.mocked(listHabitOccurrencesForRange).mockResolvedValue([]);
  });

  afterEach(() => {
    cleanup();
  });

  it("shows status and does not call Apple Reminders sync when no incomplete items exist", async () => {
    const user = userEvent.setup();

    vi.mocked(listHabitOccurrencesForDate).mockResolvedValue([
      {
        occurrence_id: "o1",
        habit_id: "h1",
        habit_name: "Read",
        scheduled_date: todayIsoDate(),
        scheduled_time: "08:00",
        completed: true,
      },
    ]);

    render(<Habits />);

    await screen.findByText(/Checklist for/);

    await user.click(screen.getByRole("button", { name: "Send to Apple Reminders" }));

    expect(
      await screen.findByText("No incomplete habits for the selected date."),
    ).toBeInTheDocument();
    expect(syncHabitsToAppleReminders).not.toHaveBeenCalled();
  });

  it("syncs only incomplete checklist items to Apple Reminders", async () => {
    const user = userEvent.setup();
    const selectedDate = todayIsoDate();

    vi.mocked(listHabitOccurrencesForDate).mockResolvedValue([
      {
        occurrence_id: "o1",
        habit_id: "h1",
        habit_name: "Walk",
        scheduled_date: selectedDate,
        scheduled_time: "08:00",
        completed: false,
      },
      {
        occurrence_id: "o2",
        habit_id: "h1",
        habit_name: "Walk",
        scheduled_date: selectedDate,
        scheduled_time: "20:00",
        completed: true,
      },
      {
        occurrence_id: "o3",
        habit_id: "h2",
        habit_name: "Stretch",
        scheduled_date: selectedDate,
        scheduled_time: "12:30",
        completed: false,
      },
    ]);

    vi.mocked(syncHabitsToAppleReminders).mockResolvedValue(
      "Created 2 reminder(s) in Apple Reminders list 'Second Brain'.",
    );

    render(<Habits />);

    await screen.findByText(/Checklist for/);

    await user.click(screen.getByRole("button", { name: "Send to Apple Reminders" }));

    await waitFor(() => {
      expect(syncHabitsToAppleReminders).toHaveBeenCalledWith(selectedDate, [
        {
          habitName: "Walk",
          scheduledTime: "08:00",
        },
        {
          habitName: "Stretch",
          scheduledTime: "12:30",
        },
      ]);
    });

    expect(
      await screen.findByText("Created 2 reminder(s) in Apple Reminders list 'Second Brain'."),
    ).toBeInTheDocument();
  });

  it("shows an error when Apple Reminders sync fails", async () => {
    const user = userEvent.setup();
    const selectedDate = todayIsoDate();

    vi.mocked(listHabitOccurrencesForDate).mockResolvedValue([
      {
        occurrence_id: "o1",
        habit_id: "h1",
        habit_name: "Walk",
        scheduled_date: selectedDate,
        scheduled_time: "08:00",
        completed: false,
      },
    ]);

    vi.mocked(syncHabitsToAppleReminders).mockRejectedValueOnce(
      new Error("Reminders permission denied"),
    );

    render(<Habits />);

    await screen.findByText(/Checklist for/);
    await user.click(screen.getByRole("button", { name: "Send to Apple Reminders" }));

    expect(syncHabitsToAppleReminders).toHaveBeenCalledWith(selectedDate, [
      {
        habitName: "Walk",
        scheduledTime: "08:00",
      },
    ]);
    expect(await screen.findByText(/Reminders permission denied/)).toBeInTheDocument();
  });
});
