.PHONY: help build up down logs clean install-backend install-frontend install crawl-games init-data

help:
	@echo "Steam Party Picker - Makefile Commands"
	@echo ""
	@echo "  make build              Build all Docker images"
	@echo "  make up                 Start all services"
	@echo "  make down               Stop all services"
	@echo "  make logs               View logs"
	@echo "  make clean              Clean up containers and volumes"
	@echo "  make install            Install all dependencies"
	@echo "  make crawl-games        Crawl Steam games"
	@echo "  make init-data          Initialize sample data"

build:
	docker-compose build

up:
	docker-compose up -d

down:
	docker-compose down

logs:
	docker-compose logs -f

clean:
	docker-compose down -v
	rm -rf frontend/node_modules
	rm -rf backend/__pycache__
	rm -rf backend/app/__pycache__

install-backend:
	cd backend && pip install -r requirements.txt

install-frontend:
	cd frontend && npm install

install: install-backend install-frontend

crawl-games:
	python scripts/crawl_games.py --limit 50

init-data:
	python scripts/init_sample_data.py

dev-backend:
	cd backend && uvicorn app.main:app --reload --host 0.0.0.0 --port 8000

dev-frontend:
	cd frontend && npm start
