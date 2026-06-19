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

export interface DocumentDetail {
  id: string;
  title: string;
  content: string;
}

/**
 * Search documents by keyword
 * @param query - Natural language search query
 */
export async function searchDocuments(query: string): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("search", { query });
}

/**
 * Get full content and title of a document
 * @param id - Document UUID
 */
export async function getDocumentDetail(id: string): Promise<DocumentDetail> {
  return invoke<DocumentDetail>("get_document_detail", { id });
}

/**
 * Create a new document
 */
export async function createDocument(
  title: string,
  content: string,
): Promise<DocumentDetail> {
  return invoke<DocumentDetail>("create_document", { title, content });
}

/**
 * Update an existing document
 */
export async function updateDocument(
  id: string,
  title: string,
  content: string,
): Promise<void> {
  return invoke<void>("update_document", { id, title, content });
}
