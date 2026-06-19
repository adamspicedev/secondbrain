use reqwest::Client;
use serde_json::json;
use std::fs;
use base64::Engine;

const OPENAI_API_KEY: &str = "OPENAI_API_KEY"; // Set via env var

fn infer_mime_type(file_path: &str) -> &'static str {
    let lower = file_path.to_lowercase();

    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".bmp") {
        "image/bmp"
    } else if lower.ends_with(".tif") || lower.ends_with(".tiff") {
        "image/tiff"
    } else {
        "image/jpeg"
    }
}

fn parse_text_from_chat_response(json_response: &serde_json::Value) -> Option<String> {
    if let Some(text) = json_response["choices"][0]["message"]["content"].as_str() {
        return Some(text.to_string());
    }

    let parts = json_response["choices"][0]["message"]["content"].as_array()?;
    let joined = parts
        .iter()
        .filter_map(|part| part["text"].as_str())
        .collect::<Vec<_>>()
        .join("\n");

    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

fn truncate_for_error(body: &str) -> String {
    const MAX: usize = 600;
    if body.len() <= MAX {
        body.to_string()
    } else {
        format!("{}...", &body[..MAX])
    }
}

pub async fn extract_text_from_image(file_path: &str) -> Result<String, String> {
    let api_key = std::env::var(OPENAI_API_KEY)
        .map_err(|_| "OPENAI_API_KEY not set".to_string())?;

    let image_data = fs::read(file_path)
        .map_err(|e| format!("Failed to read image: {}", e))?;

    let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_data);
    let mime_type = infer_mime_type(file_path);

    let client = Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&api_key)
        .json(&json!({
            "model": "gpt-4o-mini",
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": "Extract and transcribe all text from this image. Return only the extracted text. Preserve line breaks where helpful."
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": format!("data:{};base64,{}", mime_type, base64_image)
                            }
                        }
                    ]
                }
            ],
            "max_tokens": 2000,
            "temperature": 0
        }))
        .send()
        .await
        .map_err(|e| format!("OpenAI API error: {}", e))?;

    let status = response.status();
    let response_body = response
        .text()
        .await
        .map_err(|e| format!("Failed reading OpenAI response body: {}", e))?;

    if !status.is_success() {
        return Err(format!(
            "OpenAI vision request failed ({}): {}",
            status,
            truncate_for_error(&response_body)
        ));
    }

    let json_response: serde_json::Value = serde_json::from_str(&response_body)
        .map_err(|e| format!("Failed to parse OpenAI JSON: {}", e))?;

    let text = parse_text_from_chat_response(&json_response).ok_or_else(|| {
        format!(
            "No text in OpenAI response. Raw payload: {}",
            truncate_for_error(&response_body)
        )
    })?;

    Ok(text)
}

pub async fn extract_text_from_pdf(_file_path: &str) -> Result<String, String> {
    // For MVP, use pdfium or similar library
    // Placeholder for now
    Err("PDF extraction not yet implemented. Use PDF to text tool first.".to_string())
}

pub async fn extract_text_from_document(_file_path: &str) -> Result<String, String> {
    // For DOCX, use docx crate
    // Placeholder for now
    Err("Document extraction not yet implemented.".to_string())
}

pub async fn generate_embedding(text: &str) -> Result<[f32; 1536], String> {
    let api_key = std::env::var(OPENAI_API_KEY)
        .map_err(|_| "OPENAI_API_KEY not set".to_string())?;

    let client = Client::new();
    let response = client
        .post("https://api.openai.com/v1/embeddings")
        .bearer_auth(&api_key)
        .json(&json!({
            "model": "text-embedding-3-small",
            "input": text
        }))
        .send()
        .await
        .map_err(|e| format!("OpenAI API error: {}", e))?;

    let status = response.status();
    let response_body = response
        .text()
        .await
        .map_err(|e| format!("Failed reading OpenAI response body: {}", e))?;

    if !status.is_success() {
        return Err(format!(
            "OpenAI embedding request failed ({}): {}",
            status,
            truncate_for_error(&response_body)
        ));
    }

    let json_response: serde_json::Value = serde_json::from_str(&response_body)
        .map_err(|e| format!("Failed to parse OpenAI JSON: {}", e))?;

    let embedding_data = json_response["data"][0]["embedding"]
        .as_array()
        .ok_or("No embedding in response")?;

    let mut embedding = [0.0f32; 1536];
    for (i, val) in embedding_data.iter().enumerate() {
        if i >= 1536 {
            break;
        }
        embedding[i] = val
            .as_f64()
            .ok_or("Invalid embedding value")? as f32;
    }

    Ok(embedding)
}
