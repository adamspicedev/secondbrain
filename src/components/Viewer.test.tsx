import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createDocument, getDocumentDetail, updateDocument } from "../lib/api";
import { Viewer } from "./Viewer";

vi.mock("../lib/api", () => ({
  createDocument: vi.fn(),
  getDocumentDetail: vi.fn(),
  updateDocument: vi.fn(),
}));

describe("Viewer", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    cleanup();
  });

  it("shows validation when saving without title in create mode", async () => {
    const user = userEvent.setup();

    render(<Viewer documentId={null} onSaved={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: "Publish Changes" }));

    expect(await screen.findAllByText("Title is required")).toHaveLength(2);
    expect(createDocument).not.toHaveBeenCalled();
  });

  it("creates a document and switches to preview mode", async () => {
    const user = userEvent.setup();
    const onSaved = vi.fn();

    vi.mocked(createDocument).mockResolvedValueOnce({
      id: "doc-new",
      title: "Markdown Test",
      content: "- [x] done",
    });

    render(<Viewer documentId={null} onSaved={onSaved} />);

    await user.type(screen.getByPlaceholderText("Document title"), "Markdown Test");
    fireEvent.change(screen.getByPlaceholderText("this is a test"), {
      target: { value: "- [x] done" },
    });
    await user.click(screen.getByRole("button", { name: "Publish Changes" }));

    await waitFor(() => {
      expect(createDocument).toHaveBeenCalledWith("Markdown Test", "- [x] done");
      expect(onSaved).toHaveBeenCalled();
    });

    expect(screen.getByRole("button", { name: "Edit" })).toBeInTheDocument();
  });

  it("renders markdown preview for loaded documents with safe links", async () => {
    vi.mocked(getDocumentDetail).mockResolvedValueOnce({
      id: "doc-1",
      title: "Loaded",
      content: "- [x] shipped\n\n[docs](https://example.com)",
    });

    render(<Viewer documentId="doc-1" onSaved={vi.fn()} />);

    expect(await screen.findByText("Loaded")).toBeInTheDocument();

    const link = screen.getByRole("link", { name: "docs" });
    expect(link).toHaveAttribute("target", "_blank");
    expect(link).toHaveAttribute("rel", "noreferrer");

    expect(screen.getByRole("checkbox")).toBeChecked();
    expect(updateDocument).not.toHaveBeenCalled();
  });
});
