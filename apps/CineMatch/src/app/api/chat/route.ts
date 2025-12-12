import { google } from '@ai-sdk/google';
import { openai } from '@ai-sdk/openai';
import { generateText, convertToCoreMessages } from 'ai';
import { NextResponse } from 'next/server';
import { searchMulti, getMovieDetails, getTVShowDetails } from '@/lib/tmdb';

// Allow streaming responses up to 30 seconds
export const maxDuration = 30;

export async function POST(req: Request) {
    const { messages, context } = await req.json();

    const preferences = context?.preferences || {};
    const likedContent = context?.likedContent || [];
    const dislikedContent = context?.dislikedContent || [];

    // Resolve favoriteMovieId to a title if present
    let favoriteMovieTitle = '';
    if (preferences.favoriteMovieId) {
        try {
            // Determine media type (default to movie if not specified)
            const mediaType = preferences.favoriteMediaType || 'movie';

            if (mediaType === 'movie') {
                const movie = await getMovieDetails(preferences.favoriteMovieId);
                favoriteMovieTitle = movie.title;
            } else if (mediaType === 'tv') {
                const show = await getTVShowDetails(preferences.favoriteMovieId);
                favoriteMovieTitle = show.name;
            }
        } catch (e) {
            console.error('Failed to fetch favorite content title:', e);
        }
    }

    const systemPrompt = `
    You are CineMatch AI, a helpful and enthusiastic movie recommendation assistant.
    
    User Context:
    - Preferences: ${JSON.stringify(preferences, (key, value) => {
        if (key === 'favoriteMovieId' || key === 'dislikedContentIds') return undefined; // Hide IDs to prevent hallucination
        return value;
    })}
    ${favoriteMovieTitle ? `- Favorite Content (from Onboarding): ${favoriteMovieTitle}` : ''}
    - Liked Movies (Titles): ${likedContent.length > 0 ? likedContent.join(', ') : 'None yet'}
    - Disliked Content (Count): ${dislikedContent.length} items (IDs hidden)

    Your Goal:
    - Recommend movies or TV shows based on the user's history and preferences.
    - Be concise, friendly, and engaging.
    - If the user asks "What should I watch?", suggest 2-3 titles that fit their taste, explaining why.
    - If "Liked Movies" is empty, YOU MUST USE their onboarding preferences (Favorite Content, Intent, Genres) to suggest 3 titles immediately.
    - Do NOT ask for more info unless absolutely necessary. Assume their onboarding choices are enough to start.
    - Do NOT recommend movies they have explicitly disliked.
    - Use emojis occasionally to keep it fun. 🍿🎬
    - IMPORTANT: Do NOT guess or hallucinate movie titles from IDs. Only refer to movies explicitly named in "Liked Movies" or "Favorite Content".
    
    TONE & STYLE:
    - Be natural and conversational. Avoid robotic lists of user preferences.
    - Instead of saying "Since you like [Genre] and want to [Intent]...", weave it in subtly like "If you're in the mood for a laugh..." or "For a gripping story..."
    - Sound like a friend giving a recommendation, not a database query result.
    
    RESPONSE FORMAT:
    You must return a JSON object with the following structure:
    {
        "message": "Your conversational response here...",
        "recommendations": [
            { "title": "Movie/Show Title", "year": "YYYY" },
            ...
        ]
    }
    Only include the "recommendations" array if you are explicitly suggesting specific movies. Otherwise, it can be empty.
    IMPORTANT: Do not output markdown code blocks (like \`\`\`json). Just output the raw JSON string.
    `;

    console.log('--- SYSTEM PROMPT ---\n', systemPrompt, '\n---------------------');

    // Choose model based on available keys (prefer OpenAI, fallback to Gemini)
    const model = process.env.OPENAI_API_KEY
        ? openai('gpt-4o-mini')
        : google('gemini-2.5-flash');

    try {
        const { text } = await generateText({
            model,
            system: systemPrompt,
            messages: convertToCoreMessages(messages),
            maxRetries: 0,
        });

        console.log('Raw AI Response:', text); // Debug log

        // Parse the JSON response from the AI
        let parsedResponse;
        try {
            // Clean up markdown code blocks if present
            const cleanText = text.replace(/```json/g, '').replace(/```/g, '').trim();

            // Find the first '{' and last '}' to extract JSON
            const jsonMatch = cleanText.match(/\{[\s\S]*\}/);
            if (jsonMatch) {
                parsedResponse = JSON.parse(jsonMatch[0]);
            } else {
                console.warn('No JSON found in response, using raw text');
                parsedResponse = { message: text, recommendations: [] };
            }
        } catch (e) {
            console.warn('Failed to parse AI JSON response, falling back to raw text:', e);
            parsedResponse = { message: text, recommendations: [] };
        }

        // Enrich recommendations with TMDB data
        const enrichedRecommendations = [];
        if (parsedResponse.recommendations && Array.isArray(parsedResponse.recommendations)) {
            console.log('Searching for recommendations:', parsedResponse.recommendations);
            for (const rec of parsedResponse.recommendations) {
                try {
                    // Search for the movie using our shared library
                    const { results } = await searchMulti(rec.title);

                    if (results && results.length > 0) {
                        // Filter for movie or tv (already done by searchMulti, but good to be safe)
                        // Optionally filter by year if provided
                        const match = results.find((r: any) => {
                            // Optional: Check year if we want to be strict
                            // if (rec.year && r.releaseDate) {
                            //     return r.releaseDate.startsWith(rec.year);
                            // }
                            return true;
                        });

                        if (match) {
                            enrichedRecommendations.push(match);
                        } else {
                            console.log(`No matching movie/tv found in results for ${rec.title}`);
                        }
                    } else {
                        console.log(`No results found for ${rec.title}`);
                    }
                } catch (err) {
                    console.error(`Failed to fetch details for ${rec.title}:`, err);
                }
            }
        }

        return NextResponse.json({
            message: parsedResponse.message,
            recommendations: enrichedRecommendations
        });
    } catch (error: any) {
        console.error('Chat generation error:', error);
        const errorMessage = error.message || 'Failed to generate response';
        const status = errorMessage.includes('Quota') || errorMessage.includes('429') ? 429 : 500;
        return NextResponse.json({ error: errorMessage }, { status });
    }
}
