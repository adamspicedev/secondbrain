# Second Brain - Installation & Setup Guide

## 🛠️ Prerequisites

Before starting, ensure you have:

- **OpenAI Account**: [Create here](https://platform.openai.com/) and get an API key
- **Tailscale**: [Install here](https://tailscale.com/download)
  - Available on all platforms (Windows, Mac, Linux, iOS, Android)
- **Raspberry Pi**: Raspberry Pi 4 (2GB+ RAM) with latest Raspberry Pi OS
  - If using a different Pi or older OS, adjust Docker setup accordingly

### On Your Mac

- macOS 11+
- Xcode Command Line Tools: `xcode-select --install`
- Homebrew (optional but recommended): `/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"`

### On Your Raspberry Pi

- Raspberry Pi 4 (minimum 2GB RAM, 4GB+ recommended)
- Latest Raspberry Pi OS (Bookworm or later)
- 16GB+ microSD card
- SSH access enabled

---

## 📋 Step-by-Step Setup

### Step 1: Enable Tailscale on Both Devices

**On Mac:**

```bash
# Install Tailscale
brew install tailscale

# Start Tailscale daemon
sudo tailscale up

# Find your Tailscale IP
tailscale ip
# Output: 100.xx.xx.xx
```

**On Raspberry Pi:**

```bash
# Install Tailscale
curl -fsSL https://tailscale.com/install.sh | sh

# Start Tailscale
sudo tailscale up

# Find your Tailscale IP
tailscale ip
```

Test connectivity:

```bash
# From Mac
ping 100.xx.xx.xx  # Pi's Tailscale IP
```

---

### Step 2: Setup Database on Raspberry Pi

```bash
# SSH into your Pi
ssh pi@pi.local

# Or use Tailscale IP if .local doesn't work
ssh pi@100.xx.xx.xx

# Download/clone the repo to Pi
cd /home/pi
git clone <your-repo-url> secondbrain
cd secondbrain

# Make init script executable
chmod +x pi-setup/init-db.sh

# Run setup script (installs Docker, Docker Compose)
./pi-setup/init-db.sh

# Logout and login again (or run this) to apply Docker group changes
newgrp docker

# Start the database
cd pi-setup
docker-compose up -d

# Verify containers are running
docker ps

# Check logs if there are issues
docker logs secondbrain-db
```

**What this does:**

- Installs Docker & Docker Compose
- Pulls PostgreSQL 15 image
- Creates `secondbrain` database with `pgvector` extension
- Initializes vector search indexes
- Optional: Starts pgAdmin (web UI) on port 5050

**Test the connection from your Mac:**

```bash
# Install PostgreSQL client if needed
brew install postgresql

# Connect to database (replace with Pi's Tailscale IP or .local)
PGPASSWORD=changeme_securepassword psql \
  -h pi.local \
  -U secondbrain_user \
  -d secondbrain \
  -c "SELECT version();"
```

---

### Step 3: Setup Frontend on Mac

```bash
# Clone repo to Mac
cd ~/Projects
git clone <your-repo-url> secondbrain
cd secondbrain

# Install Node.js dependencies
npm install

# Create .env file
cp .env.example .env

# Edit .env with your values
nano .env
# Fill in:
# OPENAI_API_KEY=sk-...
# DATABASE_URL=postgres://secondbrain_user:changeme_securepassword@pi.local:5432/secondbrain
```

**If you don't have Node.js installed:**

```bash
# Using Homebrew
brew install node

# Or using nvm (recommended for version management)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18
```

---

### Step 4: Get OpenAI API Key

1. Go to [OpenAI API Keys](https://platform.openai.com/api-keys)
2. Click "Create new secret key"
3. Copy the key (you won't be able to see it again!)
4. Add to your `.env` file: `OPENAI_API_KEY=sk-...`

---

### Step 5: Run the App

```bash
# From the secondbrain directory on Mac
npm run dev

# This will:
# 1. Start Vite dev server (http://localhost:5173)
# 2. Compile Rust backend
# 3. Launch Tauri window with your app

# The Tauri window should open automatically
# If not, check console for errors
```

---

## 🚀 First Usage

1. **Upload a file**: Click the upload area, select an image or PDF
2. **Wait for processing**: The app will send to OpenAI, extract text, generate embeddings
3. **Search**: Type a query in the search box (e.g., "machine learning basics")
4. **View results**: Click a result to see extracted text in the right panel

---

## 📝 Configuration

### Change Database Password (Recommended for Production)

```bash
# On Pi, edit pi-setup/docker-compose.yml
POSTGRES_PASSWORD: your_secure_password

# Edit .env on Mac
DATABASE_URL=postgres://secondbrain_user:your_secure_password@pi.local:5432/secondbrain

# Restart containers
docker-compose down
docker-compose up -d
```

### Use Different Pi Hostname/IP

Edit your `.env` or directly in the Rust code:

```rust
// src-tauri/src/db.rs
let database_url = std::env::var("DATABASE_URL")
    .unwrap_or_else(|_| "postgres://user:pass@your-pi-ip:5432/secondbrain".to_string());
```

---

## 🔧 Troubleshooting

### "Cannot connect to database"

```bash
# Check Pi is reachable
ping pi.local
ping 100.xx.xx.xx  # Tailscale IP

# Check Docker containers running
ssh pi@pi.local
docker ps

# Check PostgreSQL logs
docker logs secondbrain-db

# Check network/firewall
docker exec secondbrain-db psql -U secondbrain_user -d secondbrain -c "SELECT 1;"
```

### "OpenAI API key invalid"

- Verify key in `.env`: `OPENAI_API_KEY=sk-...` (not in quotes)
- Check key has access to GPT-4 Vision API
- Verify account has credits

### "Tauri app won't start"

```bash
# Clear build cache
rm -rf src-tauri/target
rm -rf dist

# Rebuild
npm run dev

# Check for errors in console
# If still failing, check Tauri logs
RUST_LOG=debug npm run dev
```

### "pgAdmin not accessible"

- Make sure pgAdmin container is running: `docker ps | grep pgadmin`
- Access at: http://pi.local:5050 or http://100.xx.xx.xx:5050
- Default login: admin@example.com / admin

---

## 📦 Project Dependencies

### Frontend (Mac)

- **React 18**: UI framework
- **TypeScript**: Type safety
- **Tauri API**: Desktop integration
- **Axios**: HTTP client (future use)

### Backend (Rust)

- **Tauri**: Desktop framework
- **sqlx**: PostgreSQL driver
- **tokio**: Async runtime
- **reqwest**: HTTP client for OpenAI
- **pgvector**: Vector database support
- **serde**: JSON serialization

### Database (Pi)

- **PostgreSQL 15**: Relational database
- **pgvector**: Vector similarity search extension
- **Docker**: Containerization

---

## 📚 Next Steps

After setup is complete:

1. **Test the MVP**: Upload a few images/documents and search
2. **Add meeting transcription**: Record audio → transcribe → embed
3. **Add link summarization**: Paste URLs → fetch & summarize
4. **Customize UI**: Modify React components to suit your workflow
5. **Scale storage**: Add external S3 for document backups
6. **Multi-user**: Add authentication for sharing with others

---

## 💡 Tips

- **Tailscale DNS**: Use `pi.local` if available, otherwise use Tailscale IP
- **API costs**: Monitor OpenAI usage to avoid surprises
- **Database backups**: Regularly backup PostgreSQL on Pi
- **Embedding model**: `text-embedding-3-large` is accurate but ~9KB per embedding

---

## 🆘 Need Help?

Check these resources:

- [Tauri Documentation](https://tauri.app/v1/docs/)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [OpenAI API Reference](https://platform.openai.com/docs/api-reference)
- [Tailscale Documentation](https://tailscale.com/kb/)

---

**You're all set!** 🎉 Run `npm run dev` and start building your second brain.
