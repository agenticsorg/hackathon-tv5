import { Suspense } from 'react';
import { SearchBar } from '@/components/SearchBar';
import { TrendingSection } from '@/components/TrendingSection';
import { RecommendationsSection } from '@/components/RecommendationsSection';
import { SystemArchitecture } from '@/components/SystemArchitecture';
import { AIMetrics } from '@/components/AIMetrics';
import { AIMetricsCompact } from '@/components/AIMetricsCompact';
import { HowItWorks } from '@/components/HowItWorks';

export default function HomePage() {
  return (
    <main className="min-h-screen bg-gradient-to-b from-gray-900 via-gray-900 to-black">
      {/* Hero Section */}
      <section className="relative py-20 px-4 text-center bg-gradient-to-b from-purple-900/20 via-blue-900/10 to-transparent">
        <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-purple-900/20 via-transparent to-transparent" />
        <div className="relative z-10">
          <h1 className="text-5xl md:text-7xl font-bold mb-6 bg-gradient-to-r from-purple-400 via-pink-400 to-blue-400 bg-clip-text text-transparent">
            AI Media Discovery
          </h1>
          <p className="text-lg md:text-xl text-gray-300 mb-4 max-w-3xl mx-auto">
            Powered by <span className="font-semibold text-purple-400">Exogenesis Omega</span> — a distributed AI system serving 40M+ devices
          </p>
          <p className="text-base text-gray-400 mb-8 max-w-2xl mx-auto">
            Describe what you want to watch in plain English. Our AI understands your mood and finds the perfect match using federated learning across millions of smart TVs.
          </p>
        </div>

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

      {/* Two-Column Layout: Movies (left) + AI Metrics (right) */}
      <div className="py-12 px-4 md:px-8">
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-6">
          {/* Left Column - Movies (70% width on desktop) */}
          <div className="lg:col-span-8 space-y-12">
            {/* Trending Section */}
            <section>
              <h2 className="text-2xl font-bold mb-6">Trending This Week</h2>
              <Suspense fallback={<ContentSkeleton />}>
                <TrendingSection />
              </Suspense>
            </section>

            {/* Recommendations Section */}
            <section className="py-8 px-6 rounded-xl bg-gradient-to-b from-transparent to-gray-900/50">
              <h2 className="text-2xl font-bold mb-6 text-white">Recommended For You</h2>
              <Suspense fallback={<ContentSkeleton />}>
                <RecommendationsSection />
              </Suspense>
            </section>
          </div>

          {/* Right Column - AI Metrics Dashboard (30% width on desktop, sticky) */}
          <aside className="lg:col-span-4">
            <div className="lg:sticky lg:top-4 space-y-4">
              {/* Compact AI Metrics for sidebar */}
              <div className="hidden lg:block">
                <Suspense fallback={<CompactMetricsSkeleton />}>
                  <AIMetricsCompact />
                </Suspense>
              </div>

              {/* Mobile: Show above movies on small screens */}
              <div className="lg:hidden mb-8 bg-gradient-to-b from-gray-900/50 to-transparent p-6 rounded-xl">
                <Suspense fallback={<MetricsSkeleton />}>
                  <AIMetrics />
                </Suspense>
              </div>
            </div>
          </aside>
        </div>
      </div>

      {/* System Architecture */}
      <section className="py-16 px-4 md:px-8 bg-gradient-to-b from-transparent via-purple-900/5 to-transparent">
        <Suspense fallback={<ArchitectureSkeleton />}>
          <SystemArchitecture />
        </Suspense>
      </section>

      {/* How It Works */}
      <section className="py-16 px-4 md:px-8 bg-gradient-to-b from-transparent to-gray-900/50">
        <Suspense fallback={<HowItWorksSkeleton />}>
          <HowItWorks />
        </Suspense>
      </section>

      {/* Footer */}
      <footer className="py-12 px-4 text-center text-gray-400 text-sm border-t border-gray-800">
        <div className="max-w-4xl mx-auto space-y-4">
          <p className="text-lg font-semibold text-white mb-2">
            Exogenesis Omega: Distributed TV Intelligence
          </p>
          <div className="flex flex-wrap justify-center gap-4 text-xs">
            <span>40M+ Connected TVs</span>
            <span>•</span>
            <span>Sub-15ms Recommendations</span>
            <span>•</span>
            <span>99.99% Uptime</span>
            <span>•</span>
            <span>Privacy-First Design</span>
          </div>
          <p className="pt-4">
            Powered by{' '}
            <a
              href="https://www.themoviedb.org/"
              className="underline hover:text-gray-300 transition-colors"
            >
              TMDB
            </a>{' '}
            &bull; Built with{' '}
            <a href="https://arw.dev" className="underline hover:text-gray-300 transition-colors">
              ARW
            </a>{' '}
            &bull;{' '}
            <a
              href="/.well-known/arw-manifest.json"
              className="underline hover:text-gray-300 transition-colors"
            >
              Agent API
            </a>
          </p>
          <p className="text-xs text-gray-500 pt-2">
            Using SIMD-accelerated vector search, federated learning, and differential privacy
          </p>
        </div>
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

function MetricsSkeleton() {
  return (
    <div className="space-y-6">
      <div className="h-12 bg-gray-800 rounded-lg animate-pulse max-w-md mx-auto" />
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {Array.from({ length: 6 }).map((_, i) => (
          <div key={i} className="h-32 bg-gray-800 rounded-lg animate-pulse" />
        ))}
      </div>
    </div>
  );
}

function CompactMetricsSkeleton() {
  return (
    <div className="space-y-4">
      <div className="h-8 bg-gray-800 rounded-lg animate-pulse" />
      <div className="space-y-3">
        {Array.from({ length: 3 }).map((_, i) => (
          <div key={i} className="h-24 bg-gray-800 rounded-lg animate-pulse" />
        ))}
      </div>
      <div className="h-12 bg-gray-800 rounded-lg animate-pulse" />
    </div>
  );
}

function ArchitectureSkeleton() {
  return (
    <div className="space-y-8">
      <div className="h-12 bg-gray-800 rounded-lg animate-pulse max-w-lg mx-auto" />
      <div className="grid gap-6 md:grid-cols-3">
        {Array.from({ length: 3 }).map((_, i) => (
          <div key={i} className="h-96 bg-gray-800 rounded-lg animate-pulse" />
        ))}
      </div>
    </div>
  );
}

function HowItWorksSkeleton() {
  return (
    <div className="space-y-6">
      <div className="h-12 bg-gray-800 rounded-lg animate-pulse max-w-md mx-auto" />
      {Array.from({ length: 4 }).map((_, i) => (
        <div key={i} className="h-48 bg-gray-800 rounded-lg animate-pulse" />
      ))}
    </div>
  );
}
