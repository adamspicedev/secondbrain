import React, { useState, useEffect } from 'react';

interface ViewerProps {
  documentId: string | null;
}

export const Viewer: React.FC<ViewerProps> = ({ documentId }) => {
  const [content, setContent] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    if (!documentId) {
      setContent(null);
      return;
    }

    const fetchDocument = async () => {
      setIsLoading(true);
      try {
        const docContent = await (window as any).tauri.invoke('get_document', {
          id: documentId,
        });
        setContent(docContent);
      } catch (error) {
        console.error('Failed to fetch document:', error);
        setContent('Error loading document');
      } finally {
        setIsLoading(false);
      }
    };

    fetchDocument();
  }, [documentId]);

  if (!documentId) {
    return <div className="viewer-container">Select a result to view</div>;
  }

  if (isLoading) {
    return <div className="viewer-container">Loading...</div>;
  }

  return (
    <div className="viewer-container">
      <div className="viewer-content">{content}</div>
    </div>
  );
};
