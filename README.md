# 🧠 Second Brain

AI-powered knowledge management desktop app built with **Tauri**, **React**, **Rust**, and **PostgreSQL**.

**Features:**
- 📤 Upload images and documents
- 🤖 AI-powered text extraction (GPT-4 Vision)
- 🔍 Semantic search with vector embeddings
- 💾 Persistent storage on Raspberry Pi
- 🖥️ Cross-machine access via Tailscale

---

## Quick Start

### Prerequisites
- **Mac**: macOS 11+, Xcode command line tools
- **Raspberry Pi**: Raspberry Pi 4 (2GB+ RAM), running Raspberry Pi OS
- **OpenAI API Key**: [Get one here](https://platform.openai.com/api-keys)
- **Tailscale**: [Install on both machines](https://tailscale.com/download)

### 1. Raspberry Pi Setup (Backend Database)

```bash
# SSH into your Pi
ssh pi@pi.local

# Clone/copy the repo to Pi
cd /home/pi/secondbrain

# Run database setup
chmod +x pi-setup/init-db.sh
./pi-setup/init-db.sh

# After Docker group changes take effect:
newgrp docker

# Start PostgreSQL + pgAdmin
docker-compose -f pi-setup/docker-compose.yml up -d

# Verify it's running
docker ps
```

Find your Pi's Tailscale IP:
```bash
tailscale ip
# Example: 100.64.x.x
```

Test database connection (you can do this from Mac):
```bash
PGPASSWORD=changeme_securepassword psql -h pi.local -U secondbrain_user -d secondbrain -c "SELECT version();"
```

### 2. Mac Setup (Frontend Desktop App)

```bash
# Install dependencies
npm install

# Set environment variables
export OPENAI_API_KEY="sk-..."
export DATABASE_URL="postgres://secondbrain_user:changeme_securepassword@pi.local:5432/secondbrain"

# Development mode
npm run dev

# This builds and launches the Tauri desktop app
```

The app will open with:
- **Left panel**: Upload & search
- **Right panel**: Document viewer

---

## Architecture

```
┌─────────────────────────────────────────┐
│          Mac (Desktop)                  │
│  ┌──────────────────────────────────┐   │
│  │  Tauri Window (React + Rust)     │   │
│  │  - File upload UI                │   │
│  │  - Search interface              │   │
│  │  - Document viewer               │   │
│  └──────────────────────────────────┘   │
└─────────────────────────────────────────┘
              Tailscale VPN
                   ↕
┌─────────────────────────────────────────┐
│      Raspberry Pi (Backend)             │
│  ┌──────────────────────────────────┐   │
│  │  PostgreSQL + pgvector           │   │
│  │  - Documents table               │   │
│  │  - Vector embeddings (1536-dim)  │   │
│  │  - Full-text search indexes      │   │
│  └──────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

**Data Flow:**
1. Upload image/doc → Tauri app
2. Send to OpenAI GPT-4 Vision → Extract text
3. Generate embeddings (text-embedding-3-large)
4. Store in PostgreSQL on Pi
5. Vector search returns semantic matches

---

## API / Tauri Commands

### `upload_file`
Extract text and store document.

```typescript
await tauri.invoke('upload_file', {
  file_path: '/path/to/image.png',
  file_type: 'image' // 'image' | 'pdf' | 'document'
});
// Returns: { id, filename, extracted_text }
```

### `search`
Semantic search across all documents.

```typescript
await tauri.invoke('search', {
  query: 'what is machine learning?'
});
// Returns: [{ id, filename, content_preview, similarity }]
```

### `get_document`
Retrieve full text of a document.

```typescript
await tauri.invoke('get_document', { id: 'uuid' });
// Returns: string (full extracted text)
```

---

## Environment Variables

Create a `.env` file in the project root:

```bash
# OpenAI API
OPENAI_API_KEY=sk-...

# Database (Pi)
DATABASE_URL=postgres://secondbrain_user:changeme_securepassword@pi.local:5432/secondbrain

# Optional: for development
RUST_LOG=debug
```

Or set before running:
```bash
export OPENAI_API_KEY="sk-..."
npm run dev
```

---

## Project Structure

```
secondbrain/
├── src/                              # React frontend
│   ├── components/
│   │   ├── Upload.tsx               # File upload form
│   │   ├── Search.tsx               # Search interface
│   │   └── Viewer.tsx               # Document viewer
│   ├── App.tsx                      # Main app component
│   ├── App.css                      # Styling
│   └── main.tsx                     # React entry point
├── src-tauri/                        # Rust backend
│   ├── src/
│   │   ├── main.rs                  # Tauri commands
│   │   ├── db.rs                    # PostgreSQL logic
│   │   ├── ai.rs                    # OpenAI integration
│   │   └── vector.rs                # Vector math
│   ├── Cargo.toml
│   └── tauri.conf.json
├── pi-setup/                         # Pi database setup
│   ├── docker-compose.yml
│   ├── init-db.sh
│   └── init.sql
├── package.json
├── vite.config.ts
└── README.md
```

---

## Development

### Hot Reload
```bash
npm run dev
# Changes to React code auto-reload in Tauri window
```

### Build Production Binary
```bash
npm run build
# Creates: src-tauri/target/release/secondbrain.dmg (Mac)
```

### Debug Database
```bash
# Connect to Pi database directly
PGPASSWORD=changeme_securepassword psql -h pi.local -U secondbrain_user -d secondbrain

# View documents
SELECT id, filename, created_at FROM documents;

# Test vector search
SELECT id, filename, 1 - (embedding <=> query_vector) as similarity
FROM documents
ORDER BY similarity DESC
LIMIT 5;
```

---

## Next Steps (Future Enhancements)

- [ ] **Meeting transcription**: Upload audio → Auto-transcribe + embed
- [ ] **Link summarization**: Paste URL → Fetch + summarize + embed
- [ ] **Tagging system**: Auto-tag documents by content
- [ ] **Export/sync**: Backup to cloud (S3, etc.)
- [ ] **Collaboration**: Share notes with other users
- [ ] **Mobile app**: React Native client for iOS/Android
- [ ] **Web interface**: Optional web-based UI via Tauri serv
- [ ] **Local LLM**: Ollama integration for offline AI

---

## Troubleshooting

### Database won't connect
```bash
# Check Pi is accessible
ping pi.local
tailscale ip

# Check PostgreSQL is running
docker ps

# Test connection
PGPASSWORD=changeme_securepassword psql -h pi.local -U secondbrain_user -d secondbrain -c "SELECT 1;"
```

### OpenAI API errors
- Verify `OPENAI_API_KEY` is set correctly
- Check API key has sufficient quota
- Ensure network connectivity

### Tauri build fails
```bash
# Clear cache and rebuild
rm -rf src-tauri/target
npm run build
```

---

## License
MIT

**Questions?** File an issue or check Tauri/PostgreSQL docs.
