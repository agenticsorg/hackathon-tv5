import { Suspense } from 'react';
import { SearchBar } from '@/components/SearchBar';
import { TrendingSection } from '@/components/TrendingSection';
import { RecommendationsSection } from '@/components/RecommendationsSection';

export default function HomePage() {
  return (
    <main className="min-h-screen">
      {/* Hero Section */}
      <section className="relative py-20 px-4 text-center bg-gradient-to-b from-blue-600/10 to-transparent">
        <h1 className="text-4xl md:text-5xl font-bold mb-4">
          AI Media Discovery
        </h1>
        <p className="text-lg md:text-xl text-gray-600 dark:text-gray-400 mb-8 max-w-2xl mx-auto">
          Describe what you want to watch in plain English. Our AI understands
          your mood and finds the perfect match.
        </p>

        {/* Search Bar */}
        <div className="max-w-2xl mx-auto">
          <Suspense fallback={<SearchBarSkeleton />}>
            <SearchBar />
          </Suspense>
        </div>

        {/* Example Prompts */}
        <div className="mt-6 flex flex-wrap justify-center gap-2">
          {[
            'exciting sci-fi adventure',
            'cozy romantic comedy',
            'dark psychological thriller',
            'inspiring true story',
          ].map((prompt) => (
            <button
              key={prompt}
              className="px-3 py-1.5 text-sm bg-gray-100 dark:bg-gray-800 rounded-full hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
            >
              {prompt}
            </button>
          ))}
        </div>
      </section>

      {/* Trending Section */}
      <section className="py-12 px-4 md:px-8">
        <h2 className="text-2xl font-bold mb-6">Trending This Week</h2>
        <Suspense fallback={<ContentSkeleton />}>
          <TrendingSection />
        </Suspense>
      </section>

      {/* Recommendations Section */}
      <section className="py-12 px-4 md:px-8 bg-gray-50 dark:bg-gray-900/50">
        <h2 className="text-2xl font-bold mb-6">Recommended For You</h2>
        <Suspense fallback={<ContentSkeleton />}>
          <RecommendationsSection />
        </Suspense>
      </section>

      {/* Footer */}
      <footer className="py-8 px-4 text-center text-gray-500 text-sm">
        <p>
          Powered by{' '}
          <a
            href="https://www.themoviedb.org/"
            className="underline hover:text-gray-700"
          >
            TMDB
          </a>{' '}
          &bull; Built with{' '}
          <a href="https://arw.dev" className="underline hover:text-gray-700">
            ARW
          </a>{' '}
          &bull;{' '}
          <a
            href="/.well-known/arw-manifest.json"
            className="underline hover:text-gray-700"
          >
            Agent API
          </a>
        </p>
      </footer>
    </main>
  );
}

// Skeleton components
function SearchBarSkeleton() {
  return (
    <div className="h-14 bg-gray-200 dark:bg-gray-800 rounded-xl animate-pulse" />
  );
}

function ContentSkeleton() {
  return (
    <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
      {Array.from({ length: 6 }).map((_, i) => (
        <div
          key={i}
          className="aspect-[2/3] bg-gray-200 dark:bg-gray-800 rounded-lg animate-pulse"
        />
      ))}
    </div>
  );
}
