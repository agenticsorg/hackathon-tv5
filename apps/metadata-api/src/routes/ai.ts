import { Router, Request, Response } from 'express';

const router = Router();

// Gemini API configuration
const GEMINI_API_KEY = process.env.GEMINI_API_KEY;
const GEMINI_API_URL = 'https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent';

/**
 * POST /api/v1/ai/interpret-query
 * Uses Gemini to interpret natural language search queries
 * Extracts meaningful keywords for TMDb search
 */
router.post('/interpret-query', async (req: Request, res: Response) => {
  const { query } = req.body;

  if (!query || typeof query !== 'string') {
    return res.status(400).json({
      success: false,
      error: 'Query is required'
    });
  }

  // Check if API key is configured
  if (!GEMINI_API_KEY) {
    console.warn('GEMINI_API_KEY not configured, using fallback interpretation');
    return res.json({
      success: true,
      data: {
        original: query,
        interpreted: fallbackInterpret(query),
        method: 'fallback'
      }
    });
  }

  try {
    const prompt = `You are a search query interpreter for a movie and TV show discovery platform.

Given this natural language query: "${query}"

Extract the key search terms that would work best for searching a movie/TV database like TMDb.

Rules:
1. Remove conversational filler like "I want to see", "show me", "something about"
2. Extract the core topic, genre, or theme
3. If the query mentions a mood, map it to relevant genres
4. Keep it concise - just the essential keywords
5. Return ONLY the keywords, nothing else

Examples:
- "I want to see something about animals" → "animals wildlife nature"
- "show me scary movies" → "horror thriller"
- "looking for funny romantic movies" → "romantic comedy"
- "something with superheroes" → "superhero marvel dc"
- "I'm in the mood for action" → "action adventure"

Your response (keywords only):`;

    const response = await fetch(`${GEMINI_API_URL}?key=${GEMINI_API_KEY}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        contents: [{
          parts: [{
            text: prompt
          }]
        }],
        generationConfig: {
          temperature: 0.3,
          maxOutputTokens: 50,
          topP: 0.8
        }
      })
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error('Gemini API error:', response.status, errorText);

      // Fall back to rule-based interpretation
      return res.json({
        success: true,
        data: {
          original: query,
          interpreted: fallbackInterpret(query),
          method: 'fallback',
          reason: 'API error'
        }
      });
    }

    const data = await response.json();

    // Extract the generated text from Gemini response
    const generatedText = data.candidates?.[0]?.content?.parts?.[0]?.text?.trim();

    if (!generatedText) {
      return res.json({
        success: true,
        data: {
          original: query,
          interpreted: fallbackInterpret(query),
          method: 'fallback',
          reason: 'Empty response'
        }
      });
    }

    // Clean up the response (remove quotes, extra whitespace)
    const interpreted = generatedText
      .replace(/^["']|["']$/g, '')
      .replace(/\s+/g, ' ')
      .trim()
      .toLowerCase();

    return res.json({
      success: true,
      data: {
        original: query,
        interpreted: interpreted,
        method: 'gemini'
      }
    });

  } catch (error) {
    console.error('Gemini API error:', error);

    // Fall back to rule-based interpretation
    return res.json({
      success: true,
      data: {
        original: query,
        interpreted: fallbackInterpret(query),
        method: 'fallback',
        reason: 'Exception'
      }
    });
  }
});

/**
 * GET /api/v1/ai/status
 * Check if AI services are configured
 */
router.get('/status', (_req: Request, res: Response) => {
  res.json({
    success: true,
    data: {
      geminiConfigured: !!GEMINI_API_KEY,
      service: 'Gemini 1.5 Flash'
    }
  });
});

/**
 * Fallback rule-based interpretation when Gemini is unavailable
 */
function fallbackInterpret(query: string): string {
  const original = query.trim().toLowerCase();

  // Common filler phrases to remove
  const fillerPhrases = [
    'i want to see', 'i want to watch', 'i would like to see', 'i would like to watch',
    'show me', 'find me', 'looking for', 'search for', 'i am looking for',
    'something about', 'something with', 'something that has', 'something that is',
    'movies about', 'movies with', 'movies that', 'films about', 'films with',
    'tv shows about', 'shows about', 'series about', 'series with',
    'anything about', 'anything with', 'anything that',
    'can you find', 'can you show', 'please find', 'please show',
    'i need', 'i\'m in the mood for', 'in the mood for',
    'that is', 'that has', 'that are', 'which has', 'which are',
    'with a lot of', 'featuring', 'involving', 'related to'
  ];

  let cleaned = original;
  fillerPhrases.forEach(phrase => {
    cleaned = cleaned.replace(new RegExp(phrase, 'gi'), ' ');
  });

  // Remove common stop words
  const stopWords = ['a', 'an', 'the', 'is', 'are', 'was', 'were', 'be', 'been', 'being',
    'have', 'has', 'had', 'do', 'does', 'did', 'will', 'would', 'could', 'should',
    'may', 'might', 'must', 'shall', 'can', 'need', 'dare', 'ought', 'used',
    'to', 'of', 'in', 'for', 'on', 'with', 'at', 'by', 'from', 'as', 'into',
    'through', 'during', 'before', 'after', 'above', 'below', 'between',
    'and', 'but', 'or', 'nor', 'so', 'yet', 'both', 'either', 'neither',
    'not', 'only', 'own', 'same', 'than', 'too', 'very', 'just', 'also'];

  const words = cleaned.split(/\s+/).filter(word =>
    word.length > 1 && !stopWords.includes(word)
  );

  // Map common synonyms/concepts to TMDb-friendly search terms
  const conceptMap: Record<string, string> = {
    'animals': 'animal wildlife',
    'pets': 'pet dog cat',
    'space': 'space sci-fi',
    'scary': 'horror thriller',
    'funny': 'comedy',
    'romantic': 'romance love',
    'sad': 'drama emotional',
    'kids': 'family animation',
    'children': 'family animation',
    'superhero': 'superhero marvel dc',
    'mystery': 'mystery detective',
    'historical': 'history period drama',
    'war': 'war military',
    'crime': 'crime thriller',
    'magic': 'fantasy magic',
    'robots': 'robot sci-fi artificial intelligence',
    'aliens': 'alien sci-fi extraterrestrial',
    'zombies': 'zombie horror',
    'vampires': 'vampire horror',
    'dinosaurs': 'dinosaur prehistoric',
    'ocean': 'ocean sea underwater',
    'nature': 'nature documentary wildlife',
    'music': 'music musical',
    'sports': 'sports athletic',
    'cooking': 'cooking food culinary',
    'travel': 'travel adventure'
  };

  const expandedWords = words.map(word => conceptMap[word] || word);
  return expandedWords.join(' ').trim() || original;
}

export default router;
