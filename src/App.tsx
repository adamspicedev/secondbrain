import React, { useState } from "react";
import { Upload } from "./components/Upload";
import { Search } from "./components/Search";
import { Viewer } from "./components/Viewer";
import "./App.css";

interface SearchResult {
  id: string;
  filename: string;
  content_preview: string;
  similarity: number;
}

function App() {
  const [selectedResult, setSelectedResult] = useState<SearchResult | null>(
    null,
  );

  return (
    <div className="app">
      <header>
        <h1>🧠 Second Brain</h1>
      </header>

      <main>
        <div className="left-panel">
          <Upload
            onUploadSuccess={() => {
              console.log("Upload successful");
            }}
          />
          <Search onResultSelect={setSelectedResult} />
        </div>

        <div className="right-panel">
          <Viewer documentId={selectedResult?.id || null} />
        </div>
      </main>
    </div>
  );
}

export default App;
