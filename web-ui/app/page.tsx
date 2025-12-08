'use client';

import { useState } from 'react';
import Header from '@/components/Header';
import SearchSection from '@/components/SearchSection';
import AgentActivity from '@/components/AgentActivity';
import RecommendationSection, { type Recommendation } from '@/components/RecommendationSection';
import { mockRecommendations, mockTrending } from '@/lib/mockData';

export default function Home() {
  const [isSearching, setIsSearching] = useState(false);
  const [query, setQuery] = useState('');
  const [recommendations, setRecommendations] = useState<Recommendation[]>([]);
  const [useMockData, setUseMockData] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSearch = async (searchQuery: string) => {
    setQuery(searchQuery);
    setIsSearching(true);
    setError(null);

    try {
      // Call Next.js API route which proxies to Python backend
      const response = await fetch('/api/recommendations', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          query: searchQuery,
          context: {
            viewing: 'solo', // Could be dynamic from user input
            energy: 'intense',
            duration: 'movie',
          },
        }),
      });

      const data = await response.json();

      if (!response.ok) {
        // If Python backend is not running, fallback to mock data
        if (data.fallback) {
          console.warn('Python backend not available, using mock data');
          setUseMockData(true);
          setRecommendations(mockRecommendations);
          // Still show agent activity for demo purposes
          await new Promise(resolve => setTimeout(resolve, 3000));
        } else {
          throw new Error(data.error || 'Failed to get recommendations');
        }
      } else {
        // Successfully got recommendations from Python backend
        setUseMockData(false);
        setRecommendations(data.recommendations || []);
        console.log('✅ Live recommendations from Python agents:', data);
        console.log(`⏱️  Execution time: ${data.executionTime}s`);
        console.log(`📊 Candidates processed: ${data.candidatesProcessed}`);
      }
    } catch (err: any) {
      console.error('Search error:', err);
      setError(err.message);
      // Fallback to mock data on error
      setUseMockData(true);
      setRecommendations(mockRecommendations);
      await new Promise(resolve => setTimeout(resolve, 3000));
    } finally {
      setIsSearching(false);
    }
  };

  return (
    <div className="min-h-screen">
      <Header />

      <main className="max-w-[1400px] mx-auto px-4 py-8">
        {/* Status Banner */}
        {useMockData && (
          <div className="mb-4 px-4 py-2 bg-semantic-warning/10 border border-semantic-warning/20 rounded-lg text-sm text-semantic-warning">
            ⚠️ Using mock data. Python backend not available. Start with: <code className="bg-black/20 px-2 py-0.5 rounded">cd api && python3 main.py</code>
          </div>
        )}

        {error && (
          <div className="mb-4 px-4 py-2 bg-semantic-error/10 border border-semantic-error/20 rounded-lg text-sm text-semantic-error">
            ❌ Error: {error}
          </div>
        )}

        <SearchSection onSearch={handleSearch} />

        {isSearching && (
          <div className="mt-8">
            <AgentActivity />
          </div>
        )}

        {!isSearching && !query && (
          <>
            <RecommendationSection
              title="🌟 Top Picks for You"
              subtitle="Personalized recommendations based on your preferences"
              recommendations={mockRecommendations}
              layout="grid"
            />

            <RecommendationSection
              title="🔥 Trending Now"
              subtitle="What everyone is watching right now"
              recommendations={mockTrending}
              layout="scroll"
            />
          </>
        )}

        {!isSearching && query && recommendations.length > 0 && (
          <RecommendationSection
            title={useMockData ? `📊 Mock Results for "${query}"` : `🎯 AI Recommendations for "${query}"`}
            subtitle={useMockData ? 'Demo data (Python backend not running)' : `Found ${recommendations.length} personalized matches using 8-agent system`}
            recommendations={recommendations}
            layout="grid"
          />
        )}

        {!isSearching && query && recommendations.length === 0 && (
          <div className="mt-8 text-center py-12">
            <p className="text-text-dark-secondary text-lg">No recommendations found. Try a different search term.</p>
          </div>
        )}
      </main>
    </div>
  );
}
