use reqwest::Client;
use serde_json::json;
use std::fs;

const OPENAI_API_KEY: &str = "OPENAI_API_KEY"; // Set via env var

pub async fn extract_text_from_image(file_path: &str) -> Result<String, String> {
    let api_key = std::env::var(OPENAI_API_KEY)
        .map_err(|_| "OPENAI_API_KEY not set".to_string())?;

    let image_data = fs::read(file_path)
        .map_err(|e| format!("Failed to read image: {}", e))?;

    let base64_image = base64::encode(&image_data);

    let client = Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&api_key)
        .json(&json!({
            "model": "gpt-4-vision-preview",
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": "Extract and transcribe all text from this image. Be thorough and preserve formatting where possible."
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": format!("data:image/jpeg;base64,{}", base64_image)
                            }
                        }
                    ]
                }
            ],
            "max_tokens": 2000
        }))
        .send()
        .await
        .map_err(|e| format!("OpenAI API error: {}", e))?;

    let json_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let text = json_response["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("No text in response")?
        .to_string();

    Ok(text)
}

pub async fn extract_text_from_pdf(file_path: &str) -> Result<String, String> {
    // For MVP, use pdfium or similar library
    // Placeholder for now
    Err("PDF extraction not yet implemented. Use PDF to text tool first.".to_string())
}

pub async fn extract_text_from_document(file_path: &str) -> Result<String, String> {
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
            "model": "text-embedding-3-large",
            "input": text
        }))
        .send()
        .await
        .map_err(|e| format!("OpenAI API error: {}", e))?;

    let json_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

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
