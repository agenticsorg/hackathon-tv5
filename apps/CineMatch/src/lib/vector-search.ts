/* eslint-disable @typescript-eslint/no-explicit-any */
import { semanticSearch, findSimilarContent } from '@/lib/vector-db';
import { discoverMovies, discoverTVShows, getTrending, getMovieDetails, getTVShowDetails } from '@/lib/tmdb';
import type { MediaContent } from '@/types/media';
import type { UserPreferences } from '@/store/match-store';

// ---- CONFIG ----
const CANDIDATE_POOL_SIZE = 60;
const FINAL_K = 12;
const LINUCB_ALPHA = 0.7;
const SMALL_FEATURE_GENRE_DIM = 16; // consistent with your vector helpers
const USER_SMALL_DIM = 10 + SMALL_FEATURE_GENRE_DIM; // contentType/era/age/likes + top genre summary
// Note: final LinUCB dim = USER_SMALL_DIM + ITEM_SMALL_DIM
const ITEM_SMALL_DIM = 3 + SMALL_FEATURE_GENRE_DIM; // vote, pop, yearNorm + genres

// ---- In-memory bandit store (persist externally for prod) ----
type BanditState = {
  A: number[][]; // d x d
  b: number[];   // d
  lastUpdated: number;
};
const BANDIT_STORE: Record<string, BanditState> = {}; // key = userId

// ---- Linear algebra helpers (small dims) ----
function zeros(n: number) { return Array.from({ length: n }, () => 0); }
function eye(n: number) {
  return Array.from({ length: n }, (_, i) => Array.from({ length: n }, (_, j) => (i === j ? 1 : 0)));
}
function matAdd(A: number[][], B: number[][]) {
  return A.map((row, i) => row.map((v, j) => v + B[i][j]));
}
function vecAdd(a: number[], b: number[]) { return a.map((v, i) => v + b[i]); }
function matVecMul(A: number[][], v: number[]) { return A.map(row => row.reduce((s, x, i) => s + x * v[i], 0)); }
function vecOuter(a: number[], b: number[]) { return a.map(ai => b.map(bj => ai * bj)); }
// matrix inverse (Gaussian elimination) - ok for small dims (<128)
function invertMatrix(A: number[][]) {
  const n = A.length;
  const M = A.map((r, i) => r.slice().concat(eye(n)[i]));
  for (let i = 0; i < n; i++) {
    let pivot = M[i][i];
    if (Math.abs(pivot) < 1e-12) {
      let swap = i + 1;
      while (swap < n && Math.abs(M[swap][i]) < 1e-12) swap++;
      if (swap === n) throw new Error('Singular matrix in invertMatrix');
      const tmp = M[i]; M[i] = M[swap]; M[swap] = tmp;
      pivot = M[i][i];
    }
    for (let j = i; j < 2 * n; j++) M[i][j] /= pivot;
    for (let r = 0; r < n; r++) {
      if (r === i) continue;
      const factor = M[r][i];
      for (let c = i; c < 2 * n; c++) M[r][c] -= factor * M[i][c];
    }
  }
  return M.map(row => row.slice(n));
}

// ---- Feature builders ----
function buildItemSmallFeatures(item: MediaContent): number[] {
  const vote = (item.voteAverage ?? 0) / 10;
  const pop = Math.tanh((item.popularity ?? 0) / 100);
  const year = parseInt((item.releaseDate || '0').split('-')[0] || '0') || 0;
  const yearNorm = year > 1900 ? Math.min((year - 1900) / 150, 1) : 0;

  const genreVec = new Array(SMALL_FEATURE_GENRE_DIM).fill(0);
  (item.genreIds || []).forEach(g => { genreVec[g % SMALL_FEATURE_GENRE_DIM] += 1; });
  const gnorm = Math.sqrt(genreVec.reduce((s, x) => s + x * x, 0)) || 1;
  for (let i = 0; i < genreVec.length; i++) genreVec[i] /= gnorm;

  return [vote, pop, yearNorm, ...genreVec].slice(0, ITEM_SMALL_DIM); // ensure fixed length
}

function buildUserSmallFeatures(prefs: any, totalLikes: number, genreWeights: Record<number, number>) {
  // contentType one-hot (movie,tv,animation,anime,spectacle,short_film) -> 6 dims
  const types = ['movie', 'tv', 'animation', 'anime', 'spectacle', 'short_film'];
  const typeOneHot = types.map(t => (prefs.contentType === t ? 1 : 0));

  const eraVec = [prefs.era === 'new' ? 1 : 0, prefs.era === 'classic' ? 1 : 0, (!prefs.era) ? 1 : 0];
  const ageBuckets = ['under_18', '18_30', '30_50', '50_plus'];
  const ageVec = ageBuckets.map(b => prefs.age === b ? 1 : 0);

  const likesNorm = Math.tanh(totalLikes / 10);

  const topGenres = Object.entries(genreWeights || {}).sort((a, b) => Number(b[1]) - Number(a[1])).slice(0, 4).map(([id]) => Number(id));
  const genreVec = new Array(SMALL_FEATURE_GENRE_DIM).fill(0);
  topGenres.forEach((g, i) => { genreVec[g % SMALL_FEATURE_GENRE_DIM] += (i === 0 ? 1 : 0.6); });
  const gnorm = Math.sqrt(genreVec.reduce((s, x) => s + x * x, 0)) || 1;
  for (let i = 0; i < genreVec.length; i++) genreVec[i] /= gnorm;

  // assemble and limit to USER_SMALL_DIM
  const vec = [...typeOneHot.slice(0, 6), ...eraVec.slice(0, 3), ...ageVec.slice(0, 4), likesNorm];
  // pad/trim then add genreVec
  const base = vec.concat(genreVec).slice(0, USER_SMALL_DIM);
  if (base.length < USER_SMALL_DIM) {
    return base.concat(Array(USER_SMALL_DIM - base.length).fill(0));
  }
  return base;
}

// ---- Bandit helpers ----
function ensureBanditState(userId: string, dim: number) {
  if (!BANDIT_STORE[userId]) {
    BANDIT_STORE[userId] = {
      A: eye(dim),
      b: zeros(dim),
      lastUpdated: Date.now(),
    };
  } else if (BANDIT_STORE[userId].A.length !== dim) {
    BANDIT_STORE[userId] = { A: eye(dim), b: zeros(dim), lastUpdated: Date.now() };
  }
}

function linUCBScore(userId: string, x: number[]) {
  const state = BANDIT_STORE[userId];
  const Ainv = invertMatrix(state.A);
  const theta = matVecMul(Ainv, state.b);
  const xTtheta = x.reduce((s, xi, i) => s + xi * theta[i], 0);
  const Ax = matVecMul(Ainv, x);
  const xTAx = x.reduce((s, xi, i) => s + xi * Ax[i], 0);
  const u = LINUCB_ALPHA * Math.sqrt(Math.max(0, xTAx));
  return xTtheta + u;
}

function linUCBUpdate(userId: string, x: number[], reward: number) {
  const state = BANDIT_STORE[userId];
  const outer = vecOuter(x, x);
  state.A = matAdd(state.A, outer);
  state.b = vecAdd(state.b, x.map(v => v * reward));
  state.lastUpdated = Date.now();
}

// ---- Recommendation pipeline ----
/**
 * recommendForUser
 * - prefs: user preferences object (from your Zustand store)
 * - userId: persistent id string
 * - totalLikes / genreWeights: from store
 * - page: current paging
 */
export async function recommendForUser(
  userId: string,
  prefs: any,
  totalLikes: number,
  genreWeights: Record<number, number>,
  page = 1,
  k = FINAL_K
): Promise<MediaContent[]> {
  // 1) Determine Context (Countries, Genres)
  const userCountry = prefs.userCountry || 'FR';
  let favoriteCountries: string[] = [];

  // Get favorite country from favorite content
  if (prefs.favoriteMovieId) {
    try {
      const favId = prefs.favoriteMovieId;
      const favType = prefs.favoriteMediaType || (prefs.contentType === 'tv' ? 'tv' : 'movie');
      const favContent = favType === 'tv'
        ? await getTVShowDetails(favId)
        : await getMovieDetails(favId);

      if (favContent.originCountry && favContent.originCountry.length > 0) {
        favoriteCountries = favContent.originCountry;
      } else if (favContent.originalLanguage) {
        const langMap: Record<string, string[]> = {
          'en': ['US', 'GB'], 'ja': ['JP'], 'ko': ['KR'], 'fr': ['FR'], 'es': ['ES', 'MX'], 'de': ['DE'], 'it': ['IT']
        };
        favoriteCountries = langMap[favContent.originalLanguage] || [];
      }
    } catch (e) {
      console.warn('Failed to fetch favorite content details', e);
    }
  }

  // Top Genres
  const topGenres = Object.entries(genreWeights || {})
    .sort((a, b) => Number(b[1]) - Number(a[1]))
    .slice(0, 3)
    .map(([id]) => Number(id));

  // 2) Candidate Generation (Multi-Source)
  const candidates: MediaContent[] = [];
  const isTV = prefs.contentType === 'tv';
  const discoverFn = isTV ? discoverTVShows : discoverMovies;

  try {
    // Source A: User Country (High Priority)
    // Fetch more pages to ensure we have enough candidates from the selected country
    const userCountryContent = await discoverFn({
      withOriginCountry: userCountry,
      page: page,
      sortBy: 'popularity.desc'
    });
    candidates.push(...userCountryContent.results);

    // Source B: Favorite Country (if different from User Country)
    for (const favCountry of favoriteCountries) {
      if (favCountry !== userCountry) {
        const favCountryContent = await discoverFn({
          withOriginCountry: favCountry,
          page: page,
          sortBy: 'popularity.desc'
        });
        candidates.push(...favCountryContent.results);
      }
    }

    // Source C: Top Genres (Global)
    if (topGenres.length > 0) {
      const genreContent = await discoverFn({
        genres: [topGenres[0]],
        page: page,
        sortBy: 'popularity.desc'
      });
      candidates.push(...genreContent.results);
    }

    // Source D: Trending/Popular (Baseline)
    const trending = await getTrending(isTV ? 'tv' : 'movie', 'week');
    candidates.push(...trending);

  } catch (e) {
    console.error('Error fetching candidates:', e);
    // Fallback to trending
    const trending = await getTrending(isTV ? 'tv' : 'movie', 'week');
    candidates.push(...trending);
  }

  // Deduplicate by ID
  const seen = new Set<number>();
  const uniqCandidates = candidates.filter(c => c && !seen.has(c.id) ? (seen.add(c.id), true) : false);

  if (uniqCandidates.length === 0) return [];

  // 3) Scoring & Filtering
  const userSmall = buildUserSmallFeatures(prefs, totalLikes, genreWeights);
  const d = userSmall.length + ITEM_SMALL_DIM;
  ensureBanditState(userId, d);

  const scored = uniqCandidates.map(item => {
    // Base LinUCB Score (Personalization)
    const itemSmall = buildItemSmallFeatures(item);
    const x = userSmall.concat(itemSmall);
    let score = linUCBScore(userId, x);

    // Content-Based Boosting (Metadata)
    const itemCountries = item.originCountry && item.originCountry.length > 0
      ? item.originCountry
      : (item.originalLanguage === 'en' ? ['US', 'GB'] : [item.originalLanguage.toUpperCase()]);

    const isFavCountry = favoriteCountries.some(c => itemCountries.includes(c));
    const isUserCountry = userCountry && itemCountries.includes(userCountry);
    const isAnime = prefs.contentType === 'anime' || item.genreIds?.includes(16); // 16 is Animation

    // Boosts
    if (isFavCountry) score += 1.5; // Strongest boost for favorite content match
    if (isUserCountry) score += 1.0; // Strong boost for selected country
    if (topGenres.some(g => item.genreIds?.includes(g))) score += 0.5; // Genre boost

    // Penalties
    if (!isAnime) {
      if (!isFavCountry && !isUserCountry) {
        // Severe penalty for content from random countries
        score -= 5.0;
      }
    }

    return { item, score, x };
  });

  // 4) Sort & Select
  scored.sort((a, b) => b.score - a.score);

  const final: MediaContent[] = [];
  const usedIds = new Set<number>();

  for (const candidate of scored) {
    if (final.length >= k) break;
    if (usedIds.has(candidate.item.id)) continue;

    // Strict Filter Check
    if (!passesStrictFilter(candidate.item, prefs)) continue;

    final.push(candidate.item);
    usedIds.add(candidate.item.id);
  }

  // 5) Diversity (Distinct Years)
  const distinct: MediaContent[] = [];
  const seenYears = new Set<number>();
  const rest: MediaContent[] = [];

  for (const it of final) {
    const year = Number((it.releaseDate || '').split('-')[0]) || 0;
    if (distinct.length < 10) {
      if (year > 0 && !seenYears.has(year)) {
        distinct.push(it);
        seenYears.add(year);
      } else {
        rest.push(it);
      }
    } else {
      rest.push(it);
    }
  }

  return [...distinct, ...rest].slice(0, k);
}

// ---- Feedback recording: call on like/dislike to update bandit ----
export async function recordFeedback(
  userId: string,
  prefs: any,
  totalLikes: number,
  genreWeights: Record<number, number>,
  item: MediaContent,
  reward: number
) {
  const userSmall = buildUserSmallFeatures(prefs, totalLikes, genreWeights);
  const itemSmall = buildItemSmallFeatures(item);
  const x = userSmall.concat(itemSmall);
  ensureBanditState(userId, x.length);
  linUCBUpdate(userId, x, reward);
  // Optionally persist BANDIT_STORE[userId] to DB here
  return true;
}

// ---- Strict filter util (same logic as your UI) ----
function passesStrictFilter(item: MediaContent, prefs: any) {
  if (!prefs?.contentType) return true;
  switch (prefs.contentType) {
    case 'anime':
      if (!item.genreIds?.includes(16) || item.originalLanguage !== 'ja') return false;
      break;
    case 'animation':
      if (!item.genreIds?.includes(16) || ['ja', 'ko'].includes(item.originalLanguage)) return false;
      break;
    case 'tv':
      if (item.mediaType !== 'tv' || item.genreIds?.includes(16)) return false;
      break;
    case 'movie':
      if (item.mediaType !== 'movie' || item.genreIds?.includes(16)) return false;
      break;
    case 'spectacle':
      if (!item.genreIds?.some(id => [35, 10402].includes(id))) return false;
      break;
  }
  if (prefs.era === 'new') {
    const year = Number((item.releaseDate || '').split('-')[0]) || 0;
    if (year && year < 2015) return false;
  } else if (prefs.era === 'classic') {
    const year = Number((item.releaseDate || '').split('-')[0]) || 9999;
    if (year && year >= 2015) return false;
  }
  return true;
}
