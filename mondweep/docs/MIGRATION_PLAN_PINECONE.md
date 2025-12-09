
# 🗺️ Migration Plan: Switching to Pinecone Vector Search

We will transition the Netlify Application from using a memory-heavy local JSON search to a Serverless Pinecone Vector Search.

## 🏗️ Architecture Change

- **CurrentState**: `kg-search.js` loads `1.3GB JSON` into memory (Slow, fragile, OOM risk).
- **Target State**: `kg-search.js` acts as a lightweight proxy.
  1.  Receives query.
  2.  Generates embedding via **Gemini API** (REST).
  3.  Queries **Pinecone Index** (REST).
  4.  Returns results.

## 🛠️ Implementation Steps

### 1. Update Netlify Function (`netlify/functions/kg-search.js`)
Refactor the function to use the **Pinecone Data Plane REST API**. This avoids adding heavy Node.js dependencies (`@pinecone-database/pinecone` usually requires build steps).

**Logic Flow:**
```javascript
// 1. Embed Query
const embedding = await getGeminiEmbedding(query);

// 2. Search Pinecone
const pineconeResponse = await fetch(`https://${PINECONE_HOST}/vectors/query`, {
  method: 'POST',
  headers: { 'Api-Key': PINECONE_API_KEY },
  body: JSON.stringify({
    vector: embedding,
    topK: limit,
    includeMetadata: true
  })
});

// 3. Format & Return
return formatResults(pineconeResponse);
```

### 2. Environment Configuration
You will need to set the following Environment Variables in your **New Netlify Project**:

| Variable | Description |
|----------|-------------|
| `PINECONE_API_KEY` | Your Pinecone API Key |
| `PINECONE_HOST` | The Data Plane Host URL (e.g. `media-knowledge-graph-....pinecone.io`) |
| `GOOGLE_GEMINI_API_KEY` | For generating query embeddings |

### 3. Deployment (New Project)
1.  **Push** the code changes to the `antigravity` branch.
2.  **Create New Site** in Netlify dashboard (Import from Git -> `antigravity` branch).
3.  **Set Environment Variables** in the Netlify UI (Site Settings > Environment).
4.  **Deploy**.

---

### 📝 Action Plan
1.  [ ] **Extract Pinecone Host**: We need to query your index to get the specific `HOST` URL for the REST API.
2.  [ ] **Modify Code**: Rewrite `netlify/functions/kg-search.js`.
3.  [ ] **Commit & Push**: Prepare for deployment.
