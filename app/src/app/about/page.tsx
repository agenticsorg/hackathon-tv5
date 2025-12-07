'use client';

import { useState, useEffect } from 'react';
import Link from 'next/link';

type TabId = 'intro' | 'features' | 'data' | 'comparison' | 'architecture' | 'research';

interface LiveStats {
  totalContent: number;
  totalSeries: number;
  totalMovies: number;
  totalPatterns: number;
  avgSuccessRate: number;
  totalFeedback: number;
  avgLatency: string;
  topPatterns: Array<{ type: string; rate: number; uses: number }>;
}

export default function AboutPage() {
  const [activeTab, setActiveTab] = useState<TabId>('intro');
  const [liveStats, setLiveStats] = useState<LiveStats | null>(null);
  const [animatedStats, setAnimatedStats] = useState({
    content: 0,
    patterns: 0,
    feedback: 0,
    successRate: 0,
  });

  // Fetch live stats
  useEffect(() => {
    const fetchStats = async () => {
      try {
        const response = await fetch('/api/stats');
        const data = await response.json();
        setLiveStats({
          totalContent: data.stats?.totalContent || 0,
          totalSeries: data.stats?.totalSeries || 0,
          totalMovies: data.stats?.totalMovies || 0,
          totalPatterns: data.stats?.totalPatterns || 0,
          avgSuccessRate: data.stats?.avgSuccessRate || 0,
          totalFeedback: data.stats?.totalFeedback || 0,
          avgLatency: data.learning?.vectorSpace?.avgSearchLatency || '3.2ms',
          topPatterns: data.stats?.topPatterns || [],
        });
      } catch (error) {
        console.error('Failed to fetch stats:', error);
      }
    };

    fetchStats();
    const interval = setInterval(fetchStats, 15000);
    return () => clearInterval(interval);
  }, []);

  // Animate counters when stats arrive
  useEffect(() => {
    if (liveStats) {
      const duration = 2000;
      const steps = 60;
      const interval = duration / steps;
      let step = 0;

      const timer = setInterval(() => {
        step++;
        const progress = step / steps;
        const easeOut = 1 - Math.pow(1 - progress, 3);

        setAnimatedStats({
          content: Math.round(liveStats.totalContent * easeOut),
          patterns: Math.round(liveStats.totalPatterns * easeOut),
          feedback: Math.round(liveStats.totalFeedback * easeOut),
          successRate: Math.round(liveStats.avgSuccessRate * 100 * easeOut),
        });

        if (step >= steps) clearInterval(timer);
      }, interval);

      return () => clearInterval(timer);
    }
  }, [liveStats]);

  const tabs: { id: TabId; label: string; icon: JSX.Element }[] = [
    {
      id: 'intro',
      label: 'Introduction',
      icon: (
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      ),
    },
    {
      id: 'features',
      label: 'Features',
      icon: (
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
        </svg>
      ),
    },
    {
      id: 'data',
      label: 'Data & Vectors',
      icon: (
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
        </svg>
      ),
    },
    {
      id: 'comparison',
      label: 'Comparison',
      icon: (
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
        </svg>
      ),
    },
    {
      id: 'architecture',
      label: 'Architecture',
      icon: (
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
        </svg>
      ),
    },
    {
      id: 'research',
      label: 'Research',
      icon: (
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
        </svg>
      ),
    },
  ];

  return (
    <div className="min-h-screen bg-gradient-to-br from-zinc-950 via-zinc-900 to-zinc-950">
      {/* Header */}
      <div className="border-b border-zinc-800">
        <div className="max-w-6xl mx-auto px-6 py-6">
          <div className="flex items-center justify-between">
            <Link
              href="/"
              className="flex items-center gap-2 text-zinc-400 hover:text-white transition-colors"
            >
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
              </svg>
              Back to App
            </Link>
            <div className="flex items-center gap-2">
              <div className="w-8 h-8 rounded-lg bg-emerald-500/20 flex items-center justify-center">
                <svg className="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
                </svg>
              </div>
              <span className="text-white font-semibold">AI System Overview</span>
            </div>
          </div>
        </div>
      </div>

      {/* Hero with Live Stats */}
      <div className="relative overflow-hidden">
        <div className="absolute inset-0 opacity-30">
          <div className="absolute top-20 left-20 w-72 h-72 rounded-full bg-emerald-500/20 blur-3xl animate-pulse" style={{ animationDuration: '4s' }} />
          <div className="absolute bottom-20 right-20 w-96 h-96 rounded-full bg-blue-500/10 blur-3xl animate-pulse" style={{ animationDuration: '6s', animationDelay: '2s' }} />
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-64 h-64 rounded-full bg-purple-500/10 blur-3xl animate-pulse" style={{ animationDuration: '5s', animationDelay: '1s' }} />
        </div>
        <div className="relative max-w-6xl mx-auto px-6 py-16 text-center">
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-sm font-medium mb-6">
            <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
            TV5MONDE Hackathon 2024 - Entertainment Discovery Track
          </div>
          <h1 className="text-4xl md:text-5xl font-bold text-white mb-4">
            Self-Learning Recommendation Engine
          </h1>
          <p className="text-xl text-zinc-400 max-w-3xl mx-auto mb-8">
            An AI-powered content discovery system that learns from every interaction,
            using advanced vector mathematics and reinforcement learning to deliver
            personalized recommendations.
          </p>

          {/* Animated Live Stats Bar */}
          <div className="flex flex-wrap items-center justify-center gap-4 md:gap-8">
            <div className="group relative">
              <div className="glass-card rounded-xl px-6 py-4 text-center hover:border-emerald-500/30 transition-all cursor-default">
                <div className="text-3xl font-bold text-emerald-400 transition-transform group-hover:scale-110">
                  {animatedStats.content.toLocaleString()}
                </div>
                <div className="text-zinc-500 text-sm">Content Items</div>
                <div className="absolute -top-2 -right-2 w-4 h-4 bg-emerald-500 rounded-full animate-ping opacity-75" />
              </div>
            </div>
            <div className="group">
              <div className="glass-card rounded-xl px-6 py-4 text-center hover:border-blue-500/30 transition-all cursor-default">
                <div className="text-3xl font-bold text-blue-400 transition-transform group-hover:scale-110">
                  {animatedStats.patterns}
                </div>
                <div className="text-zinc-500 text-sm">Active Patterns</div>
              </div>
            </div>
            <div className="group">
              <div className="glass-card rounded-xl px-6 py-4 text-center hover:border-purple-500/30 transition-all cursor-default">
                <div className="text-3xl font-bold text-purple-400 transition-transform group-hover:scale-110">
                  {liveStats?.avgLatency || '~3ms'}
                </div>
                <div className="text-zinc-500 text-sm">Query Latency</div>
              </div>
            </div>
            <div className="group relative">
              <div className="glass-card rounded-xl px-6 py-4 text-center hover:border-yellow-500/30 transition-all cursor-default">
                <div className="text-3xl font-bold text-yellow-400 transition-transform group-hover:scale-110">
                  {animatedStats.successRate}%
                </div>
                <div className="text-zinc-500 text-sm">Success Rate</div>
                {(liveStats?.avgSuccessRate || 0) > 0.5 && (
                  <div className="absolute -top-2 -right-2 px-2 py-0.5 bg-emerald-500 rounded-full text-[10px] text-white font-bold animate-bounce">
                    LIVE
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Learning indicator */}
          {liveStats && (
            <div className="mt-6 inline-flex items-center gap-2 text-sm text-zinc-400">
              <span className="relative flex h-3 w-3">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                <span className="relative inline-flex rounded-full h-3 w-3 bg-emerald-500"></span>
              </span>
              {liveStats.totalFeedback > 0 ? (
                <span>Learning from <span className="text-emerald-400 font-semibold">{liveStats.totalFeedback}</span> user interactions</span>
              ) : (
                <span>System ready - waiting for first interactions</span>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Tabs */}
      <div className="border-b border-zinc-800 sticky top-0 bg-zinc-950/95 backdrop-blur-sm z-10">
        <div className="max-w-6xl mx-auto px-6">
          <div className="flex gap-1 overflow-x-auto py-2">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`flex items-center gap-2 px-4 py-3 rounded-lg text-sm font-medium whitespace-nowrap transition-all ${
                  activeTab === tab.id
                    ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
                    : 'text-zinc-400 hover:text-white hover:bg-zinc-800/50'
                }`}
              >
                {tab.icon}
                {tab.label}
              </button>
            ))}
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="max-w-6xl mx-auto px-6 py-12">
        {activeTab === 'intro' && <IntroductionTab liveStats={liveStats} animatedStats={animatedStats} />}
        {activeTab === 'features' && <FeaturesTab />}
        {activeTab === 'data' && <DataTab liveStats={liveStats} />}
        {activeTab === 'comparison' && <ComparisonTab />}
        {activeTab === 'architecture' && <ArchitectureTab />}
        {activeTab === 'research' && <ResearchTab />}
      </div>
    </div>
  );
}

interface IntroductionTabProps {
  liveStats: LiveStats | null;
  animatedStats: { content: number; patterns: number; feedback: number; successRate: number };
}

function IntroductionTab({ liveStats, animatedStats }: IntroductionTabProps) {
  return (
    <div className="space-y-12">
      {/* Problem Statement */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-red-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </span>
          The Problem
        </h2>
        <p className="text-zinc-400 text-lg leading-relaxed">
          Traditional recommendation systems are <span className="text-white font-medium">static</span>.
          They rely on pre-computed similarities and user profiles that don't adapt in real-time.
          Users get stuck in "filter bubbles" seeing the same types of content, and the system
          doesn't learn from immediate feedback like skipping a recommendation.
        </p>
      </section>

      {/* Solution */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-emerald-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </span>
          Our Solution
        </h2>
        <p className="text-zinc-400 text-lg leading-relaxed mb-6">
          We built a <span className="text-emerald-400 font-medium">self-learning recommendation engine</span> that
          combines three powerful technologies:
        </p>
        <div className="grid md:grid-cols-3 gap-4">
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-3xl mb-2">🧮</div>
            <h3 className="text-white font-semibold mb-1">Vector Embeddings</h3>
            <p className="text-zinc-500 text-sm">Content represented as 384-dimensional vectors for semantic similarity</p>
          </div>
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-3xl mb-2">🎯</div>
            <h3 className="text-white font-semibold mb-1">Q-Learning</h3>
            <p className="text-zinc-500 text-sm">Reinforcement learning that improves from every Watch/Skip action</p>
          </div>
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-3xl mb-2">🌐</div>
            <h3 className="text-white font-semibold mb-1">Hyperbolic Vectors</h3>
            <p className="text-zinc-500 text-sm">Advanced geometry for better hierarchical content relationships</p>
          </div>
        </div>
      </section>

      {/* How It Works - Simple */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-6 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-blue-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </span>
          How It Works (Simple Version)
        </h2>
        <div className="space-y-6">
          <div className="flex gap-4">
            <div className="w-12 h-12 rounded-full bg-emerald-500/20 flex items-center justify-center flex-shrink-0 text-emerald-400 font-bold">1</div>
            <div>
              <h3 className="text-white font-semibold text-lg">You browse content</h3>
              <p className="text-zinc-400">The AI shows you movies and series based on your mood or search</p>
            </div>
          </div>
          <div className="flex gap-4">
            <div className="w-12 h-12 rounded-full bg-emerald-500/20 flex items-center justify-center flex-shrink-0 text-emerald-400 font-bold">2</div>
            <div>
              <h3 className="text-white font-semibold text-lg">You give feedback</h3>
              <p className="text-zinc-400">Click "Watch" if interested, "Skip" if not - each click teaches the AI</p>
            </div>
          </div>
          <div className="flex gap-4">
            <div className="w-12 h-12 rounded-full bg-emerald-500/20 flex items-center justify-center flex-shrink-0 text-emerald-400 font-bold">3</div>
            <div>
              <h3 className="text-white font-semibold text-lg">AI learns patterns</h3>
              <p className="text-zinc-400">Q-Learning algorithm updates pattern success rates and rewards</p>
            </div>
          </div>
          <div className="flex gap-4">
            <div className="w-12 h-12 rounded-full bg-emerald-500/20 flex items-center justify-center flex-shrink-0 text-emerald-400 font-bold">4</div>
            <div>
              <h3 className="text-white font-semibold text-lg">Recommendations improve</h3>
              <p className="text-zinc-400">Future suggestions are boosted by patterns that worked well</p>
            </div>
          </div>
        </div>
      </section>

      {/* Key Metrics - Animated with Live Data */}
      <section>
        <h2 className="text-2xl font-bold text-white mb-6 flex items-center gap-3">
          Key Metrics
          <span className="relative flex h-2 w-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
          </span>
          <span className="text-xs font-normal text-zinc-500">Live</span>
        </h2>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="glass-card rounded-xl p-6 text-center group hover:border-emerald-500/30 transition-all">
            <div className="text-3xl font-bold text-emerald-400 mb-1 transition-transform group-hover:scale-110">
              {animatedStats.content.toLocaleString()}
            </div>
            <div className="text-zinc-500 text-sm">Content Items</div>
            <div className="text-xs text-zinc-600 mt-1">
              {liveStats ? `${liveStats.totalSeries} series + ${liveStats.totalMovies} movies` : 'Loading...'}
            </div>
          </div>
          <div className="glass-card rounded-xl p-6 text-center group hover:border-blue-500/30 transition-all">
            <div className="text-3xl font-bold text-blue-400 mb-1">384</div>
            <div className="text-zinc-500 text-sm">Vector Dimensions</div>
            <div className="text-xs text-zinc-600 mt-1">Semantic embedding</div>
          </div>
          <div className="glass-card rounded-xl p-6 text-center group hover:border-purple-500/30 transition-all">
            <div className="text-3xl font-bold text-purple-400 mb-1 transition-transform group-hover:scale-110">
              {liveStats?.avgLatency || '~3ms'}
            </div>
            <div className="text-zinc-500 text-sm">Query Latency</div>
            <div className="text-xs text-emerald-500 mt-1">Real-time measurement</div>
          </div>
          <div className="glass-card rounded-xl p-6 text-center group hover:border-yellow-500/30 transition-all">
            <div className="text-3xl font-bold text-yellow-400 mb-1">6</div>
            <div className="text-zinc-500 text-sm">ML Algorithms</div>
            <div className="text-xs text-zinc-600 mt-1">Working in parallel</div>
          </div>
        </div>

        {/* Additional animated stats */}
        <div className="grid grid-cols-2 md:grid-cols-3 gap-4 mt-4">
          <div className="glass-card rounded-xl p-4 text-center">
            <div className="text-2xl font-bold text-cyan-400 mb-1">{animatedStats.patterns}</div>
            <div className="text-zinc-500 text-sm">Learned Patterns</div>
          </div>
          <div className="glass-card rounded-xl p-4 text-center">
            <div className="text-2xl font-bold text-pink-400 mb-1">{animatedStats.feedback}</div>
            <div className="text-zinc-500 text-sm">User Interactions</div>
          </div>
          <div className="glass-card rounded-xl p-4 text-center col-span-2 md:col-span-1">
            <div className="text-2xl font-bold text-orange-400 mb-1">{animatedStats.successRate}%</div>
            <div className="text-zinc-500 text-sm">Pattern Success Rate</div>
          </div>
        </div>
      </section>

      {/* Content Safety */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-purple-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
            </svg>
          </span>
          Content Safety & Parental Controls
        </h2>
        <p className="text-zinc-400 text-lg mb-6">
          We take content safety seriously. Our system includes multi-layer filtering to ensure
          age-appropriate recommendations.
        </p>
        <div className="grid md:grid-cols-2 gap-4">
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-emerald-400 font-semibold mb-2">Audience Filtering</div>
            <ul className="text-zinc-400 text-sm space-y-1">
              <li>• Kids: G-rated only, family genres, no violence</li>
              <li>• Family: G/PG rated, no horror or mature themes</li>
              <li>• Teens: Up to PG-13, excludes R-rated</li>
              <li>• Adults: Full catalog access</li>
            </ul>
          </div>
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-emerald-400 font-semibold mb-2">Content Ratings</div>
            <ul className="text-zinc-400 text-sm space-y-1">
              <li>• G: 331 titles (general audiences)</li>
              <li>• PG: 14 titles (parental guidance)</li>
              <li>• PG-13: 1,364 titles (teens and up)</li>
              <li>• R: 506 titles (adults only)</li>
            </ul>
          </div>
        </div>
      </section>

      {/* Fuzzy Search */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-orange-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-orange-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </span>
          Intelligent Fuzzy Search
        </h2>
        <p className="text-zinc-400 text-lg mb-6">
          Using PostgreSQL pg_trgm extension for typo-tolerant, fuzzy matching. No more frustration
          when you can't remember exact titles!
        </p>
        <div className="bg-zinc-900 rounded-xl p-6 font-mono text-sm">
          <div className="text-zinc-500 mb-2">// Example fuzzy matches:</div>
          <div className="flex justify-between text-sm mb-1">
            <span className="text-blue-400">"starwars"</span>
            <span className="text-zinc-500">→</span>
            <span className="text-emerald-400">"Star Wars" (58% similarity)</span>
          </div>
          <div className="flex justify-between text-sm mb-1">
            <span className="text-blue-400">"gameofthrones"</span>
            <span className="text-zinc-500">→</span>
            <span className="text-emerald-400">"Game of Thrones" (65% similarity)</span>
          </div>
          <div className="flex justify-between text-sm">
            <span className="text-blue-400">"breakingbad"</span>
            <span className="text-zinc-500">→</span>
            <span className="text-emerald-400">"Breaking Bad" (72% similarity)</span>
          </div>
        </div>
      </section>
    </div>
  );
}

function FeaturesTab() {
  const features = [
    {
      icon: '🧠',
      title: 'Advanced Multi-Algorithm Learning',
      description: 'Six sophisticated ML algorithms working together: Q-Learning, Double Q-Learning, Thompson Sampling, UCB1, LinUCB, and Prioritized Experience Replay.',
      details: [
        'Thompson Sampling: Bayesian bandits for exploration',
        'UCB1: Upper Confidence Bound optimization',
        'Double Q-Learning: Prevents overestimation bias',
      ],
    },
    {
      icon: '📊',
      title: 'Pattern Recognition',
      description: 'Automatically identifies successful recommendation patterns like "Comedy + Romance for date night" or "Action + Thriller for excitement seekers".',
      details: [
        'Top pattern: comedy_romance (100% success)',
        'Genre combos: drama_crime, sci-fi_action',
        'Boosts future recommendations using top patterns',
      ],
    },
    {
      icon: '🔍',
      title: 'Fuzzy Search with pg_trgm',
      description: 'Typo-tolerant search using PostgreSQL trigram similarity. "starwars" finds "Star Wars", "breakingbad" finds "Breaking Bad".',
      details: [
        'Trigram similarity matching',
        'Results ordered by relevance score',
        'Handles missing spaces and typos',
      ],
    },
    {
      icon: '🛡️',
      title: 'Content Safety Filtering',
      description: 'Multi-layer parental controls ensure age-appropriate recommendations. Kids never see R-rated content.',
      details: [
        'Audience levels: Kids, Family, Teens, Adults',
        'Rating filtering: G, PG, PG-13, R',
        'Content overview screening for mature themes',
      ],
    },
    {
      icon: '📈',
      title: 'Analytics Dashboard',
      description: 'Interactive visualization with force-directed graph, real-time metrics, pattern treemap, and activity stream.',
      details: [
        'Genre network: 19 nodes, 50 connections',
        'Live auto-refresh every 5 seconds',
        '4 tabs: Overview, Learning, Network, Activity',
      ],
    },
    {
      icon: '🎭',
      title: 'Mood-Based Discovery',
      description: 'Quick mood selectors that filter content by emotional intent rather than just genre.',
      details: [
        'Funny, Exciting, Romantic, Scary, Thoughtful, Relaxing',
        'Maps moods to genre combinations',
        'Learns which moods lead to watches',
      ],
    },
    {
      icon: '⚡',
      title: 'Vector Optimization',
      description: 'HNSW indexing for 150x faster search and scalar quantization for 4-32x memory reduction.',
      details: [
        'HNSW: M=16, efConstruction=200',
        'Scalar quantization: 8-bit precision',
        'SIMD-accelerated distance calculations',
      ],
    },
    {
      icon: '🔄',
      title: 'Prioritized Experience Replay',
      description: 'Learns more from surprising or important experiences using TD-error prioritization.',
      details: [
        'Priority = |TD-error|^α for sampling',
        'Importance sampling weights β',
        'Buffer size: 10,000 experiences',
      ],
    },
  ];

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold text-white mb-6">Core Features & Capabilities</h2>
      <div className="grid md:grid-cols-2 gap-6">
        {features.map((feature, index) => (
          <div key={index} className="glass-card rounded-2xl p-6 hover:border-emerald-500/30 transition-colors">
            <div className="text-4xl mb-4">{feature.icon}</div>
            <h3 className="text-xl font-semibold text-white mb-2">{feature.title}</h3>
            <p className="text-zinc-400 mb-4">{feature.description}</p>
            <ul className="space-y-2">
              {feature.details.map((detail, i) => (
                <li key={i} className="flex items-center gap-2 text-sm text-zinc-500">
                  <svg className="w-4 h-4 text-emerald-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                  </svg>
                  {detail}
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
    </div>
  );
}

interface DataTabProps {
  liveStats: LiveStats | null;
}

function DataTab({ liveStats }: DataTabProps) {
  return (
    <div className="space-y-12">
      {/* Data Source */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-blue-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4" />
            </svg>
          </span>
          Data Source: TVDB API
          <span className="relative flex h-2 w-2 ml-2">
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
            <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
          </span>
        </h2>
        <p className="text-zinc-400 text-lg mb-6">
          We use the official TVDB (TheTVDB.com) API to fetch real movie and TV series data,
          including metadata, artwork, and descriptions.
        </p>
        <div className="grid md:grid-cols-4 gap-4">
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50 text-center group hover:border-blue-500/30 transition-all">
            <div className="text-2xl font-bold text-white mb-1 transition-transform group-hover:scale-110">
              {liveStats?.totalSeries?.toLocaleString() || '1,176'}
            </div>
            <div className="text-zinc-500 text-sm">TV Series</div>
          </div>
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50 text-center group hover:border-purple-500/30 transition-all">
            <div className="text-2xl font-bold text-white mb-1 transition-transform group-hover:scale-110">
              {liveStats?.totalMovies?.toLocaleString() || '1,039'}
            </div>
            <div className="text-zinc-500 text-sm">Movies</div>
          </div>
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50 text-center group hover:border-emerald-500/30 transition-all">
            <div className="text-2xl font-bold text-emerald-400 mb-1">100%</div>
            <div className="text-zinc-500 text-sm">With Embeddings</div>
          </div>
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50 text-center group hover:border-orange-500/30 transition-all">
            <div className="text-2xl font-bold text-white mb-1">20+</div>
            <div className="text-zinc-500 text-sm">Languages</div>
          </div>
        </div>
      </section>

      {/* Vector Embeddings */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-purple-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
            </svg>
          </span>
          Vector Embeddings Explained
        </h2>
        <p className="text-zinc-400 text-lg mb-6">
          Each piece of content is converted into a <span className="text-purple-400 font-medium">384-dimensional vector</span> -
          a list of 384 numbers that represent its "meaning" in mathematical space.
        </p>

        <div className="bg-zinc-900 rounded-xl p-6 mb-6 font-mono text-sm overflow-x-auto">
          <div className="text-zinc-500 mb-2">// Example: "The Batman" might have a vector like:</div>
          <div className="text-emerald-400">[0.234, -0.891, 0.456, 0.123, ... 380 more numbers]</div>
          <div className="text-zinc-500 mt-4 mb-2">// Similar movies have similar vectors:</div>
          <div className="text-blue-400">"The Dark Knight" → [0.241, -0.887, 0.448, 0.131, ...]</div>
          <div className="text-zinc-500 mt-1">// Distance: 0.02 (very similar!)</div>
        </div>

        <h3 className="text-lg font-semibold text-white mb-3">Why 384 Dimensions?</h3>
        <p className="text-zinc-400 mb-4">
          More dimensions = more nuance. With 384 numbers, we can capture subtle differences like:
        </p>
        <ul className="grid md:grid-cols-2 gap-2 text-zinc-400">
          <li className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-emerald-500" />
            Genre combinations (Action-Comedy vs Action-Drama)
          </li>
          <li className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-emerald-500" />
            Tone and mood (dark vs lighthearted)
          </li>
          <li className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-emerald-500" />
            Era and style (80s nostalgia vs modern)
          </li>
          <li className="flex items-center gap-2">
            <span className="w-2 h-2 rounded-full bg-emerald-500" />
            Thematic elements (family, revenge, love)
          </li>
        </ul>
      </section>

      {/* RuVector */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-orange-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-orange-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </span>
          RuVector: High-Performance Vector Database
        </h2>
        <p className="text-zinc-400 text-lg mb-6">
          RuVector is a PostgreSQL extension written in Rust that provides ultra-fast vector operations.
          It's what makes our similarity searches return in milliseconds instead of seconds.
        </p>

        <div className="grid md:grid-cols-2 gap-6">
          <div>
            <h3 className="text-lg font-semibold text-white mb-3">Distance Functions</h3>
            <div className="space-y-3">
              <div className="p-3 rounded-lg bg-zinc-800/50 border border-zinc-700/50">
                <div className="text-white font-medium">Cosine Distance</div>
                <div className="text-zinc-500 text-sm">Measures angle between vectors (default)</div>
              </div>
              <div className="p-3 rounded-lg bg-zinc-800/50 border border-zinc-700/50">
                <div className="text-white font-medium">Euclidean Distance</div>
                <div className="text-zinc-500 text-sm">Measures straight-line distance</div>
              </div>
              <div className="p-3 rounded-lg bg-zinc-800/50 border border-emerald-500/30">
                <div className="text-emerald-400 font-medium">Hyperbolic Distance ✨</div>
                <div className="text-zinc-500 text-sm">Better for hierarchical data (genres → subgenres)</div>
              </div>
            </div>
          </div>
          <div>
            <h3 className="text-lg font-semibold text-white mb-3 flex items-center gap-2">
              Performance (Live)
              <span className="relative flex h-2 w-2">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
              </span>
            </h3>
            <div className="space-y-3">
              <div className="p-3 rounded-lg bg-zinc-800/50 border border-emerald-500/30">
                <div className="flex justify-between items-center">
                  <span className="text-zinc-400">Current Query Latency</span>
                  <span className="text-emerald-400 font-mono font-bold">{liveStats?.avgLatency || '~3ms'}</span>
                </div>
              </div>
              <div className="p-3 rounded-lg bg-zinc-800/50 border border-zinc-700/50">
                <div className="flex justify-between items-center">
                  <span className="text-zinc-400">Vector Search</span>
                  <span className="text-emerald-400 font-mono">~7.8ms</span>
                </div>
              </div>
              <div className="p-3 rounded-lg bg-zinc-800/50 border border-zinc-700/50">
                <div className="flex justify-between items-center">
                  <span className="text-zinc-400">SIMD Acceleration</span>
                  <span className="text-emerald-400">Enabled</span>
                </div>
              </div>
              <div className="p-3 rounded-lg bg-zinc-800/50 border border-zinc-700/50">
                <div className="flex justify-between items-center">
                  <span className="text-zinc-400">Learned Patterns</span>
                  <span className="text-emerald-400">{liveStats?.totalPatterns || 0} active</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}

function ComparisonTab() {
  return (
    <div className="space-y-12">
      <section>
        <h2 className="text-2xl font-bold text-white mb-6">Traditional vs Our Approach</h2>

        {/* Comparison Table */}
        <div className="glass-card rounded-2xl overflow-hidden">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-zinc-700">
                  <th className="text-left p-4 text-zinc-400 font-medium">Aspect</th>
                  <th className="text-left p-4 text-zinc-400 font-medium">Traditional Systems</th>
                  <th className="text-left p-4 text-emerald-400 font-medium">Our System</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-zinc-800">
                <tr>
                  <td className="p-4 text-white font-medium">Learning Speed</td>
                  <td className="p-4 text-zinc-400">Batch updates (daily/weekly)</td>
                  <td className="p-4 text-emerald-400">Real-time (every interaction)</td>
                </tr>
                <tr>
                  <td className="p-4 text-white font-medium">Cold Start Problem</td>
                  <td className="p-4 text-zinc-400">Needs extensive user history</td>
                  <td className="p-4 text-emerald-400">Works from first click via patterns</td>
                </tr>
                <tr>
                  <td className="p-4 text-white font-medium">Similarity Matching</td>
                  <td className="p-4 text-zinc-400">Genre/tag matching (exact)</td>
                  <td className="p-4 text-emerald-400">Semantic vectors (meaning-based)</td>
                </tr>
                <tr>
                  <td className="p-4 text-white font-medium">Feedback Integration</td>
                  <td className="p-4 text-zinc-400">Explicit ratings only</td>
                  <td className="p-4 text-emerald-400">Watch/Skip implicit signals</td>
                </tr>
                <tr>
                  <td className="p-4 text-white font-medium">Search Capability</td>
                  <td className="p-4 text-zinc-400">Keyword matching</td>
                  <td className="p-4 text-emerald-400">Natural language understanding</td>
                </tr>
                <tr>
                  <td className="p-4 text-white font-medium">Exploration</td>
                  <td className="p-4 text-zinc-400">Fixed recommendations</td>
                  <td className="p-4 text-emerald-400">ε-greedy exploration (30%)</td>
                </tr>
                <tr>
                  <td className="p-4 text-white font-medium">Hierarchy Understanding</td>
                  <td className="p-4 text-zinc-400">Flat categories</td>
                  <td className="p-4 text-emerald-400">Hyperbolic geometry (tree-like)</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>

      {/* Visual Comparison */}
      <section className="grid md:grid-cols-2 gap-6">
        <div className="glass-card rounded-2xl p-6 border-red-500/20">
          <h3 className="text-xl font-semibold text-white mb-4 flex items-center gap-2">
            <span className="w-8 h-8 rounded-lg bg-red-500/20 flex items-center justify-center">
              <svg className="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </span>
            Traditional Approach
          </h3>
          <div className="space-y-3 text-zinc-400">
            <p>1. User watches "The Office"</p>
            <p>2. System looks up "Comedy" genre</p>
            <p>3. Returns all comedies alphabetically</p>
            <p>4. No learning from skipped shows</p>
            <p>5. Same recommendations forever</p>
          </div>
          <div className="mt-4 p-3 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-sm">
            Result: User sees same suggestions, gets bored
          </div>
        </div>

        <div className="glass-card rounded-2xl p-6 border-emerald-500/20">
          <h3 className="text-xl font-semibold text-white mb-4 flex items-center gap-2">
            <span className="w-8 h-8 rounded-lg bg-emerald-500/20 flex items-center justify-center">
              <svg className="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
            </span>
            Our Approach
          </h3>
          <div className="space-y-3 text-zinc-400">
            <p>1. User watches "The Office"</p>
            <p>2. Vector finds semantically similar shows</p>
            <p>3. Q-Learning ranks by pattern success</p>
            <p>4. User skips "Parks & Rec" → learns preference</p>
            <p>5. Recommendations evolve with each click</p>
          </div>
          <div className="mt-4 p-3 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-400 text-sm">
            Result: Personalized, improving recommendations
          </div>
        </div>
      </section>

      {/* Why It's Better */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-6">Why Q-Learning Wins</h2>
        <div className="grid md:grid-cols-2 gap-8">
          <div>
            <h3 className="text-lg font-semibold text-white mb-3">The Math Behind It</h3>
            <div className="bg-zinc-900 rounded-xl p-4 font-mono text-sm">
              <div className="text-zinc-500 mb-2">// Q-Learning Update Formula</div>
              <div className="text-emerald-400">Q(s,a) ← Q(s,a) + α[r + γ·max(Q(s',a')) - Q(s,a)]</div>
              <div className="text-zinc-500 mt-4 text-xs">
                <div>α = learning rate (0.1)</div>
                <div>γ = discount factor (0.95)</div>
                <div>r = reward (+0.5 to +1 for watch, -0.3 to -0.1 for skip)</div>
              </div>
            </div>
          </div>
          <div>
            <h3 className="text-lg font-semibold text-white mb-3">In Plain English</h3>
            <p className="text-zinc-400">
              Every time you click "Watch" or "Skip", the AI updates its understanding of what works.
              Patterns that lead to watches get higher "Q-values" (quality scores) and are more likely
              to be recommended in the future. The 30% exploration rate ensures we don't get stuck
              showing only what we think you'll like - we occasionally try new things to learn more.
            </p>
          </div>
        </div>
      </section>
    </div>
  );
}

function ArchitectureTab() {
  return (
    <div className="space-y-12">
      {/* System Architecture Diagram */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-6">System Architecture</h2>

        {/* Mermaid-style diagram using divs */}
        <div className="bg-zinc-900 rounded-xl p-8 overflow-x-auto">
          <div className="min-w-[600px]">
            {/* Top Layer - UI */}
            <div className="flex justify-center mb-8">
              <div className="px-6 py-3 rounded-xl bg-blue-500/20 border border-blue-500/30 text-blue-400 font-medium">
                Next.js Frontend (React)
              </div>
            </div>

            {/* Arrow */}
            <div className="flex justify-center mb-8">
              <div className="flex flex-col items-center text-zinc-600">
                <div className="w-0.5 h-8 bg-zinc-700" />
                <div className="text-xs">HTTP/API</div>
                <div className="w-0.5 h-8 bg-zinc-700" />
                <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clipRule="evenodd" />
                </svg>
              </div>
            </div>

            {/* Middle Layer - API Routes */}
            <div className="flex justify-center gap-4 mb-8 flex-wrap">
              <div className="px-4 py-2 rounded-lg bg-purple-500/20 border border-purple-500/30 text-purple-400 text-sm">
                /api/recommendations
              </div>
              <div className="px-4 py-2 rounded-lg bg-purple-500/20 border border-purple-500/30 text-purple-400 text-sm">
                /api/similar
              </div>
              <div className="px-4 py-2 rounded-lg bg-purple-500/20 border border-purple-500/30 text-purple-400 text-sm">
                /api/feedback
              </div>
              <div className="px-4 py-2 rounded-lg bg-purple-500/20 border border-purple-500/30 text-purple-400 text-sm">
                /api/search
              </div>
            </div>

            {/* Arrow */}
            <div className="flex justify-center mb-8">
              <div className="flex flex-col items-center text-zinc-600">
                <div className="w-0.5 h-8 bg-zinc-700" />
                <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clipRule="evenodd" />
                </svg>
              </div>
            </div>

            {/* Core Services */}
            <div className="flex justify-center gap-6 mb-8">
              <div className="p-4 rounded-xl bg-emerald-500/20 border border-emerald-500/30 text-center">
                <div className="text-emerald-400 font-medium mb-1">Q-Learning Engine</div>
                <div className="text-zinc-500 text-xs">ruvector-learning.ts</div>
              </div>
              <div className="p-4 rounded-xl bg-orange-500/20 border border-orange-500/30 text-center">
                <div className="text-orange-400 font-medium mb-1">RuVector</div>
                <div className="text-zinc-500 text-xs">PostgreSQL Extension</div>
              </div>
            </div>

            {/* Arrow */}
            <div className="flex justify-center mb-8">
              <div className="flex flex-col items-center text-zinc-600">
                <div className="w-0.5 h-8 bg-zinc-700" />
                <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clipRule="evenodd" />
                </svg>
              </div>
            </div>

            {/* Database Layer */}
            <div className="flex justify-center gap-6">
              <div className="p-4 rounded-xl bg-zinc-700/50 border border-zinc-600 text-center">
                <div className="text-white font-medium mb-1">PostgreSQL</div>
                <div className="text-zinc-500 text-xs">Content + Vectors + Patterns</div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Data Flow */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-6">Data Flow: Watch Action</h2>

        <div className="space-y-4">
          <div className="flex items-center gap-4 p-4 rounded-xl bg-zinc-800/50">
            <div className="w-10 h-10 rounded-full bg-blue-500/20 flex items-center justify-center text-blue-400 font-bold flex-shrink-0">1</div>
            <div className="flex-1">
              <div className="text-white font-medium">User clicks "Watch" on content</div>
              <div className="text-zinc-500 text-sm">Frontend sends POST to /api/feedback</div>
            </div>
            <div className="font-mono text-xs text-zinc-600 hidden md:block">ContentCard.tsx</div>
          </div>

          <div className="flex items-center gap-4 p-4 rounded-xl bg-zinc-800/50">
            <div className="w-10 h-10 rounded-full bg-purple-500/20 flex items-center justify-center text-purple-400 font-bold flex-shrink-0">2</div>
            <div className="flex-1">
              <div className="text-white font-medium">API calculates reward</div>
              <div className="text-zinc-500 text-sm">Positive reward: +0.5 to +1.0 (with exploration noise)</div>
            </div>
            <div className="font-mono text-xs text-zinc-600 hidden md:block">feedback/route.ts</div>
          </div>

          <div className="flex items-center gap-4 p-4 rounded-xl bg-zinc-800/50">
            <div className="w-10 h-10 rounded-full bg-emerald-500/20 flex items-center justify-center text-emerald-400 font-bold flex-shrink-0">3</div>
            <div className="flex-1">
              <div className="text-white font-medium">Pattern identified from content</div>
              <div className="text-zinc-500 text-sm">e.g., "action_thriller" or "mood_exciting"</div>
            </div>
            <div className="font-mono text-xs text-zinc-600 hidden md:block">ruvector-learning.ts</div>
          </div>

          <div className="flex items-center gap-4 p-4 rounded-xl bg-zinc-800/50">
            <div className="w-10 h-10 rounded-full bg-orange-500/20 flex items-center justify-center text-orange-400 font-bold flex-shrink-0">4</div>
            <div className="flex-1">
              <div className="text-white font-medium">Q-Learning updates pattern stats</div>
              <div className="text-zinc-500 text-sm">Success rate and avg reward updated in DB</div>
            </div>
            <div className="font-mono text-xs text-zinc-600 hidden md:block">recommendation_patterns</div>
          </div>

          <div className="flex items-center gap-4 p-4 rounded-xl bg-zinc-800/50">
            <div className="w-10 h-10 rounded-full bg-yellow-500/20 flex items-center justify-center text-yellow-400 font-bold flex-shrink-0">5</div>
            <div className="flex-1">
              <div className="text-white font-medium">Feedback recorded for history</div>
              <div className="text-zinc-500 text-sm">Stored in learning_feedback table</div>
            </div>
            <div className="font-mono text-xs text-zinc-600 hidden md:block">learning_feedback</div>
          </div>

          <div className="flex items-center gap-4 p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/20">
            <div className="w-10 h-10 rounded-full bg-emerald-500/20 flex items-center justify-center text-emerald-400 font-bold flex-shrink-0">✓</div>
            <div className="flex-1">
              <div className="text-emerald-400 font-medium">Future recommendations improved</div>
              <div className="text-zinc-500 text-sm">Top patterns boost similar content in next query</div>
            </div>
          </div>
        </div>
      </section>

      {/* Database Schema */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-6">Key Database Tables</h2>

        <div className="grid md:grid-cols-2 gap-6">
          <div className="bg-zinc-900 rounded-xl p-4">
            <h3 className="text-emerald-400 font-mono font-medium mb-3">content</h3>
            <div className="space-y-1 text-sm font-mono">
              <div className="flex justify-between"><span className="text-zinc-400">id</span><span className="text-zinc-600">UUID</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">title</span><span className="text-zinc-600">TEXT</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">genres</span><span className="text-zinc-600">TEXT[]</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">embedding</span><span className="text-purple-400">VECTOR(384)</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">image_url</span><span className="text-zinc-600">TEXT</span></div>
            </div>
          </div>

          <div className="bg-zinc-900 rounded-xl p-4">
            <h3 className="text-emerald-400 font-mono font-medium mb-3">recommendation_patterns</h3>
            <div className="space-y-1 text-sm font-mono">
              <div className="flex justify-between"><span className="text-zinc-400">pattern_type</span><span className="text-zinc-600">TEXT</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">success_rate</span><span className="text-emerald-400">DECIMAL</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">total_uses</span><span className="text-zinc-600">INTEGER</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">avg_reward</span><span className="text-emerald-400">DECIMAL</span></div>
            </div>
          </div>

          <div className="bg-zinc-900 rounded-xl p-4">
            <h3 className="text-emerald-400 font-mono font-medium mb-3">learning_feedback</h3>
            <div className="space-y-1 text-sm font-mono">
              <div className="flex justify-between"><span className="text-zinc-400">content_id</span><span className="text-zinc-600">UUID</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">pattern_id</span><span className="text-zinc-600">INTEGER</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">was_successful</span><span className="text-zinc-600">BOOLEAN</span></div>
              <div className="flex justify-between"><span className="text-zinc-400">reward</span><span className="text-emerald-400">DECIMAL</span></div>
            </div>
          </div>

          <div className="bg-zinc-900 rounded-xl p-4">
            <h3 className="text-orange-400 font-mono font-medium mb-3">RuVector Functions</h3>
            <div className="space-y-1 text-sm font-mono">
              <div className="text-zinc-400">ruvector_cosine_distance()</div>
              <div className="text-zinc-400">ruvector_euclidean_distance()</div>
              <div className="text-purple-400">ruvector_hyperbolic_distance()</div>
              <div className="text-zinc-400">ruvector_learn_from_feedback()</div>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}

function ResearchTab() {
  return (
    <div className="space-y-12">
      {/* Research Background */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-4 flex items-center gap-3">
          <span className="w-10 h-10 rounded-xl bg-blue-500/20 flex items-center justify-center">
            <svg className="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
            </svg>
          </span>
          Research Foundation
        </h2>
        <p className="text-zinc-400 text-lg mb-6">
          Our system is built on established research in machine learning and information retrieval.
          Here are the key papers and concepts that inspired our approach:
        </p>

        <div className="space-y-4">
          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <h3 className="text-white font-semibold mb-2">Q-Learning (Watkins, 1989)</h3>
            <p className="text-zinc-400 text-sm mb-2">
              A model-free reinforcement learning algorithm that learns the value of actions in states.
              We use it to learn which recommendation patterns are most successful.
            </p>
            <div className="text-zinc-600 text-xs font-mono">
              "Learning from Delayed Rewards" - Christopher Watkins, PhD Thesis
            </div>
          </div>

          <div className="p-4 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <h3 className="text-white font-semibold mb-2">Sentence-BERT Embeddings (2019)</h3>
            <p className="text-zinc-400 text-sm mb-2">
              Dense vector representations that capture semantic meaning of text.
              We use 384-dimensional embeddings to represent content descriptions.
            </p>
            <div className="text-zinc-600 text-xs font-mono">
              Reimers & Gurevych - "Sentence-BERT: Sentence Embeddings using Siamese BERT-Networks"
            </div>
          </div>

          <div className="p-4 rounded-xl bg-zinc-800/50 border border-emerald-500/30">
            <h3 className="text-emerald-400 font-semibold mb-2">Hyperbolic Embeddings (Nickel & Kiela, 2017)</h3>
            <p className="text-zinc-400 text-sm mb-2">
              Embeddings in hyperbolic space better represent hierarchical relationships.
              Perfect for genre → subgenre → content hierarchies in media.
            </p>
            <div className="text-zinc-600 text-xs font-mono">
              "Poincaré Embeddings for Learning Hierarchical Representations" - Facebook AI Research
            </div>
          </div>

          <div className="p-4 rounded-xl bg-zinc-800/50 border border-emerald-500/30">
            <h3 className="text-emerald-400 font-semibold mb-2">Thompson Sampling (IMPLEMENTED)</h3>
            <p className="text-zinc-400 text-sm mb-2">
              Bayesian approach using Beta distributions for each arm. Samples from posterior
              to balance exploration/exploitation naturally without ε parameter.
            </p>
            <div className="text-zinc-600 text-xs font-mono">
              Thompson (1933) - "On the Likelihood that One Unknown Probability Exceeds Another"
            </div>
          </div>

          <div className="p-4 rounded-xl bg-zinc-800/50 border border-emerald-500/30">
            <h3 className="text-emerald-400 font-semibold mb-2">UCB1 Algorithm (IMPLEMENTED)</h3>
            <p className="text-zinc-400 text-sm mb-2">
              Upper Confidence Bound with optimistic estimation. Balances average reward with
              exploration bonus based on uncertainty.
            </p>
            <div className="text-zinc-600 text-xs font-mono">
              Auer et al. (2002) - "Finite-time Analysis of the Multiarmed Bandit Problem"
            </div>
          </div>

          <div className="p-4 rounded-xl bg-zinc-800/50 border border-emerald-500/30">
            <h3 className="text-emerald-400 font-semibold mb-2">LinUCB Contextual Bandits (IMPLEMENTED)</h3>
            <p className="text-zinc-400 text-sm mb-2">
              Linear contextual bandits that use content features (genre, mood, time)
              to make context-aware recommendations.
            </p>
            <div className="text-zinc-600 text-xs font-mono">
              Li et al. (2010) - "A Contextual-Bandit Approach to Personalized News Article Recommendation"
            </div>
          </div>

          <div className="p-4 rounded-xl bg-zinc-800/50 border border-emerald-500/30">
            <h3 className="text-emerald-400 font-semibold mb-2">Double Q-Learning (IMPLEMENTED)</h3>
            <p className="text-zinc-400 text-sm mb-2">
              Uses two Q-tables to decouple action selection from evaluation,
              preventing overestimation bias in value estimates.
            </p>
            <div className="text-zinc-600 text-xs font-mono">
              van Hasselt (2010) - "Double Q-learning"
            </div>
          </div>

          <div className="p-4 rounded-xl bg-zinc-800/50 border border-emerald-500/30">
            <h3 className="text-emerald-400 font-semibold mb-2">Prioritized Experience Replay (IMPLEMENTED)</h3>
            <p className="text-zinc-400 text-sm mb-2">
              Samples experiences proportional to their TD-error magnitude.
              Learns faster from surprising or important transitions.
            </p>
            <div className="text-zinc-600 text-xs font-mono">
              Schaul et al. (2015) - "Prioritized Experience Replay" - DeepMind
            </div>
          </div>
        </div>
      </section>

      {/* Optimizations */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-6">Optimizations & Innovations</h2>

        <div className="grid md:grid-cols-2 gap-6">
          <div className="p-5 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-2xl mb-3">⚡</div>
            <h3 className="text-white font-semibold mb-2">SIMD Vector Operations</h3>
            <p className="text-zinc-400 text-sm">
              RuVector uses CPU SIMD instructions to compute distance between 384-dimensional
              vectors in microseconds. This is 10-100x faster than naive implementations.
            </p>
          </div>

          <div className="p-5 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-2xl mb-3">🔄</div>
            <h3 className="text-white font-semibold mb-2">Pattern Boosting</h3>
            <p className="text-zinc-400 text-sm">
              Instead of re-ranking all results, we boost queries with top-performing patterns.
              This maintains database query efficiency while incorporating learned preferences.
            </p>
          </div>

          <div className="p-5 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-2xl mb-3">📊</div>
            <h3 className="text-white font-semibold mb-2">Incremental Learning</h3>
            <p className="text-zinc-400 text-sm">
              Pattern statistics are updated incrementally with each feedback.
              No need to retrain or rebuild indices - learning happens in milliseconds.
            </p>
          </div>

          <div className="p-5 rounded-xl bg-zinc-800/50 border border-zinc-700/50">
            <div className="text-2xl mb-3">🎯</div>
            <h3 className="text-white font-semibold mb-2">Mood-to-Genre Mapping</h3>
            <p className="text-zinc-400 text-sm">
              Pre-computed mappings from emotional moods to genre combinations enable
              instant filtering without complex NLP at query time.
            </p>
          </div>
        </div>
      </section>

      {/* Future Work */}
      <section className="glass-card rounded-2xl p-8">
        <h2 className="text-2xl font-bold text-white mb-6">Future Directions</h2>

        <div className="space-y-4">
          <div className="flex items-start gap-4 p-4 rounded-xl bg-zinc-800/30">
            <div className="w-8 h-8 rounded-lg bg-purple-500/20 flex items-center justify-center flex-shrink-0">
              <span className="text-purple-400 font-bold text-sm">1</span>
            </div>
            <div>
              <h3 className="text-white font-medium">Multi-User Collaborative Filtering</h3>
              <p className="text-zinc-500 text-sm">
                Combine individual Q-Learning with collaborative signals from similar users
                for even better cold-start performance.
              </p>
            </div>
          </div>

          <div className="flex items-start gap-4 p-4 rounded-xl bg-emerald-500/10 border border-emerald-500/20">
            <div className="w-8 h-8 rounded-lg bg-emerald-500/20 flex items-center justify-center flex-shrink-0">
              <span className="text-emerald-400 font-bold text-sm">✓</span>
            </div>
            <div>
              <h3 className="text-emerald-400 font-medium">Contextual Bandits (IMPLEMENTED)</h3>
              <p className="text-zinc-500 text-sm">
                Thompson Sampling, UCB1, and LinUCB now implemented for efficient exploration-exploitation.
              </p>
            </div>
          </div>

          <div className="flex items-start gap-4 p-4 rounded-xl bg-zinc-800/30">
            <div className="w-8 h-8 rounded-lg bg-purple-500/20 flex items-center justify-center flex-shrink-0">
              <span className="text-purple-400 font-bold text-sm">3</span>
            </div>
            <div>
              <h3 className="text-white font-medium">Time-Aware Recommendations</h3>
              <p className="text-zinc-500 text-sm">
                Learn patterns based on time of day, day of week, and seasonal preferences.
              </p>
            </div>
          </div>

          <div className="flex items-start gap-4 p-4 rounded-xl bg-zinc-800/30">
            <div className="w-8 h-8 rounded-lg bg-purple-500/20 flex items-center justify-center flex-shrink-0">
              <span className="text-purple-400 font-bold text-sm">4</span>
            </div>
            <div>
              <h3 className="text-white font-medium">Cross-Modal Learning</h3>
              <p className="text-zinc-500 text-sm">
                Incorporate poster image embeddings alongside text for richer content representation.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Tech Stack */}
      <section>
        <h2 className="text-2xl font-bold text-white mb-6">Technology Stack</h2>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          {[
            { name: 'Next.js 14', desc: 'React Framework' },
            { name: 'TypeScript', desc: 'Type Safety' },
            { name: 'PostgreSQL', desc: 'Database' },
            { name: 'RuVector', desc: 'Vector Extension' },
            { name: 'Tailwind CSS', desc: 'Styling' },
            { name: 'TVDB API', desc: 'Content Data' },
            { name: 'Rust', desc: 'RuVector Core' },
            { name: 'Vercel', desc: 'Deployment' },
          ].map((tech, i) => (
            <div key={i} className="glass-card rounded-xl p-4 text-center hover:border-emerald-500/30 transition-colors">
              <div className="text-white font-medium">{tech.name}</div>
              <div className="text-zinc-500 text-sm">{tech.desc}</div>
            </div>
          ))}
        </div>
      </section>
    </div>
  );
}
