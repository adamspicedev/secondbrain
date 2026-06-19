import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { searchDocuments } from "../lib/api";
import { Search } from "./Search";

vi.mock("../lib/api", () => ({
  searchDocuments: vi.fn(),
}));

describe("Search", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("shows empty-state message when no documents are returned", async () => {
    vi.mocked(searchDocuments).mockResolvedValueOnce([]);

    render(<Search refreshToken={0} onResultSelect={vi.fn()} />);

    expect(await screen.findByText("No documents match this filter.")).toBeInTheDocument();
  });

  it("loads documents and calls onResultSelect when a result is clicked", async () => {
    const onResultSelect = vi.fn();
    vi.mocked(searchDocuments).mockResolvedValueOnce([
      {
        id: "doc-1",
        filename: "Medication",
        content_preview: "Take meds 3 times a day",
        similarity: 1,
      },
    ]);

    render(<Search refreshToken={0} onResultSelect={onResultSelect} />);

    const cardButton = await screen.findByRole("button", {
      name: /Medication/i,
    });
    await userEvent.click(cardButton);

    await waitFor(() => {
      expect(onResultSelect).toHaveBeenCalledWith(
        expect.objectContaining({
          id: "doc-1",
          filename: "Medication",
        }),
      );
    });
  });
});
