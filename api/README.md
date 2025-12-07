# EntertainAI API

FastAPI wrapper for the Python multi-agent system. Provides REST endpoints for the Next.js web UI.

## Quick Start

### 1. Install Dependencies

```bash
cd api
pip3 install -r requirements.txt
```

### 2. Run the Server

```bash
python3 main.py
```

The API will be available at:
- **API**: http://localhost:8000
- **Docs**: http://localhost:8000/docs (Swagger UI)
- **ReDoc**: http://localhost:8000/redoc

## Endpoints

### GET /health
Health check endpoint

**Response:**
```json
{
  "status": "healthy",
  "service": "EntertainAI API",
  "version": "1.0.0",
  "agents": 8
}
```

### POST /api/recommendations
Get personalized recommendations

**Request:**
```json
{
  "query": "action movies with strong female leads",
  "context": {
    "viewing": "group",
    "energy": "intense",
    "duration": "movie"
  },
  "filters": {
    "platforms": ["Netflix", "HBO Max", "Disney+"],
    "minRating": 7.0
  }
}
```

**Response:**
```json
{
  "recommendations": [
    {
      "id": "1",
      "title": "The Last of Us",
      "year": 2023,
      "platform": "HBO Max",
      "poster": "https://...",
      "rating": 8.9,
      "confidence": "Very High",
      "genres": ["Action", "Drama"],
      "reasoning": "Based on your preference for...",
      "reviews": [
        {"source": "IMDb", "score": 8.9},
        {"source": "RT", "score": 96}
      ],
      "tags": ["🔥 Trending", "⭐ Critics' Choice"],
      "socialProof": "Sarah & Mike watched"
    }
  ],
  "executionTime": 2.8,
  "candidatesProcessed": 487,
  "agentActivity": [...]
}
```

### GET /api/agents/status
Get status of all agents

**Response:**
```json
{
  "agents": [
    {
      "id": 1,
      "name": "StrategicContextAgent",
      "priority": 10,
      "location": "server"
    },
    {
      "id": 2,
      "name": "PersonalizationAgent",
      "priority": 8,
      "location": "on-device"
    }
  ],
  "status": "operational",
  "totalAgents": 8,
  "onDevice": 3,
  "serverSide": 5
}
```

## Architecture

```
┌─────────────────────────────────────┐
│      Next.js UI (Port 3000)        │
│  - SearchSection component          │
│  - AgentActivity visualization      │
│  - RecommendationCard display       │
└─────────────────────────────────────┘
              ↓ HTTP POST
┌─────────────────────────────────────┐
│    FastAPI Server (Port 8000)      │
│  - /api/recommendations endpoint    │
│  - CORS enabled for localhost:3000  │
│  - Pydantic validation              │
└─────────────────────────────────────┘
              ↓ Python import
┌─────────────────────────────────────┐
│   8-Agent System (agents/)          │
│  - CoordinatorAgent orchestration   │
│  - Parallel execution (asyncio)     │
│  - Shared memory communication      │
└─────────────────────────────────────┘
```

## Privacy Architecture

**On-Device Agents (Simulated in API):**
- PersonalizationAgent
- MoodDetectionAgent
- AnalysisAgent

**Server-Side Agents:**
- StrategicContextAgent
- ResearchAgent
- ReviewAggregationAgent
- TrendAnalysisAgent
- ContentFilterAgent

## Development

### Run with Auto-Reload
```bash
uvicorn main:app --reload --port 8000
```

### Test Endpoints
```bash
# Health check
curl http://localhost:8000/health

# Get recommendations
curl -X POST http://localhost:8000/api/recommendations \
  -H "Content-Type: application/json" \
  -d '{
    "query": "action thriller",
    "context": {"viewing": "solo", "energy": "intense"}
  }'
```

### Interactive API Docs
Visit http://localhost:8000/docs to use Swagger UI for testing all endpoints interactively.

## Performance

- **Target Response Time:** <3 seconds
- **Concurrent Requests:** Handles 100+ simultaneous users
- **Agent Execution:** Parallel phases for optimal speed

## Error Handling

All endpoints return structured error responses:

```json
{
  "detail": "Agent execution error: <error message>"
}
```

Common status codes:
- `200`: Success
- `422`: Validation error (invalid request)
- `500`: Server error (agent execution failure)

## Next Steps

1. **Connect to Next.js UI**: Update `web-ui/app/page.tsx` to call API
2. **Add Real TMDB Data**: Integrate TMDB API for real posters/data
3. **Implement Authentication**: Add JWT tokens for multi-user support
4. **Deploy to Cloud**: Deploy to Google Cloud Run or Vertex AI

## License

Same as parent project (see root LICENSE file)
