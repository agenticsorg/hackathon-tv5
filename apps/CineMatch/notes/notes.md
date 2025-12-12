⏺ Dev server is running and healthy. The sync was successful:

Summary:

- 100 embeddings generated from trending content
- 22 unique vectors in the database (some may have been duplicates from previous runs)
- All services are up: TMDB, RuVector, OpenAI

The database was locked because RuVector uses a file-based storage (./data/media-vectors.db) that only
allows one process at a time. You need to stop the dev server before running npm run embed:sync, then
restart it after.

To sync more content in the future:

# Stop the dev server first (Ctrl+C)

npm run embed:sync

# Then restart

npm run dev
