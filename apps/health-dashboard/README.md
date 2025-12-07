# Health Dashboard

A static HTML/CSS/JS dashboard for monitoring Media Gateway service health.

## Features

- Real-time service health monitoring
- Auto-refresh every 10 seconds
- Color-coded status indicators (green/yellow/red)
- Response time tracking
- Dependency health monitoring (PostgreSQL, Redis, Qdrant, Kafka)
- Last check timestamps
- Responsive design

## Architecture

### Files

- `index.html` - Dashboard structure
- `styles.css` - Modern dark theme styling
- `app.js` - Vanilla JavaScript for data fetching and rendering

### Data Source

The dashboard polls `/health/aggregate` endpoint every 10 seconds to retrieve:
- Overall system status
- Individual service health (discovery, sona, auth, sync, ingestion, playback)
- Dependency health (PostgreSQL, Redis, Qdrant)
- Response times and latency metrics

## Access

The dashboard is served at `/dashboard/health` via the API gateway.

## Status Indicators

- **Green (Healthy)**: All systems operational
- **Yellow (Degraded)**: Some non-critical components failing
- **Red (Unhealthy)**: Critical components failing

## Implementation Details

- No external dependencies or frameworks
- Pure vanilla HTML/CSS/JS
- Automatic pause/resume when tab is hidden
- Error handling with user notifications
- Countdown timer for next refresh
