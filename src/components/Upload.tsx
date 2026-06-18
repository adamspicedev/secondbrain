import React, { useState } from 'react';
import axios from 'axios';

interface UploadProps {
  onUploadSuccess: (filename: string) => void;
}

export const Upload: React.FC<UploadProps> = ({ onUploadSuccess }) => {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleFileChange = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    setIsLoading(true);
    setError(null);

    try {
      // Determine file type
      const fileType = file.type.startsWith('image/')
        ? 'image'
        : file.type === 'application/pdf'
        ? 'pdf'
        : 'document';

      // In production, use Tauri fs API to read the file
      const formData = new FormData();
      formData.append('file', file);
      formData.append('fileType', fileType);

      // Call Tauri command
      const response = await (window as any).tauri.invoke('upload_file', {
        file_path: file.name,
        file_type: fileType,
      });

      onUploadSuccess(response.filename);
      setError(null);
    } catch (err) {
      setError(`Upload failed: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="upload-container">
      <h2>Upload Document or Image</h2>
      <input
        type="file"
        onChange={handleFileChange}
        disabled={isLoading}
        accept="image/*,.pdf,.docx"
      />
      {isLoading && <p>Processing...</p>}
      {error && <p className="error">{error}</p>}
    </div>
  );
};
