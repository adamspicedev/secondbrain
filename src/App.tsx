import React, { useEffect, useState } from "react";
import { Search } from "./components/Search";
import { Viewer } from "./components/Viewer";

interface SearchResult {
  id: string;
  filename: string;
  content_preview: string;
  similarity: number;
}

function App() {
  const [theme, setTheme] = useState<"dawn" | "midnight">("dawn");
  const [selectedResult, setSelectedResult] = useState<SearchResult | null>(
    null,
  );

  const [searchRefreshKey, setSearchRefreshKey] = useState(0);

  const handleSaved = () => {
    setSearchRefreshKey((value) => value + 1);
  };

  const handleNewDocument = () => {
    setSelectedResult(null);
  };

  useEffect(() => {
    const storedTheme = localStorage.getItem("secondbrain-theme");
    if (storedTheme === "midnight" || storedTheme === "dawn") {
      setTheme(storedTheme);
      return;
    }

    const prefersDark = window.matchMedia(
      "(prefers-color-scheme: dark)",
    ).matches;
    setTheme(prefersDark ? "midnight" : "dawn");
  }, []);

  useEffect(() => {
    if (theme === "midnight") {
      document.documentElement.setAttribute("data-theme", "midnight");
    } else {
      document.documentElement.removeAttribute("data-theme");
    }
    localStorage.setItem("secondbrain-theme", theme);
  }, [theme]);

  const handleToggleTheme = () => {
    setTheme((prev) => (prev === "dawn" ? "midnight" : "dawn"));
  };

  return (
    <div className="min-h-screen bg-[#f7f3e8] p-4 text-(--ink)">
      <div className="mx-auto flex min-h-[calc(100vh-2rem)] max-w-[1400px] flex-col overflow-hidden rounded-[2rem] border border-white/80 bg-[#fbf8f1] p-4 shadow-[0_18px_50px_rgba(164,145,110,0.14)]">
        <header className="flex items-center justify-between rounded-[1.5rem] px-2 py-2">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-[#d9ddff] text-xl text-[#6472ff] shadow-inner">
              ☼
            </div>
            <h1 className="text-xl font-semibold tracking-tight text-[#24314a]">
              Second Brain
            </h1>
          </div>

          <button
            onClick={handleNewDocument}
            className="rounded-full bg-[#9ea8ff] px-5 py-2.5 text-sm font-semibold text-white shadow-[0_10px_24px_rgba(147,156,255,0.35)] transition hover:bg-[#8792ff]"
          >
            + New Idea
          </button>
        </header>

        <main className="grid flex-1 grid-cols-[270px_minmax(0,1fr)] gap-6 px-2 pb-2 pt-4">
          <aside className="overflow-y-auto">
            <Search
              refreshToken={searchRefreshKey}
              onResultSelect={setSelectedResult}
            />
          </aside>

          <section className="overflow-hidden rounded-[2rem] bg-white shadow-[0_12px_30px_rgba(164,145,110,0.12)] ring-1 ring-white/80">
            <Viewer
              documentId={selectedResult?.id || null}
              onSaved={handleSaved}
            />
          </section>
        </main>
      </div>
    </div>
  );
}

export default App;
