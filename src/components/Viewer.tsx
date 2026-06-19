import type React from "react";
import { useEffect, useState } from "react";
import ReactMarkdown from "react-markdown";
import { createDocument, getDocumentDetail, updateDocument } from "../lib/api";

interface ViewerProps {
  documentId: string | null;
  onSaved: () => void;
}

export const Viewer: React.FC<ViewerProps> = ({ documentId, onSaved }) => {
  const [activeDocumentId, setActiveDocumentId] = useState<string | null>(documentId);
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [isEditing, setIsEditing] = useState(!documentId);
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [status, setStatus] = useState<string | null>(null);

  useEffect(() => {
    setActiveDocumentId(documentId);

    if (!documentId) {
      setTitle("");
      setContent("");
      setIsEditing(true);
      setStatus("Creating a new document");
      return;
    }

    setIsEditing(false);

    const fetchDocument = async () => {
      setIsLoading(true);
      try {
        const doc = await getDocumentDetail(documentId);
        setTitle(doc.title);
        setContent(doc.content);
        setStatus(null);
      } catch (error) {
        console.error("Failed to fetch document:", error);
        setStatus("Error loading document");
      } finally {
        setIsLoading(false);
      }
    };

    fetchDocument();
  }, [documentId]);

  if (isLoading) {
    return <div className="p-5 text-sm text-[#7d8aa0]">Loading...</div>;
  }

  const handleSave = async () => {
    if (!title.trim()) {
      setStatus("Title is required");
      return;
    }

    setIsSaving(true);
    setStatus("Saving...");

    try {
      if (activeDocumentId) {
        await updateDocument(activeDocumentId, title.trim(), content);
      } else {
        const created = await createDocument(title.trim(), content);
        setActiveDocumentId(created.id);
      }
      setStatus("Saved");
      setIsEditing(false);
      onSaved();
    } catch (error) {
      setStatus(`Save failed: ${error instanceof Error ? error.message : String(error)}`);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="flex h-full flex-col p-5">
      <div className="flex items-start justify-between gap-4 px-2 pb-4 pt-1">
        <div className="min-w-0">
          {isEditing ? (
            <input
              className="w-full bg-transparent text-3xl font-semibold tracking-tight text-[#24314a] outline-none placeholder:text-[#c0c8d8]"
              placeholder="Document title"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
            />
          ) : (
            <div className="text-3xl font-semibold tracking-tight text-[#24314a]">
              {title || "Untitled"}
            </div>
          )}
          <div className="mt-2 flex items-center gap-2 text-xs font-medium text-[#22a06b]">
            <span>✓</span>
            <span>{status ?? "Auto-saved in Studio"}</span>
          </div>
        </div>

        <div className="flex gap-3">
          {!isEditing ? (
            <button
              type="button"
              onClick={() => setIsEditing(true)}
              className="rounded-full bg-white px-5 py-3 text-sm font-semibold text-[#24314a] shadow-[0_8px_20px_rgba(164,145,110,0.10)] ring-1 ring-[#f0e8d9] transition hover:bg-[#fff9f1]"
            >
              Edit
            </button>
          ) : null}

          <button
            type="button"
            onClick={handleSave}
            disabled={isSaving}
            className="rounded-full bg-[#39d39a] px-6 py-3 text-sm font-semibold text-white shadow-[0_12px_28px_rgba(57,211,154,0.25)] transition hover:bg-[#2fc58d] disabled:cursor-not-allowed disabled:opacity-60"
          >
            Publish Changes
          </button>
        </div>
      </div>

      {status && !status.includes("Auto-saved") && (
        <div className="px-2 pb-3 text-xs text-[#7d8aa0]">{status}</div>
      )}

      <div className="flex-1 rounded-4xl bg-[#fffdf8] p-6 shadow-[0_10px_24px_rgba(164,145,110,0.08)] ring-1 ring-[#f0e8d9]">
        {isEditing ? (
          <textarea
            className="min-h-105 w-full resize-none border-0 bg-transparent text-[15px] leading-8 text-[#2b3547] outline-none placeholder:text-[#b8c1d0]"
            value={content}
            onChange={(e) => setContent(e.target.value)}
            placeholder="this is a test"
          />
        ) : (
          <article className="markdown-preview prose prose-slate max-w-none text-[#2b3547]">
            <ReactMarkdown>{content || ""}</ReactMarkdown>
          </article>
        )}
      </div>

      <div className="flex items-center justify-between px-2 pt-3 text-[11px] font-semibold uppercase tracking-[0.18em] text-[#c0c8d8]">
        <div className="flex gap-6">
          <span className={isEditing ? "text-[#a9b4c5]" : "text-[#24314a]"}>Markdown</span>
          <span className={!isEditing ? "text-[#a9b4c5]" : "text-[#24314a]"}>Visual</span>
        </div>
        <div>{content.length} characters of magic</div>
      </div>
    </div>
  );
};
