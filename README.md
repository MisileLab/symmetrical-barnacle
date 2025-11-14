# Steam Party Picker

스팀 그룹 게임 추천 서비스 - 여러 명의 스팀 프로필을 분석하여 모두가 만족할 만한 게임을 추천합니다.

## 주요 기능

### Phase 1 - MVP
- 스팀 프로필 입력 및 라이브러리 로딩
- 그룹 기반 게임 추천 (임베딩 + GPT)
- 기본 Like/Dislike 피드백
- 세션 기반 그룹 관리

### Phase 2 - 학습 & 품질 개선
- 피드백 학습을 통한 개인화된 추천
- 게임 제외 및 재추천 기능
- G1(모두 소유)/G2(일부 소유)/G3(미소유) 분류
- 추천 품질 지표 수집

### Phase 3 - 확장
- 자동 게임 데이터 크롤링 및 업데이트
- 디스코드 봇 연동
- 세션 히스토리 및 즐겨찾기
- 그룹 프로필 및 통계

## 기술 스택

### Backend
- FastAPI (Python 3.11+)
- PostgreSQL (데이터베이스)
- ChromaDB (벡터 데이터베이스)
- OpenAI API (GPT-4)
- Steam Web API

### Frontend
- React 18
- TypeScript
- Tailwind CSS
- React Router
- Axios

### Infrastructure
- Docker & Docker Compose
- Nginx (프록시)

## 프로젝트 구조

```
.
├── backend/              # FastAPI 백엔드
│   ├── app/
│   │   ├── api/         # API 라우터
│   │   ├── models/      # 데이터베이스 모델
│   │   ├── services/    # 비즈니스 로직
│   │   ├── utils/       # 유틸리티
│   │   └── main.py
│   ├── requirements.txt
│   └── Dockerfile
├── frontend/            # React 프론트엔드
│   ├── src/
│   │   ├── components/
│   │   ├── pages/
│   │   ├── services/
│   │   └── App.tsx
│   ├── package.json
│   └── Dockerfile
├── database/            # DB 스키마
├── scripts/             # 유틸리티 스크립트
├── bot/                 # 디스코드 봇 (Phase 3)
└── docker-compose.yml
```

## 시작하기

### 환경 변수 설정

`.env` 파일을 생성하고 다음 값들을 설정하세요:

```env
# Database
POSTGRES_USER=steamparty
POSTGRES_PASSWORD=your_password
POSTGRES_DB=steamparty
DATABASE_URL=postgresql://steamparty:your_password@db:5432/steamparty

# API Keys
STEAM_API_KEY=your_steam_api_key
OPENAI_API_KEY=your_openai_api_key

# App Config
SECRET_KEY=your_secret_key
FRONTEND_URL=http://localhost:3000
BACKEND_URL=http://localhost:8000
```

### Docker로 실행

```bash
# 모든 서비스 시작
docker-compose up -d

# 로그 확인
docker-compose logs -f

# 서비스 중지
docker-compose down
```

### 개발 모드

#### Backend

```bash
cd backend
python -m venv venv
source venv/bin/activate  # Windows: venv\Scripts\activate
pip install -r requirements.txt
uvicorn app.main:app --reload
```

#### Frontend

```bash
cd frontend
npm install
npm start
```

## API 문서

백엔드가 실행 중일 때 다음 URL에서 API 문서를 확인할 수 있습니다:
- Swagger UI: http://localhost:8000/docs
- ReDoc: http://localhost:8000/redoc

## 라이선스

MIT License - LICENSE 파일 참조

## 기여

이슈와 PR은 언제나 환영합니다!
