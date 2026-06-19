import type React from "react";
import { useCallback, useEffect, useState } from "react";
import { searchDocuments } from "../lib/api";

interface SearchResult {
  id: string;
  filename: string;
  content_preview: string;
  similarity: number;
}

interface SearchProps {
  refreshToken: number;
  onResultSelect: (result: SearchResult) => void;
}

export const Search: React.FC<SearchProps> = ({ refreshToken, onResultSelect }) => {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const fetchResults = useCallback(async (value: string, _token: number) => {
    setIsLoading(true);

    try {
      const searchResults = await searchDocuments(value);
      setResults(searchResults);
    } catch (error) {
      console.error("Search failed:", error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchResults(query, refreshToken);
  }, [fetchResults, query, refreshToken]);

  return (
    <div className="flex h-full flex-col rounded-[1.75rem] bg-transparent">
      <div className="mb-4 rounded-full bg-white px-4 py-3 shadow-[0_10px_24px_rgba(164,145,110,0.08)] ring-1 ring-white/80">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Find inspiration..."
          disabled={isLoading}
          className="w-full bg-transparent text-sm text-[#223047] placeholder:text-[#a9b4c5] outline-none disabled:cursor-not-allowed"
        />
      </div>

      <div className="flex flex-1 flex-col gap-3 overflow-y-auto pr-1">
        {!isLoading && results.length === 0 && (
          <div className="rounded-3xl border border-dashed border-[#eadfbf] bg-[#fff9eb] px-4 py-8 text-center text-xs font-semibold uppercase tracking-[0.18em] text-[#d7a84f]">
            No documents match this filter.
          </div>
        )}
        {results.map((result) => (
          <button
            type="button"
            key={result.id}
            className="group m-0 w-full cursor-pointer rounded-3xl border border-[#e9eef8] bg-white p-3 text-left shadow-[0_10px_20px_rgba(164,145,110,0.08)] transition hover:border-[#cfd8ff] hover:shadow-[0_14px_24px_rgba(164,145,110,0.12)]"
            onClick={() => onResultSelect(result)}
          >
            <div className="flex items-start gap-3">
              <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-2xl bg-[#d7f6e8] text-sm font-semibold text-[#10956a]">
                {result.filename.slice(0, 1).toUpperCase()}
              </div>
              <div className="min-w-0 flex-1">
                <h3 className="truncate text-sm font-semibold text-[#24314a]">{result.filename}</h3>
                <p className="mt-1 line-clamp-2 text-xs leading-5 text-[#8f9aad]">
                  {result.content_preview}
                </p>
              </div>
            </div>
          </button>
        ))}
      </div>
    </div>
  );
};
