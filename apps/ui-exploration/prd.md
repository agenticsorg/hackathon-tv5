

Extending our team brainstorm I ended up into the following ideas that could be answer to the Hackathon's challenge. Let me approach it zoomed out, with a brand strategy perspective:
- step 1: platforms should position its brand. Offer movies to already filtered audience (peeps know what they expect)
- reduced decision fatigue - we are halfway solved the problem
- step 2, next stage: communicate how the given media platform is DIFFERENT than the competition (example "we've got the best rom coms") -
- the positioning needs to be solved, not just serve ANY audience segment
- before the algorithms kick in, do manual curation
- but as we discussed, we don't have user profiles after they signed up (cold start). the key, we don't need to categorize them IMMEdiately. content-based filtering remains underutilized. ColdRAG could be a solution too, no questionnaires needed
- first, we group them by asking few simple questions (fav actor, genre etc)
- we now have segments, then monitor the views and likes and fine tune the algo - not YET personalised, but segmented (these steps are potentally obvious for the team :)  
- next question: how do we avoid user churn? we need feedback loop (but not star reviews)
- we might not show the complete media library, just a a few recommendation
- we also display manual curation on the UI - this is the differentiator. Trust kicks in
The above use case is for a niche platform, not serving millions of people. Hybrid data/human curation.

To move away from legacy metadata providers, we need semantic understanding. This is where out embeddings-first approach comes in:
- the missing items are: thematic depth, style, cinematography, mood, tone, emotional trajectory etc
- we miss explainability: Users want to know WHY something is recommended
- 3 embedding approaches: 1. Emotional arc embeddings (rags to riches, etc) "find content with a similar emotional journey" discovery https://github.com/nchibana/moviearcs
- 2. Theme extraction with LLMs - Neo4j: Movie → Theme → Stem knowledge graphs
- 3. recco on visual similarity of thumbnails: embeddings of posters/still frames - colour palette, composition, visual mood etc.
- we are working already on our  Zero-Shot Mood-Based Discovery!
- "I want something visually stunning and emotionally complex"
- for thumbnails, TMDB library is not enough, we need stills/actual frames
- what's innovative could be the json schema based prompting with Gemini Pro 3 (nano banana)

I tested and it's working almost pixel perfect. Segment: Romance-seekers
→ Emphasize: Lead chemistry, warm color palette, intimate framing
→ De-emphasize: Action sequences, ensemble cast shots

Segment: Thriller fans  
→ Emphasize: Tension moments, cool/dark palette, isolated protagonist
→ De-emphasize: Romantic subplot, comedic beats

just gemini 3 pro demo: /**
 * Gemini 3 Pro - Movie Thumbnail Generator
 * Creates audience-segmented movie promotional images
 *
 * Segment 1: Romance-seekers (warm palette, intimate framing, chemistry)
 * Segment 2: Thriller fans (cool/dark palette, tension, isolated protagonist)
 */

require('dotenv').config();
const fs = require('fs');
const {
  generateImageWithGemini,
  decodeBase64Image,
  extractImageFromResponse,
  sleep
} = require('./gemini-api-helper');

const OUTPUT_DIR = './output-gemini-movie-thumbnails';

// Ensure output directory exists
if (!fs.existsSync(OUTPUT_DIR)) {
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
}

// ============================================
// SEGMENT 1: ROMANCE-SEEKERS
// ============================================
const romancePrompts = [
  {
    name: "romance-couple-sunset",
    prompt: {
      scene: "Two people standing close on a rooftop at golden hour",
      subjects: {
        lead1: "Man in casual blazer, looking tenderly at partner",
        lead2: "Woman in flowing dress, hand touching his chest"
      },
      composition: {
        framing: "Medium close-up, intimate two-shot",
        focus: "Eyes and hands, soft focus background",
        rule: "Faces slightly angled toward each other"
      },
      lighting: {
        type: "Golden hour backlight with soft fill",
        quality: "Warm, dreamy, lens flare touches"
      },
      color_palette: {
        primary: "Warm amber, soft coral, dusty rose",
        accent: "Champagne gold highlights",
        mood: "Romantic, inviting, emotionally warm"
      },
      style: "Cinematic romance movie poster aesthetic, shallow depth of field",
      exclude: "Action elements, violence, dark shadows, blue tones"
    }
  },
  {
    name: "romance-rain-umbrella",
    prompt: {
      scene: "Two people sharing an umbrella in light rain on a city street",
      subjects: {
        lead1: "Man holding umbrella, looking down at partner with warmth",
        lead2: "Woman pressed close, laughing, looking up"
      },
      composition: {
        framing: "Full body with romantic city backdrop",
        focus: "The space between them, shared intimacy",
        rule: "Umbrella creates intimate frame"
      },
      lighting: {
        type: "Soft street lights creating warm bokeh",
        quality: "Gentle rain catching light, romantic atmosphere"
      },
      color_palette: {
        primary: "Warm yellows from streetlights, soft peach",
        accent: "Reflected warm tones in wet pavement",
        mood: "Cozy, nostalgic, sweetly romantic"
      },
      style: "Classic romance film still, European cinema feel",
      exclude: "Cold blue tones, harsh lighting, action"
    }
  },
  {
    name: "romance-cafe-moment",
    prompt: {
      scene: "Two people at a small cafe table, hands almost touching",
      subjects: {
        lead1: "Person leaning forward, genuine smile",
        lead2: "Person looking down shyly, soft expression"
      },
      composition: {
        framing: "Close-up on hands and faces, intimate crop",
        focus: "The moment of connection, anticipation",
        rule: "Negative space suggests what's unspoken"
      },
      lighting: {
        type: "Warm cafe ambient light, candle glow",
        quality: "Soft, flattering, romantic"
      },
      color_palette: {
        primary: "Warm burgundy, cream, caramel",
        accent: "Soft rose and copper",
        mood: "First date energy, butterflies, warmth"
      },
      style: "Indie romance film aesthetic, natural and authentic",
      exclude: "Bright colors, action, thriller elements"
    }
  }
];

TikTok doesn't ask. It shows you content and measures your behavior at millisecond granularity. What made me think, Patrice said too (just spotted Mondweep's transcript): Metadata is commoditized. Everyone has identical inputs. The transformation layer is the entire game.

[2:29 pm, 06/12/2025] Bence: so moving away from librarian brain to 'salesperson' or movie curator brain: "Position this as escapism for stressed professionals"
"Surface this when user has 90min window"
"Lead with actor X for segment Y, director Z for segment W"
[2:32 pm, 06/12/2025] Bence: What if we applied tiktok's method, we'd show curated clips (trailers lie), we'd measure watch-through, replay, skip speed, 10 clips are enough
[2:32 pm, 06/12/2025] Bence: or even purely thumbnails  > so under a minute, we nailed the cold start problem
[2:33 pm, 06/12/2025] Bence: YouTube has trailer/clip data API
[2:36 pm, 06/12/2025] Bence: the learning: embedding the clips + tracking 
winner: 
- demoable
- can be done hybrid thumbnail generation and using exsiting video data etc
- solves the hackathon's original problem  (2 hours down to 2 mins)
- new idea (tiktok style applied to movie discovery)
- sellable - TV5 can't do this themselves
- it enriches inputs, not replaces outputs
- no proprietary data needed
- Demonstrable value — show same movie, different contexts, different positioning

