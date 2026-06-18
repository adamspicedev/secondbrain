-- Initialize pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Create documents table with vector column
CREATE TABLE IF NOT EXISTS documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    filename TEXT NOT NULL,
    file_type TEXT NOT NULL,
    extracted_text TEXT NOT NULL,
    embedding vector(1536),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Create index for fast vector similarity search
CREATE INDEX IF NOT EXISTS idx_embedding ON documents 
USING ivfflat (embedding vector_cosine_ops);

-- Create index on filename for text search
CREATE INDEX IF NOT EXISTS idx_filename ON documents (filename);

-- Create search log table (optional - for tracking searches)
CREATE TABLE IF NOT EXISTS search_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    query TEXT NOT NULL,
    results_count INT,
    created_at TIMESTAMP DEFAULT NOW()
);

GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO secondbrain_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO secondbrain_user;
