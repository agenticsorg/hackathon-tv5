'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';
import { Button, Spinner } from '@heroui/react';
import { SearchBar } from '@/components/SearchBar';
import { ContentCard } from '@/components/ContentCard';
import { LearningVisualization } from '@/components/LearningVisualization';
import { MoodSelector } from '@/components/MoodSelector';
import { ConfigurationModal } from '@/components/ConfigurationModal';
import type { Content, LearningStats } from '@/lib/db';

export default function Home() {
  const [content, setContent] = useState<Content[]>([]);
  const [stats, setStats] = useState<LearningStats | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [selectedMood, setSelectedMood] = useState<string | null>(null);
  const [selectedType, setSelectedType] = useState<string>('both');
  const [view, setView] = useState<'search' | 'recommendations'>('recommendations');
  const [showConfig, setShowConfig] = useState(false);

  // Fetch initial recommendations
  useEffect(() => {
    fetchRecommendations();
    fetchStats();
  }, []);

  // Fetch recommendations when mood or type changes
  useEffect(() => {
    if (view === 'recommendations') {
      fetchRecommendations();
    }
  }, [selectedMood, selectedType, view]);

  const fetchRecommendations = async () => {
    setIsLoading(true);
    try {
      const params = new URLSearchParams();
      if (selectedMood) params.set('mood', selectedMood);
      if (selectedType && selectedType !== 'both') params.set('type', selectedType);

      const response = await fetch(`/api/recommendations?${params}`);
      const data = await response.json();
      setContent(data.results || []);

      // Dispatch event to trigger latency update in LearningVisualization
      window.dispatchEvent(new CustomEvent('recommendation-query'));
    } catch (error) {
      console.error('Failed to fetch recommendations:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const fetchStats = async () => {
    try {
      const response = await fetch('/api/stats');
      const data = await response.json();
      setStats(data.stats);
    } catch (error) {
      console.error('Failed to fetch stats:', error);
    }
  };

  const handleSearch = async (query: string) => {
    setView('search');
    setIsLoading(true);
    try {
      const response = await fetch(`/api/search?q=${encodeURIComponent(query)}`);
      const data = await response.json();
      setContent(data.results || []);
    } catch (error) {
      console.error('Search failed:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleLike = async (id: string) => {
    try {
      await fetch('/api/feedback', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ contentId: id, wasSuccessful: true }),
      });
      fetchStats();
    } catch (error) {
      console.error('Failed to record feedback:', error);
    }
  };

  const handleSkip = async (id: string) => {
    try {
      await fetch('/api/feedback', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ contentId: id, wasSuccessful: false }),
      });
      fetchStats();
    } catch (error) {
      console.error('Failed to record feedback:', error);
    }
  };

  const handleViewSimilar = async (id: string) => {
    setIsLoading(true);
    try {
      const response = await fetch(`/api/similar?id=${id}`);
      const data = await response.json();
      setContent(data.results || []);
      setView('search');
    } catch (error) {
      console.error('Failed to fetch similar content:', error);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <main className="min-h-screen p-4 sm:p-6 md:p-12 relative z-10">
      {/* Animated background elements - hidden on small mobile for performance */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none z-0 hidden sm:block">
        {/* Glowing orbs */}
        <div className="glow-orb w-64 sm:w-96 h-64 sm:h-96 bg-emerald-500 top-20 -left-32 sm:-left-48" style={{ animationDelay: '0s' }} />
        <div className="glow-orb w-48 sm:w-80 h-48 sm:h-80 bg-blue-500 top-1/3 -right-24 sm:-right-40" style={{ animationDelay: '2s' }} />
        <div className="glow-orb w-40 sm:w-64 h-40 sm:h-64 bg-purple-500 bottom-20 left-1/4" style={{ animationDelay: '4s' }} />

        {/* Floating particles */}
        <div className="particle w-2 h-2 bg-emerald-400 top-1/4 left-1/4" style={{ animationDelay: '0s' }} />
        <div className="particle w-1.5 h-1.5 bg-blue-400 top-1/3 right-1/4" style={{ animationDelay: '1s' }} />
        <div className="particle w-2.5 h-2.5 bg-emerald-300 top-2/3 left-1/3" style={{ animationDelay: '2s' }} />
        <div className="particle w-1 h-1 bg-purple-400 bottom-1/4 right-1/3" style={{ animationDelay: '3s' }} />
        <div className="particle w-2 h-2 bg-emerald-500 top-1/2 left-2/3" style={{ animationDelay: '4s' }} />
      </div>

      {/* Hero Header */}
      <header className="text-center mb-8 sm:mb-12 relative hero-animated-bg rounded-2xl sm:rounded-3xl py-6 sm:py-8 px-3 sm:px-4 -mx-2 sm:-mx-4">
        <div className="flex flex-col sm:flex-row items-center justify-center gap-3 sm:gap-4 mb-4 sm:mb-6">
          <div className="relative">
            <div className="w-12 h-12 sm:w-16 sm:h-16 rounded-xl sm:rounded-2xl feature-icon flex items-center justify-center animate-float">
              <svg className="w-6 h-6 sm:w-8 sm:h-8 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
            </div>
            <div className="absolute -bottom-1 -right-1 w-4 h-4 sm:w-5 sm:h-5 rounded-full bg-emerald-500 flex items-center justify-center animate-pulse">
              <svg className="w-2.5 h-2.5 sm:w-3 sm:h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                <path fillRule="evenodd" d="M11.3 1.046A1 1 0 0112 2v5h4a1 1 0 01.82 1.573l-7 10A1 1 0 018 18v-5H4a1 1 0 01-.82-1.573l7-10a1 1 0 011.12-.38z" clipRule="evenodd" />
              </svg>
            </div>
          </div>
          <div className="text-center sm:text-left">
            <h1 className="text-2xl sm:text-4xl md:text-5xl font-bold gradient-text">
              TVDB Smart
            </h1>
            <h2 className="text-xl sm:text-3xl md:text-4xl font-bold gradient-text-accent">
              Recommendations
            </h2>
          </div>
        </div>

        <p className="text-zinc-400 text-sm sm:text-lg max-w-2xl mx-auto mb-4 sm:mb-6 px-2">
          AI-powered entertainment discovery that learns your preferences.
        </p>

        {/* Hero stats row - stack on mobile */}
        <div className="flex flex-wrap items-center justify-center gap-2 sm:gap-6 mb-4 sm:mb-8">
          <div className="flex items-center gap-2">
            <span className="badge-success px-2 sm:px-3 py-1 rounded-full text-xs sm:text-sm">
              <span className="font-semibold">{stats?.totalContent?.toLocaleString() || '1,444'}</span> titles
            </span>
          </div>
          <div className="flex items-center gap-2">
            <span className="badge-info px-2 sm:px-3 py-1 rounded-full text-xs sm:text-sm">
              <span className="font-semibold">3.2ms</span> search
            </span>
          </div>
          <div className="flex items-center gap-1 sm:gap-2 text-zinc-400 text-xs sm:text-sm">
            <svg className="w-3 h-3 sm:w-4 sm:h-4 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
            </svg>
            <span className="text-purple-400 font-semibold">6 ML</span> <span className="hidden sm:inline">algorithms active</span><span className="sm:hidden">algorithms</span>
          </div>
        </div>

        {/* Tagline - responsive */}
        <div className="glass-card inline-block px-3 sm:px-6 py-2 sm:py-3 rounded-full">
          <p className="text-xs sm:text-sm text-zinc-300">
            <span className="text-emerald-400 font-medium">Find what to watch in seconds</span>
            <span className="text-zinc-500 mx-1 sm:mx-2 hidden sm:inline">•</span>
            <span className="text-zinc-400 hidden sm:inline">Not 45 minutes</span>
          </p>
        </div>
      </header>

      {/* Search */}
      <section className="mb-6 sm:mb-10 max-w-4xl mx-auto px-0 sm:px-2">
        <SearchBar onSearch={handleSearch} isLoading={isLoading} />
      </section>

      {/* Main content */}
      <div className="flex flex-col lg:flex-row gap-4 sm:gap-8">
        {/* Content area */}
        <div className="flex-1 order-2 lg:order-1">
          {/* Filters bar */}
          <div className="glass-card rounded-xl sm:rounded-2xl p-3 sm:p-4 mb-4 sm:mb-6">
            <div className="flex flex-col gap-3 sm:gap-4">
              {/* View tabs - scrollable on mobile */}
              <div className="flex gap-2 overflow-x-auto pb-1 scrollbar-hide">
                <button
                  onClick={() => setView('recommendations')}
                  className={`flex-shrink-0 px-3 sm:px-4 py-2 rounded-lg sm:rounded-xl text-xs sm:text-sm font-medium transition-all ${
                    view === 'recommendations'
                      ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
                      : 'text-zinc-400 hover:text-white hover:bg-zinc-800'
                  }`}
                >
                  <span className="flex items-center gap-1.5 sm:gap-2">
                    <svg className="w-3.5 h-3.5 sm:w-4 sm:h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
                    </svg>
                    For You
                  </span>
                </button>
                <button
                  onClick={() => setView('search')}
                  className={`flex-shrink-0 px-3 sm:px-4 py-2 rounded-lg sm:rounded-xl text-xs sm:text-sm font-medium transition-all ${
                    view === 'search'
                      ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
                      : 'text-zinc-400 hover:text-white hover:bg-zinc-800'
                  }`}
                >
                  <span className="flex items-center gap-1.5 sm:gap-2">
                    <svg className="w-3.5 h-3.5 sm:w-4 sm:h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                    </svg>
                    <span className="hidden sm:inline">Search Results</span>
                    <span className="sm:hidden">Results</span>
                  </span>
                </button>
              </div>

              {/* Content type filter - full width on mobile */}
              <div className="flex gap-1.5 sm:gap-2">
                {['both', 'series', 'movie'].map((type) => (
                  <button
                    key={type}
                    onClick={() => setSelectedType(type)}
                    className={`flex-1 sm:flex-none px-3 sm:px-4 py-2 rounded-lg sm:rounded-xl text-xs sm:text-sm font-medium transition-all ${
                      selectedType === type
                        ? 'bg-zinc-700 text-white'
                        : 'text-zinc-400 hover:text-white hover:bg-zinc-800/50'
                    }`}
                  >
                    {type === 'both' ? 'All' : type === 'series' ? 'Series' : 'Movies'}
                  </button>
                ))}
              </div>
            </div>
          </div>

          {/* Mood selector */}
          {view === 'recommendations' && (
            <div className="mb-4 sm:mb-8">
              <p className="text-xs sm:text-sm text-zinc-500 text-center mb-3 sm:mb-4">What&apos;s your mood?</p>
              <MoodSelector selectedMood={selectedMood} onMoodChange={setSelectedMood} />
            </div>
          )}

          {/* Content grid */}
          {isLoading ? (
            <div className="flex flex-col items-center justify-center py-12 sm:py-20">
              <div className="w-12 h-12 sm:w-16 sm:h-16 rounded-xl sm:rounded-2xl feature-icon flex items-center justify-center mb-3 sm:mb-4 animate-pulse-glow">
                <Spinner size="lg" color="success" />
              </div>
              <p className="text-zinc-500 text-sm sm:text-base">Finding perfect matches...</p>
            </div>
          ) : content.length > 0 ? (
            <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-4 sm:gap-6">
              {content.map((item) => (
                <ContentCard
                  key={item.id}
                  content={item}
                  onLike={handleLike}
                  onSkip={handleSkip}
                  onViewSimilar={handleViewSimilar}
                />
              ))}
            </div>
          ) : (
            <div className="glass-card rounded-xl sm:rounded-2xl p-8 sm:p-12 text-center">
              <div className="w-12 h-12 sm:w-16 sm:h-16 rounded-xl sm:rounded-2xl feature-icon flex items-center justify-center mx-auto mb-3 sm:mb-4">
                <svg className="w-6 h-6 sm:w-8 sm:h-8 text-zinc-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              <p className="text-zinc-400 text-base sm:text-lg mb-2">No content found</p>
              <p className="text-zinc-500 text-xs sm:text-sm">Try a different search or select a mood</p>
            </div>
          )}
        </div>

        {/* Sidebar - shows first on mobile */}
        <aside className="w-full lg:w-96 shrink-0 space-y-4 sm:space-y-6 order-1 lg:order-2">
          {/* Learning Visualization */}
          <LearningVisualization stats={stats} />

          {/* How it works - collapsible on mobile */}
          <div className="stats-card p-4 sm:p-6">
            <h3 className="text-base sm:text-lg font-semibold text-white mb-3 sm:mb-4 flex items-center gap-2">
              <div className="w-7 h-7 sm:w-8 sm:h-8 rounded-lg feature-icon flex items-center justify-center">
                <svg className="w-3.5 h-3.5 sm:w-4 sm:h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              How It Works
            </h3>
            <ul className="space-y-3 sm:space-y-4">
              {[
                { step: '1', title: 'Search naturally', desc: 'Use everyday language to find content' },
                { step: '2', title: 'Interact', desc: 'Click Watch or Skip to teach the AI' },
                { step: '3', title: 'AI learns', desc: 'System improves from your choices' },
                { step: '4', title: 'Better results', desc: 'Get personalized recommendations' },
              ].map((item) => (
                <li key={item.step} className="flex items-start gap-2.5 sm:gap-3">
                  <div className="w-5 h-5 sm:w-6 sm:h-6 rounded-full bg-emerald-500/20 flex items-center justify-center text-[10px] sm:text-xs text-emerald-400 font-bold shrink-0">
                    {item.step}
                  </div>
                  <div>
                    <p className="text-white text-xs sm:text-sm font-medium">{item.title}</p>
                    <p className="text-zinc-500 text-[10px] sm:text-xs">{item.desc}</p>
                  </div>
                </li>
              ))}
            </ul>
            {/* Learn More Button */}
            <Link
              href="/about"
              className="mt-4 sm:mt-6 w-full flex items-center justify-center gap-2 px-3 sm:px-4 py-2.5 sm:py-3 rounded-lg sm:rounded-xl bg-emerald-500/20 border border-emerald-500/30 text-emerald-400 text-sm font-medium hover:bg-emerald-500/30 hover:border-emerald-500/50 transition-all group"
            >
              <svg className="w-4 h-4 sm:w-5 sm:h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
              </svg>
              <span className="hidden sm:inline">Learn About Our AI System</span>
              <span className="sm:hidden">About Our AI</span>
              <svg className="w-3.5 h-3.5 sm:w-4 sm:h-4 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
              </svg>
            </Link>
          </div>

          {/* Analytics Dashboard Button */}
          <Link
            href="/analytics"
            className="w-full glass-card rounded-xl sm:rounded-2xl p-3 sm:p-4 flex items-center justify-between group hover:border-purple-500/30 transition-all"
          >
            <div className="flex items-center gap-2 sm:gap-3">
              <div className="w-8 h-8 sm:w-10 sm:h-10 rounded-lg sm:rounded-xl bg-purple-500/20 flex items-center justify-center">
                <svg className="w-4 h-4 sm:w-5 sm:h-5 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                </svg>
              </div>
              <div className="text-left">
                <p className="text-xs sm:text-sm font-medium text-white">Analytics Dashboard</p>
                <p className="text-[10px] sm:text-xs text-zinc-500">Live graphs & network view</p>
              </div>
            </div>
            <svg className="w-4 h-4 sm:w-5 sm:h-5 text-zinc-500 group-hover:text-purple-400 group-hover:translate-x-1 transition-all" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
            </svg>
          </Link>

          {/* Configuration Button */}
          <button
            onClick={() => setShowConfig(true)}
            className="w-full glass-card rounded-xl sm:rounded-2xl p-3 sm:p-4 flex items-center justify-between group hover:border-emerald-500/30 transition-all"
          >
            <div className="flex items-center gap-2 sm:gap-3">
              <div className="w-8 h-8 sm:w-10 sm:h-10 rounded-lg sm:rounded-xl bg-emerald-500/20 flex items-center justify-center">
                <svg className="w-4 h-4 sm:w-5 sm:h-5 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                </svg>
              </div>
              <div className="text-left">
                <p className="text-xs sm:text-sm font-medium text-white">System Configuration</p>
                <p className="text-[10px] sm:text-xs text-zinc-500">Explore AI parameters</p>
              </div>
            </div>
            <svg className="w-4 h-4 sm:w-5 sm:h-5 text-zinc-500 group-hover:text-emerald-400 group-hover:translate-x-1 transition-all" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
            </svg>
          </button>

          {/* Tech stack */}
          <div className="glass-card rounded-xl sm:rounded-2xl p-3 sm:p-4">
            <div className="flex items-center gap-2 text-[10px] sm:text-xs text-zinc-500 mb-2 sm:mb-3">
              <svg className="w-3.5 h-3.5 sm:w-4 sm:h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
              </svg>
              Built with
            </div>
            <div className="flex flex-wrap gap-1.5 sm:gap-2">
              {['TVDB API', 'RuVector', 'AgentDB', 'Next.js', 'HeroUI'].map((tech) => (
                <span key={tech} className="px-2 py-1 rounded-md sm:rounded-lg bg-zinc-800/50 text-[10px] sm:text-xs text-zinc-400">
                  {tech}
                </span>
              ))}
            </div>
          </div>
        </aside>
      </div>

      {/* Footer */}
      <footer className="mt-8 sm:mt-16 pt-6 sm:pt-8">
        <div className="divider-glow mb-6 sm:mb-8" />
        <div className="text-center px-4">
          <p className="text-xs sm:text-sm text-zinc-500 mb-2">
            Solving the &quot;45-minute decision problem&quot; with AI-powered recommendations
          </p>
          <p className="text-[10px] sm:text-xs text-zinc-600">
            Built for the TV5 Hackathon • Entertainment Discovery Track
          </p>
        </div>
      </footer>

      {/* Configuration Modal */}
      <ConfigurationModal isOpen={showConfig} onClose={() => setShowConfig(false)} />
    </main>
  );
}
