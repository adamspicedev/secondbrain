import React, { useState } from "react";

interface SearchResult {
  id: string;
  filename: string;
  content_preview: string;
  similarity: number;
}

interface SearchProps {
  onResultSelect: (result: SearchResult) => void;
}

export const Search: React.FC<SearchProps> = ({ onResultSelect }) => {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!query.trim()) return;

    setIsLoading(true);

    try {
      const searchResults = await (window as any).tauri.invoke("search", {
        query,
      });

      setResults(searchResults);
    } catch (error) {
      console.error("Search failed:", error);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="search-container">
      <form onSubmit={handleSearch}>
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search your documents..."
          disabled={isLoading}
        />
        <button type="submit" disabled={isLoading}>
          {isLoading ? "Searching..." : "Search"}
        </button>
      </form>

      <div className="results">
        {results.map((result) => (
          <div
            key={result.id}
            className="result-item"
            onClick={() => onResultSelect(result)}
          >
            <h3>{result.filename}</h3>
            <p className="similarity">
              Match: {(result.similarity * 100).toFixed(1)}%
            </p>
            <p className="preview">{result.content_preview}</p>
          </div>
        ))}
      </div>
    </div>
  );
};
