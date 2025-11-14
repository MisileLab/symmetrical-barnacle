# Deployment Guide

## Prerequisites

- Docker and Docker Compose
- Steam Web API Key (get from https://steamcommunity.com/dev/apikey)
- OpenAI API Key
- (Optional) Discord Bot Token for Discord integration

## Environment Setup

1. Copy the example environment file:
```bash
cp .env.example .env
```

2. Edit `.env` and fill in your API keys:
```env
STEAM_API_KEY=your_steam_api_key_here
OPENAI_API_KEY=your_openai_api_key_here
SECRET_KEY=generate_a_random_secret_key
DISCORD_BOT_TOKEN=your_discord_bot_token_here  # Optional
```

## Quick Start with Docker

1. Build all services:
```bash
docker-compose build
```

2. Start all services:
```bash
docker-compose up -d
```

3. Check if services are running:
```bash
docker-compose ps
```

4. View logs:
```bash
docker-compose logs -f
```

The application will be available at:
- Frontend: http://localhost:3000
- Backend API: http://localhost:8000
- API Documentation: http://localhost:8000/docs

## Initial Data Setup

After starting the services, initialize some sample game data:

```bash
# Using Docker
docker-compose exec backend python /app/../scripts/init_sample_data.py

# Or crawl real games from Steam
docker-compose exec backend python /app/../scripts/crawl_games.py --limit 50
```

## Development Setup

### Backend Development

1. Create a virtual environment:
```bash
cd backend
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
```

2. Install dependencies:
```bash
pip install -r requirements.txt
```

3. Run the development server:
```bash
uvicorn app.main:app --reload
```

### Frontend Development

1. Install dependencies:
```bash
cd frontend
npm install
```

2. Start the development server:
```bash
npm start
```

## Discord Bot Setup (Phase 3)

1. Create a Discord application at https://discord.com/developers/applications

2. Add bot token to `.env`:
```env
DISCORD_BOT_TOKEN=your_bot_token_here
DISCORD_CLIENT_ID=your_client_id_here
```

3. Start the bot:
```bash
# Using Docker
docker-compose up discord-bot

# Or locally
cd bot
pip install -r requirements.txt
python discord_bot.py
```

4. Invite the bot to your server using the OAuth2 URL from Discord Developer Portal

## Production Deployment

### Using Docker Compose

1. Update environment variables for production in `.env`

2. Build and start services:
```bash
docker-compose -f docker-compose.prod.yml up -d
```

### Security Considerations

- Change all default passwords and secrets
- Use HTTPS in production (configure reverse proxy like Nginx)
- Set `ENVIRONMENT=production` in `.env`
- Regularly backup the PostgreSQL database
- Keep all dependencies updated

### Database Backups

Backup PostgreSQL:
```bash
docker-compose exec db pg_dump -U steamparty steamparty > backup.sql
```

Restore:
```bash
cat backup.sql | docker-compose exec -T db psql -U steamparty steamparty
```

## Monitoring

View service health:
```bash
curl http://localhost:8000/health
```

Check logs:
```bash
docker-compose logs -f backend
docker-compose logs -f frontend
```

## Troubleshooting

### Database connection issues
```bash
docker-compose restart db
docker-compose logs db
```

### ChromaDB not accessible
```bash
docker-compose restart chromadb
```

### Frontend can't connect to backend
- Check CORS settings in `backend/app/main.py`
- Verify `REACT_APP_API_URL` in frontend environment

## Scaling

For production traffic, consider:
- Using a managed PostgreSQL service (AWS RDS, Google Cloud SQL)
- Deploying ChromaDB separately
- Using Redis for caching and rate limiting
- Load balancing multiple backend instances
- CDN for frontend assets

## Support

For issues and questions:
- GitHub Issues: https://github.com/yourusername/steam-party-picker/issues
- Documentation: See README.md
