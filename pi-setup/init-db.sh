#!/bin/bash
# Pi Database Setup Script
# Run on Raspberry Pi to initialize PostgreSQL with pgvector

set -e

echo "🚀 Setting up Second Brain database on Raspberry Pi..."

# Install dependencies
echo "📦 Installing dependencies..."
sudo apt-get update
sudo apt-get install -y docker.io docker-compose

# Enable Docker
sudo usermod -aG docker $USER
sudo systemctl enable docker
sudo systemctl start docker

echo "✅ Docker installed. Run this to complete group changes:"
echo "   newgrp docker"
echo ""
echo "Next, run: docker-compose up -d"
