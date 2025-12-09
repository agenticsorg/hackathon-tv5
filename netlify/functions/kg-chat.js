
/**
 * Knowledge Graph Chatbot API using Gemini
 * POST /.netlify/functions/kg-chat
 * Body: { 
 *   "messages": [{ "role": "user", "content": "..." }],
 *   "context": { "searchResults": [...] }
 * }
 */

exports.handler = async (event, context) => {
    const headers = {
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Headers': 'Content-Type',
        'Content-Type': 'application/json',
    };

    if (event.httpMethod === 'OPTIONS') {
        return { statusCode: 200, headers, body: '' };
    }

    try {
        const body = JSON.parse(event.body || '{}');
        const messages = body.messages || [];
        const searchContext = body.context || {};

        // Validate configuration
        const GEMINI_API_KEY = process.env.GOOGLE_GEMINI_API_KEY || process.env.GEMINI_API_KEY;
        if (!GEMINI_API_KEY) {
            throw new Error('Missing Gemini API Key');
        }

        // Construct the System Prompt
        let moviesContext = "";
        if (searchContext.searchResults && searchContext.searchResults.length > 0) {
            const topMovies = searchContext.searchResults.slice(0, 5); // Just top 5 to save tokens
            moviesContext = "Current Search Results shown to user:\n" +
                topMovies.map((m, i) => `${i + 1}. ${m.title} (${m.year}): ${m.overview}`).join("\n");
        } else {
            moviesContext = "No movies are currently currently shown.";
        }

        const systemInstruction = `You are a helpful Movie Assistant integrated into a Knowledge Graph semantic search engine.
    
${moviesContext}

Your Goal: Help the user explore these movies or find new ones.
1. If the user asks "Why did I get this?", explain the connection between their query and the movie's themes/plot (referenced in the results).
2. If the user wants to refine their search (e.g. "something happier", "older movies"), suggest what they should type in the search bar, or just give recommendations from your general knowledge.
3. Be concise, friendly, and conversational.
4. If asked about technical details, these results come from a Pinecone Vector Database using 768-dimensional Vertex AI embeddings.`;

        // Format history for Gemini (checking for 'user' vs 'model' roles)
        // Gemini API expects: { role: 'user'|'model', parts: [{ text: '...' }] }
        // Format history for Gemini (checking for 'user' vs 'model' roles)
        // Gemini API expects: { role: 'user'|'model', parts: [{ text: '...' }] }
        // IMPORTANT: The conversation MUST start with a user message.
        // We filter out any initial 'model' messages (like the welcome message).

        let contents = messages
            .filter((msg, index) => {
                // If it's the first message and it's from model, skip it
                if (index === 0 && msg.role === 'model') return false;
                // Basic cleanup of consecutive roles if needed (though UI handles this usually)
                return true;
            })
            .map(msg => ({
                role: msg.role === 'user' ? 'user' : 'model',
                parts: [{ text: msg.content }]
            }));

        // Double check: if still empty or starts with model, force fix or fail gracefully
        if (contents.length === 0 || contents[0].role !== 'user') {
            // If user hasn't said anything yet, we can't really "chat" in this API structure easily without a prompt
            // But usually this function is called AFTER user says something.
            // If we ended up with a model message first (e.g. from history filtering weirdness), skip it
            if (contents.length > 0 && contents[0].role === 'model') {
                contents.shift();
            }
        }

        // Add system instruction effectively by prepending it to the first user message
        if (contents.length > 0 && contents[0].role === 'user') {
            contents[0].parts[0].text = systemInstruction + "\n\nUser Question: " + contents[0].parts[0].text;
        } else {
            // Fallback if no user message found (should be impossible if triggered by user input)
            contents = [{ role: 'user', parts: [{ text: systemInstruction + "\n\n(User sent empty message)" }] }];
        }

        // Call Gemini API using native https module to avoid dependency issues
        const https = require('https');
        const model = 'gemini-1.5-flash';
        const hostname = 'generativelanguage.googleapis.com';
        const path = `/v1beta/models/${model}:generateContent?key=${GEMINI_API_KEY}`;

        const requestBody = JSON.stringify({
            contents: contents,
            generationConfig: {
                temperature: 0.7,
                maxOutputTokens: 500,
            }
        });

        const reply = await new Promise((resolve, reject) => {
            const req = https.request({
                hostname: hostname,
                path: path,
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Content-Length': Buffer.byteLength(requestBody)
                }
            }, (res) => {
                let responseData = '';
                res.on('data', (chunk) => { responseData += chunk; });
                res.on('end', () => {
                    if (res.statusCode >= 200 && res.statusCode < 300) {
                        try {
                            const data = JSON.parse(responseData);
                            const text = data.candidates?.[0]?.content?.parts?.[0]?.text || "I'm sorry, I couldn't generate a response.";
                            resolve(text);
                        } catch (e) {
                            reject(new Error(`Failed to parse response: ${e.message}`));
                        }
                    } else {
                        reject(new Error(`Gemini API Error: ${res.statusCode} ${responseData}`));
                    }
                });
            });

            req.on('error', (e) => {
                reject(new Error(`Request error: ${e.message}`));
            });

            req.write(requestBody);
            req.end();
        });

        return {
            statusCode: 200,
            headers,
            body: JSON.stringify({ reply }),
        };

    } catch (error) {
        console.error('Chat error:', error);
        return {
            statusCode: 500,
            headers,
            body: JSON.stringify({ error: 'Chat failed', details: error.message }),
        };
    }
};
