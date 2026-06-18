import { invoke } from "@tauri-apps/api/tauri";

export interface SearchResult {
  id: string;
  filename: string;
  content_preview: string;
  similarity: number;
}

export interface UploadResponse {
  id: string;
  filename: string;
  extracted_text: string;
}

/**
 * Upload and process a file
 * @param filePath - Full path to the file
 * @param fileType - 'image', 'pdf', or 'document'
 */
export async function uploadFile(
  filePath: string,
  fileType: "image" | "pdf" | "document",
): Promise<UploadResponse> {
  return invoke<UploadResponse>("upload_file", {
    file_path: filePath,
    file_type: fileType,
  });
}

/**
 * Search documents by semantic similarity
 * @param query - Natural language search query
 */
export async function searchDocuments(query: string): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("search", { query });
}

/**
 * Get full content of a document
 * @param id - Document UUID
 */
export async function getDocument(id: string): Promise<string> {
  return invoke<string>("get_document", { id });
}

/**
 * Delete a document
 * @param id - Document UUID
 */
export async function deleteDocument(id: string): Promise<void> {
  return invoke<void>("delete_document", { id });
}
