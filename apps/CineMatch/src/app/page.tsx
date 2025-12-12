'use client';

import { useEffect, useState, useRef } from 'react';
import { localDB } from '@/lib/local-db';
import { useMatchStore } from '@/store/match-store';
import { Onboarding } from '@/components/Onboarding';
import { SwipeCard } from '@/components/SwipeCard';
import { DetailsModal } from '@/components/DetailsModal';
import { CountrySelector } from '@/components/CountrySelector';
import { AnimatePresence } from 'framer-motion';
import { getSimilarMovies, getSimilarTVShows, getTrending, discoverMovies, discoverTVShows } from '@/lib/tmdb';

import Link from 'next/link';
import { Info, RotateCcw, History, MessageSquare } from 'lucide-react';

export default function Home() {
  const {
    preferences,
    recommendations,
    currentIndex,
    page,
    lastLikedContentId, // Added lastLikedContentId
    setPreferences,     // Added setPreferences
    setRecommendations,
    appendRecommendations,
    likeContent,
    dislikeContent,
    nextCard,
    reset,
    incrementPage, // Added incrementPage
  } = useMatchStore();

  const [isLoading, setIsLoading] = useState(false);
  const [isFetchingMore, setIsFetchingMore] = useState(false);
  const [isMounted, setIsMounted] = useState(false);
  const [showDetails, setShowDetails] = useState(false);

  useEffect(() => {
    setIsMounted(true);
  }, []);

  // Strict Content Filter
  const applyStrictFilter = (results: any[], prefs: typeof preferences) => {
    if (!prefs.contentType) return results;

    return results.filter(item => {
      // 1. Content Type & Language/Genre Exclusions
      switch (prefs.contentType) {
        case 'anime':
          // Must be Animation (16) AND Japanese (ja)
          if (!item.genreIds?.includes(16) || item.originalLanguage !== 'ja') return false;
          break;
        case 'animation':
          // Must be Animation (16) AND NOT Japanese/Korean
          if (!item.genreIds?.includes(16) || ['ja', 'ko'].includes(item.originalLanguage)) return false;
          break;
        case 'tv':
          // Must be TV AND NOT Animation (16)
          if (item.mediaType !== 'tv' || item.genreIds?.includes(16)) return false;
          break;
        case 'movie':
          // Must be Movie AND NOT Animation (16)
          if (item.mediaType !== 'movie' || item.genreIds?.includes(16)) return false;
          break;
        case 'spectacle':
          // Must be Comedy (35) or Music (10402)
          if (!item.genreIds?.some((id: number) => [35, 10402].includes(id))) return false;
          break;
      }

      // 2. Era Filtering
      // 2. Era Filtering
      if (prefs.era === 'new') {
        // New = After 2014 (2015+)
        // New = After 2014 (2015+)
        const yearStr = item.release_date?.split('-')[0] || item.first_air_date?.split('-')[0] || item.releaseDate?.split('-')[0] || item.firstAirDate?.split('-')[0] || item.releaseYear || '0';
        const year = parseInt(yearStr);
        if (year < 2015) {
          console.log(`[StrictFilter] Dropped ${item.title || item.name} (${year}) - Too Old for New Era`);
          return false;
        }
      } else if (prefs.era === 'classic') {
        // Archives = Before 2015 (<= 2014)
        // Archives = Before 2015 (<= 2014)
        const yearStr = item.release_date?.split('-')[0] || item.first_air_date?.split('-')[0] || item.releaseDate?.split('-')[0] || item.firstAirDate?.split('-')[0] || item.releaseYear || '9999';
        const year = parseInt(yearStr);
        if (year >= 2015) {
          console.log(`[StrictFilter] Dropped ${item.title || item.name} (${year}) - Too New for Classic Era`);
          return false;
        }
      }

      return true;
    });
  };


  // Helper to fetch recommendations
  // Helper to fetch recommendations
  const fetchRecommendations = async (pageNum: number) => {
    const { totalLikes, genreWeights } = useMatchStore.getState();

    // We need a persistent user ID. For now, we'll generate one if not present in localStorage
    // In a real app, this would come from auth.
    let userId = localStorage.getItem('movie-match-user-id');
    if (!userId) {
      userId = crypto.randomUUID();
      localStorage.setItem('movie-match-user-id', userId);
    }

    const response = await fetch('/api/recommend', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        userId,
        preferences,
        totalLikes,
        genreWeights,
        page: pageNum
      })
    });

    if (!response.ok) {
      throw new Error('Failed to fetch recommendations');
    }

    const data = await response.json();
    return data.results;
  };

  // Feedback wrapper
  const handleFeedback = async (item: any, reward: number) => {
    const { totalLikes, genreWeights } = useMatchStore.getState();
    let userId = localStorage.getItem('movie-match-user-id');
    if (!userId) return; // Should exist by now

    try {
      await fetch('/api/feedback', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          userId,
          preferences,
          totalLikes,
          genreWeights,
          item,
          reward
        })
      });
    } catch (e) {
      console.error('Failed to record feedback', e);
    }
  };

  // Initial fetch after onboarding
  useEffect(() => {
    const init = async () => {
      if (isMounted && preferences.isOnboarded && recommendations.length === 0) {
        setIsLoading(true);
        try {
          const results = await fetchRecommendations(1);
          setRecommendations(results);
        } catch (error) {
          console.error('Failed to fetch recommendations:', error instanceof Error ? error.message : JSON.stringify(error));
        } finally {
          setIsLoading(false);
        }
      }
    };
    init();
  }, [isMounted, preferences.isOnboarded, recommendations.length, setRecommendations]);

  const [hasMore, setHasMore] = useState(true);
  const isFetchingRef = useRef(false);

  // Infinite Scroll: Fetch more when running low
  useEffect(() => {
    const checkAndFetchMore = async () => {
      const remainingCards = recommendations.length - currentIndex;

      // Only fetch if:
      // 1. We have few cards left (< 3)
      // 2. We are not already fetching (check ref)
      // 3. We have initial recommendations (length > 0)
      // 4. We haven't reached the page limit (50)
      // 5. We believe there is more content (hasMore)
      if (isMounted && remainingCards < 3 && !isFetchingRef.current && recommendations.length > 0 && page < 50 && hasMore) {
        isFetchingRef.current = true;
        setIsFetchingMore(true);
        console.log('Fetching more recommendations... Page:', page + 1);
        try {
          const newResults = await fetchRecommendations(page + 1);

          // Stop if no new results found
          if (newResults.length === 0) {
            console.log('No more results found from API.');
            setHasMore(false);
            return;
          }

          // Filter out duplicates
          const existingIds = new Set(recommendations.map(r => r.id));
          const filteredNewResults = newResults.filter((r: any) => !existingIds.has(r.id));

          if (filteredNewResults.length > 0) {
            appendRecommendations(filteredNewResults);
            // Note: appendRecommendations already increments the page in the store
          } else {
            console.log('All fetched results were duplicates. Stopping fetch to prevent loop.');
            setHasMore(false);
          }
        } catch (error) {
          console.error('Failed to fetch more recommendations:', error instanceof Error ? error.message : JSON.stringify(error));
        } finally {
          setIsFetchingMore(false);
          isFetchingRef.current = false;
        }
      }
    };

    checkAndFetchMore();
  }, [isMounted, currentIndex, recommendations.length, page, appendRecommendations, hasMore]);

  if (!isMounted) {
    return null; // Prevent hydration mismatch
  }

  if (!preferences.isOnboarded) {
    return (
      <main className="min-h-screen bg-black flex items-center justify-center overflow-hidden">
        <Onboarding />
      </main>
    );
  }

  if (isLoading) {
    return (
      <main className="min-h-screen bg-black flex items-center justify-center">
        <div className="animate-pulse text-white text-xl font-bold">Finding your match...</div>
      </main>
    );
  }

  const currentCard = recommendations[currentIndex];

  if (!currentCard) {
    // Check if we've reached the end
    if (!hasMore || page >= 50) {
      return (
        <main className="min-h-screen bg-black flex items-center justify-center text-white p-6 text-center">
          <div className="max-w-md">
            <h2 className="text-3xl font-bold mb-4 text-transparent bg-clip-text bg-gradient-to-r from-pink-500 to-purple-500">
              You've seen it all!
            </h2>
            <p className="text-gray-400 mb-8">
              We've run out of recommendations based on your current preferences.
            </p>
            <button
              onClick={() => {
                reset();
                window.location.reload();
              }}
              className="px-8 py-3 bg-white text-black rounded-full font-bold hover:scale-105 transition-transform"
            >
              Start Over
            </button>
          </div>
        </main>
      );
    }

    return (
      <main className="min-h-screen bg-black flex items-center justify-center text-white p-6 text-center">
        <div>
          <h2 className="text-3xl font-bold mb-4">Loading more...</h2>
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-white mx-auto"></div>
        </div>
      </main>
    );
  }


  const handleReset = () => {
    reset();
    window.location.reload(); // Hard reload to clear any local state/cache issues
  };

  return (
    <main className="min-h-screen bg-black overflow-hidden flex flex-col">
      {/* Header */}
      <header className="p-4 flex justify-between items-center z-10">
        <CountrySelector />
        <div className="flex items-center gap-2">
          <img src="/logo.png" alt="Logo" className="w-8 h-8 mb-1" />
          <h1 className="text-2xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-pink-500 to-purple-500">
            CineMatch
          </h1>
        </div>
        <button
          onClick={reset}
          className="p-3 rounded-full bg-white/10 hover:bg-white/20 transition-colors backdrop-blur-sm border border-white/10"
          aria-label="Reset"
        >
          <RotateCcw className="w-6 h-6 text-white" />
        </button>
      </header>

      {/* Card Deck */}
      <div className="flex-1 relative flex items-center justify-center p-4 max-w-md mx-auto w-full">
        <div className="relative w-full aspect-[2/3]">
          <AnimatePresence>
            <SwipeCard
              key={currentCard.id}
              content={currentCard}
              onSwipeLeft={() => {
                dislikeContent(currentCard.id);
                handleFeedback(currentCard, 0);
              }}
              onSwipeRight={() => {
                likeContent(currentCard);
                handleFeedback(currentCard, 1);
              }}
              onSwipeUp={() => setShowDetails(true)}
            />
          </AnimatePresence>
        </div>
      </div>

      {/* Controls */}
      <div className="p-8 flex justify-center gap-6 z-10 items-center">
        <Link
          href="/chat"
          className="w-12 h-12 rounded-full bg-gray-900 border border-pink-500/50 text-pink-500 flex items-center justify-center hover:scale-110 transition-transform"
        >
          <MessageSquare className="w-6 h-6" />
        </Link>

        <button
          onClick={() => {
            dislikeContent(currentCard.id);
            handleFeedback(currentCard, 0);
          }}
          className="w-16 h-16 rounded-full bg-gray-950 border border-red-500/50 text-red-500 flex items-center justify-center text-2xl hover:scale-110 transition-transform"
        >
          <img src="/cross.png" alt="Cross" className="w-9 h-9" />
        </button>

        <button
          onClick={() => setShowDetails(true)}
          className="w-12 h-12 rounded-full bg-gray-900 border border-blue-500/50 text-blue-500 flex items-center justify-center hover:scale-110 transition-transform"
        >
          <Info className="w-6 h-6" />
        </button>

        <button
          onClick={() => {
            likeContent(currentCard);
            handleFeedback(currentCard, 1);
          }}
          className="w-16 h-16 rounded-full bg-gray-950 border border-green-500/50 text-green-500 flex items-center justify-center text-2xl hover:scale-110 transition-transform"
        >
          <img src="/heart.png" alt="Heart" className="w-8 h-8 mt-1" />
        </button>

        <Link
          href="/history"
          className="w-12 h-12 rounded-full bg-gray-900 border border-purple-500/50 text-purple-500 flex items-center justify-center hover:scale-110 transition-transform"
        >
          <History className="w-6 h-6" />
        </Link>
      </div>

      {/* Details Modal */}
      <AnimatePresence>
        {showDetails && (
          <DetailsModal
            content={currentCard}
            onClose={() => setShowDetails(false)}
          />
        )}
      </AnimatePresence>
    </main>
  );
}
