import { GoogleGenerativeAI, GenerativeModel } from '@google/generative-ai';

export interface GeminiEmotionResponse {
  valence: number;      // -1 to 1
  arousal: number;      // -1 to 1
  stress: number;       // 0 to 1
  dominantEmotion: string;
  plutchikEmotions: {
    joy: number;
    trust: number;
    fear: number;
    surprise: number;
    sadness: number;
    disgust: number;
    anger: number;
    anticipation: number;
  };
  confidence: number;
}

const EMOTION_PROMPT = `Analyze the emotional content of the following text and respond with a JSON object containing:
- valence: a number from -1 (very negative) to 1 (very positive)
- arousal: a number from -1 (very calm) to 1 (very excited)
- stress: a number from 0 (no stress) to 1 (high stress)
- dominantEmotion: the primary emotion detected (e.g., "joy", "anxiety", "sadness")
- plutchikEmotions: object with values 0-1 for each of Plutchik's 8 basic emotions (joy, trust, fear, surprise, sadness, disgust, anger, anticipation)
- confidence: a number from 0-1 indicating confidence in the analysis

Respond ONLY with valid JSON, no other text.

Text to analyze:
`;

/**
 * Gemini API client for emotion detection
 */
export class GeminiClient {
  private model: GenerativeModel | null = null;
  private readonly maxRetries = 3;
  private readonly timeout = 30000;

  constructor() {
    const apiKey = process.env.GEMINI_API_KEY;
    if (apiKey) {
      const genAI = new GoogleGenerativeAI(apiKey);
      this.model = genAI.getGenerativeModel({ model: 'gemini-2.0-flash-exp' });
    }
  }

  isAvailable(): boolean {
    return this.model !== null;
  }

  async analyzeEmotion(text: string): Promise<GeminiEmotionResponse> {
    if (!this.model) {
      throw new Error('Gemini API key not configured');
    }

    let lastError: Error | null = null;

    for (let attempt = 1; attempt <= this.maxRetries; attempt++) {
      try {
        const result = await this.callWithTimeout(text);
        return result;
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));

        if (attempt < this.maxRetries) {
          // Exponential backoff
          const delay = Math.pow(2, attempt) * 1000;
          await this.sleep(delay);
        }
      }
    }

    throw lastError || new Error('Failed to analyze emotion');
  }

  private async callWithTimeout(text: string): Promise<GeminiEmotionResponse> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const prompt = EMOTION_PROMPT + text;

      const result = await Promise.race([
        this.model!.generateContent({
          contents: [{ role: 'user', parts: [{ text: prompt }] }],
          generationConfig: { temperature: 0.3 }
        }),
        new Promise<never>((_, reject) => {
          setTimeout(() => reject(new Error('Request timeout')), this.timeout);
        })
      ]);

      clearTimeout(timeoutId);

      const response = result.response;
      const responseText = response.text();

      // Parse JSON from response
      const jsonMatch = responseText.match(/\{[\s\S]*\}/);
      if (!jsonMatch) {
        throw new Error('No JSON found in response');
      }

      const parsed = JSON.parse(jsonMatch[0]) as GeminiEmotionResponse;
      return this.validateAndNormalize(parsed);
    } finally {
      clearTimeout(timeoutId);
    }
  }

  private validateAndNormalize(response: GeminiEmotionResponse): GeminiEmotionResponse {
    return {
      valence: this.clamp(response.valence, -1, 1),
      arousal: this.clamp(response.arousal, -1, 1),
      stress: this.clamp(response.stress, 0, 1),
      dominantEmotion: response.dominantEmotion || 'neutral',
      plutchikEmotions: {
        joy: this.clamp(response.plutchikEmotions?.joy || 0, 0, 1),
        trust: this.clamp(response.plutchikEmotions?.trust || 0, 0, 1),
        fear: this.clamp(response.plutchikEmotions?.fear || 0, 0, 1),
        surprise: this.clamp(response.plutchikEmotions?.surprise || 0, 0, 1),
        sadness: this.clamp(response.plutchikEmotions?.sadness || 0, 0, 1),
        disgust: this.clamp(response.plutchikEmotions?.disgust || 0, 0, 1),
        anger: this.clamp(response.plutchikEmotions?.anger || 0, 0, 1),
        anticipation: this.clamp(response.plutchikEmotions?.anticipation || 0, 0, 1),
      },
      confidence: this.clamp(response.confidence || 0.5, 0, 1),
    };
  }

  private clamp(value: number, min: number, max: number): number {
    return Math.max(min, Math.min(max, value));
  }

  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}
