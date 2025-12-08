# EmotiStream Backend - Cloud Run Dockerfile
FROM node:20-slim AS builder

WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY . .

# Build TypeScript
RUN npm run build

# Production stage
FROM node:20-slim AS runner

WORKDIR /app

# Copy built files and dependencies
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./

# Create data directory for persistence
RUN mkdir -p /app/data

# Copy schemas for database initialization (ruvector for local, pgvector for cloud)
COPY --from=builder /app/src/persistence/schema.sql ./dist/persistence/schema.sql
COPY --from=builder /app/src/persistence/schema-pgvector.sql ./dist/persistence/schema-pgvector.sql

# Set environment variables
ENV NODE_ENV=production
# Cloud Run provides PORT env var (default 8080), don't hardcode
# ENV PORT=8080

# Expose port (documentation only, Cloud Run uses PORT env var)
EXPOSE 8080

# Start the server
CMD ["node", "dist/server.js"]
