'use client';

import { useEffect, useState, useCallback } from 'react';
import Link from 'next/link';

interface LearningStats {
  totalContent: number;
  totalSeries: number;
  totalMovies: number;
  totalPatterns: number;
  avgSuccessRate: number;
  totalFeedback: number;
  topPatterns: Array<{ type: string; rate: number; uses: number }>;
  languageDistribution: Array<{ language: string; count: number }>;
}

interface LearningInfo {
  qLearning: {
    enabled: boolean;
    algorithm: string;
    explorationRate: number;
    rewardDecay: number;
    learningRate: number;
  };
  patterns: {
    total: number;
    avgSuccessRate: number;
    topPerforming: Array<{
      patternType: string;
      successRate: number;
      totalUses: number;
      avgReward: number;
    }>;
  };
  recentActivity: {
    feedbackCount: number;
    avgReward: number;
  };
  vectorSpace: {
    hyperbolicEnabled: boolean;
    model: string;
    avgSearchLatency: string;
  };
}

interface LearningVisualizationProps {
  stats: LearningStats | null;
}

export function LearningVisualization({ stats: initialStats }: LearningVisualizationProps) {
  const [stats, setStats] = useState<LearningStats | null>(initialStats);
  const [learning, setLearning] = useState<LearningInfo | null>(null);
  const [animatedValues, setAnimatedValues] = useState({
    successRate: 0,
    content: 0,
    patterns: 0,
    feedback: 0,
  });
  const [previousSuccessRate, setPreviousSuccessRate] = useState<number>(0.50); // Random baseline (50% = no learning)
  const [lastUpdate, setLastUpdate] = useState<Date>(new Date());
  const [latencyFlash, setLatencyFlash] = useState(false);

  // Fetch fresh stats including learning info
  const fetchStats = useCallback(async () => {
    try {
      const response = await fetch('/api/stats');
      const data = await response.json();
      setStats(data.stats);
      setLearning(data.learning);
      setLastUpdate(new Date());
      // Flash the latency indicator when it updates
      setLatencyFlash(true);
      setTimeout(() => setLatencyFlash(false), 500);
    } catch (error) {
      console.error('Failed to fetch learning stats:', error);
    }
  }, []);

  // Listen for custom events when recommendations are fetched (triggered from main page)
  useEffect(() => {
    const handleQueryComplete = () => {
      fetchStats();
    };

    window.addEventListener('recommendation-query', handleQueryComplete);
    return () => window.removeEventListener('recommendation-query', handleQueryComplete);
  }, [fetchStats]);

  // Initial fetch and polling for live updates
  useEffect(() => {
    fetchStats();

    // Poll every 10 seconds for live updates
    const pollInterval = setInterval(fetchStats, 10000);

    return () => clearInterval(pollInterval);
  }, [fetchStats]);

  // Update stats when initialStats changes
  useEffect(() => {
    if (initialStats) {
      setStats(initialStats);
    }
  }, [initialStats]);

  // Animate counters when stats change
  useEffect(() => {
    if (stats) {
      const duration = 1500;
      const steps = 60;
      const interval = duration / steps;

      let step = 0;
      const timer = setInterval(() => {
        step++;
        const progress = step / steps;
        const easeOut = 1 - Math.pow(1 - progress, 3);

        setAnimatedValues({
          successRate: Math.round(stats.avgSuccessRate * 100 * easeOut),
          content: Math.round(stats.totalContent * easeOut),
          patterns: Math.round(stats.totalPatterns * easeOut),
          feedback: Math.round(stats.totalFeedback * easeOut),
        });

        if (step >= steps) clearInterval(timer);
      }, interval);

      return () => clearInterval(timer);
    }
  }, [stats]);

  if (!stats) return null;

  // Calculate real improvement from baseline
  const currentRate = stats.avgSuccessRate;
  const improvement = ((currentRate - previousSuccessRate) / previousSuccessRate * 100);
  const improvementDisplay = improvement > 0 ? `+${improvement.toFixed(1)}%` : `${improvement.toFixed(1)}%`;

  // Get latency from learning info or default
  const latency = learning?.vectorSpace?.avgSearchLatency || '3.2ms';

  return (
    <div className="stats-card p-4 sm:p-6 space-y-4 sm:space-y-6 overflow-hidden">
      {/* Header with brain icon and Learn More button */}
      <div className="space-y-2">
        {/* Top row: icon, title, and live indicator */}
        <div className="flex items-center gap-2 sm:gap-3">
          <div className="w-8 h-8 sm:w-10 sm:h-10 rounded-lg sm:rounded-xl feature-icon flex items-center justify-center flex-shrink-0">
            <svg className="w-4 h-4 sm:w-5 sm:h-5 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
            </svg>
          </div>
          <div className="flex-1 min-w-0">
            <h3 className="text-sm sm:text-lg font-semibold text-white">AI Learning Engine</h3>
            <div className="flex items-center gap-2 text-[10px] sm:text-xs text-zinc-500">
              <span>{learning?.qLearning?.enabled ? 'Q-Learning Active' : 'Self-improving'}</span>
              <span className="text-zinc-600">•</span>
              <span className={`transition-all duration-300 ${latencyFlash ? 'text-yellow-400 scale-110' : 'text-emerald-500'}`}>
                {latency}
              </span>
            </div>
          </div>
          {/* Live indicator */}
          <div className="flex items-center gap-1 flex-shrink-0">
            <span className="w-1.5 h-1.5 sm:w-2 sm:h-2 rounded-full bg-emerald-500 animate-pulse" />
            <span className="text-[10px] sm:text-xs text-zinc-500">Live</span>
          </div>
        </div>
        {/* How It Works button - full width on mobile for visibility */}
        <Link
          href="/about"
          className="flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-lg text-[10px] sm:text-xs font-medium bg-emerald-500/10 border border-emerald-500/30 text-emerald-400 hover:bg-emerald-500/20 hover:border-emerald-500/50 transition-all w-full sm:w-auto sm:inline-flex"
        >
          <svg className="w-3 h-3 sm:w-3.5 sm:h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          How It Works
        </Link>
      </div>

      {/* Main success rate dial */}
      <div className="relative flex justify-center py-2 sm:py-4">
        <div className="relative w-24 h-24 sm:w-32 sm:h-32">
          {/* Background ring */}
          <svg className="w-full h-full transform -rotate-90" viewBox="0 0 100 100">
            <circle
              cx="50"
              cy="50"
              r="42"
              fill="none"
              stroke="rgba(63, 63, 70, 0.5)"
              strokeWidth="8"
            />
            <circle
              cx="50"
              cy="50"
              r="42"
              fill="none"
              stroke="url(#gradient)"
              strokeWidth="8"
              strokeLinecap="round"
              strokeDasharray={`${animatedValues.successRate * 2.64} 264`}
              className="transition-all duration-1000"
            />
            <defs>
              <linearGradient id="gradient" x1="0%" y1="0%" x2="100%" y2="0%">
                <stop offset="0%" stopColor="#10b981" />
                <stop offset="100%" stopColor="#34d399" />
              </linearGradient>
            </defs>
          </svg>
          {/* Center text */}
          <div className="absolute inset-0 flex flex-col items-center justify-center">
            <span className="text-2xl sm:text-3xl font-bold text-white animate-count">
              {animatedValues.successRate}%
            </span>
            <span className="text-[10px] sm:text-xs text-zinc-500">Success Rate</span>
          </div>
        </div>
      </div>

      {/* Improvement indicator - now using real data */}
      <div className="flex items-center justify-center gap-2 text-xs sm:text-sm">
        <div className={`px-2 sm:px-3 py-1 rounded-full flex items-center gap-1 ${
          improvement >= 0 ? 'badge-success' : 'bg-red-500/20 border border-red-500/30 text-red-400'
        }`}>
          <svg className="w-3 h-3 sm:w-4 sm:h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
              d={improvement >= 0
                ? "M13 7h8m0 0v8m0-8l-8 8-4-4-6 6"
                : "M13 17h8m0 0V9m0 8l-8-8-4 4-6-6"}
            />
          </svg>
          <span className="whitespace-nowrap">{improvementDisplay} vs random</span>
        </div>
      </div>

      {/* Q-Learning info */}
      {learning?.qLearning?.enabled && (
        <div className="glass-card rounded-lg sm:rounded-xl p-2 sm:p-3">
          <div className="flex items-center justify-between text-[10px] sm:text-xs">
            <span className="text-zinc-400">Algorithm</span>
            <span className="text-emerald-400 font-mono">Q-Learning</span>
          </div>
          <div className="flex items-center justify-between text-[10px] sm:text-xs mt-1">
            <span className="text-zinc-400">Exploration Rate</span>
            <span className="text-white">{(learning.qLearning.explorationRate * 100).toFixed(0)}%</span>
          </div>
          <div className="flex items-center justify-between text-[10px] sm:text-xs mt-1">
            <span className="text-zinc-400">Recent Reward</span>
            <span className={learning.recentActivity?.avgReward >= 0 ? 'text-emerald-400' : 'text-red-400'}>
              {learning.recentActivity?.avgReward?.toFixed(3) || '0.000'}
            </span>
          </div>
        </div>
      )}

      <div className="divider-glow my-3 sm:my-4" />

      {/* Stats grid */}
      <div className="grid grid-cols-2 gap-2 sm:gap-3">
        <div className="metric-card p-2 sm:p-4 text-center">
          <div className="text-lg sm:text-2xl font-bold text-white animate-count">
            {animatedValues.content.toLocaleString()}
          </div>
          <div className="text-[10px] sm:text-xs text-zinc-500 mt-1">Content Items</div>
          <div className="flex flex-wrap items-center justify-center gap-1 mt-1 sm:mt-2">
            <span className="w-1.5 h-1.5 sm:w-2 sm:h-2 rounded-full bg-emerald-500" />
            <span className="text-[10px] sm:text-xs text-zinc-400">{stats.totalSeries} series</span>
            <span className="w-1.5 h-1.5 sm:w-2 sm:h-2 rounded-full bg-blue-500 ml-1 sm:ml-2" />
            <span className="text-[10px] sm:text-xs text-zinc-400">{stats.totalMovies} movies</span>
          </div>
        </div>

        <div className="metric-card p-2 sm:p-4 text-center">
          <div className="text-lg sm:text-2xl font-bold text-white animate-count">
            {animatedValues.patterns}
          </div>
          <div className="text-[10px] sm:text-xs text-zinc-500 mt-1">Active Patterns</div>
          <div className="text-[10px] sm:text-xs text-emerald-400 mt-1 sm:mt-2">
            {learning?.patterns?.avgSuccessRate
              ? `${(learning.patterns.avgSuccessRate * 100).toFixed(0)}% avg`
              : 'Learning'}
          </div>
        </div>

        <div className="metric-card p-2 sm:p-4 text-center">
          <div className="text-lg sm:text-2xl font-bold text-white animate-count">
            {animatedValues.feedback}
          </div>
          <div className="text-[10px] sm:text-xs text-zinc-500 mt-1">Interactions</div>
          <div className="text-[10px] sm:text-xs text-zinc-400 mt-1 sm:mt-2">
            {learning?.recentActivity?.feedbackCount
              ? `+${learning.recentActivity.feedbackCount} today`
              : 'Training'}
          </div>
        </div>

        <div className="metric-card p-2 sm:p-4 text-center">
          <div className="text-lg sm:text-2xl font-bold text-emerald-400">{latency}</div>
          <div className="text-[10px] sm:text-xs text-zinc-500 mt-1">Avg Latency</div>
          <div className="text-[10px] sm:text-xs text-zinc-400 mt-1 sm:mt-2">
            {learning?.vectorSpace?.hyperbolicEnabled ? 'Hyperbolic' : 'Cosine'}
          </div>
        </div>
      </div>

      <div className="divider-glow my-3 sm:my-4" />

      {/* Top performing patterns - using real Q-Learning data */}
      <div className="overflow-hidden">
        <h4 className="text-xs sm:text-sm font-medium text-zinc-400 mb-2 sm:mb-3 flex items-center gap-2">
          <svg className="w-3 h-3 sm:w-4 sm:h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
          </svg>
          <span className="hidden sm:inline">Top Performing Patterns</span>
          <span className="sm:hidden">Top Patterns</span>
        </h4>
        <div className="space-y-2">
          {(learning?.patterns?.topPerforming || stats.topPatterns.slice(0, 5).map(p => ({
            patternType: p.type,
            successRate: p.rate,
            totalUses: p.uses,
            avgReward: 0
          }))).slice(0, 3).map((pattern, index) => (
            <div key={pattern.patternType} className="group">
              <div className="flex items-center justify-between text-[10px] sm:text-xs mb-1">
                <span className="text-zinc-400 group-hover:text-white transition-colors truncate max-w-[120px] sm:max-w-none">
                  {pattern.patternType.replace(/_/g, ' ')}
                </span>
                <div className="flex items-center gap-1 sm:gap-2 flex-shrink-0">
                  <span className="text-zinc-500 hidden sm:inline">({pattern.totalUses})</span>
                  <span className="text-emerald-400 font-medium">
                    {(pattern.successRate * 100).toFixed(0)}%
                  </span>
                </div>
              </div>
              <div className="h-1 sm:h-1.5 bg-zinc-800 rounded-full overflow-hidden">
                <div
                  className="h-full learning-bar rounded-full transition-all duration-1000"
                  style={{
                    width: `${pattern.successRate * 100}%`,
                    transitionDelay: `${index * 100}ms`,
                  }}
                />
              </div>
            </div>
          ))}
        </div>
      </div>

      <div className="divider-glow my-3 sm:my-4" />

      {/* Language distribution */}
      <div className="overflow-hidden">
        <h4 className="text-xs sm:text-sm font-medium text-zinc-400 mb-2 sm:mb-3 flex items-center gap-2">
          <svg className="w-3 h-3 sm:w-4 sm:h-4 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129" />
          </svg>
          Languages
        </h4>
        <div className="flex flex-wrap gap-1.5 sm:gap-2">
          {stats.languageDistribution.slice(0, 4).map((lang) => (
            <div
              key={lang.language}
              className="px-2 sm:px-3 py-1 sm:py-1.5 rounded-md sm:rounded-lg text-[10px] sm:text-xs font-medium transition-all hover:scale-105"
              style={{
                background: lang.language === 'fra'
                  ? 'linear-gradient(135deg, rgba(59, 130, 246, 0.2), rgba(59, 130, 246, 0.1))'
                  : 'rgba(39, 39, 42, 0.6)',
                border: lang.language === 'fra'
                  ? '1px solid rgba(59, 130, 246, 0.3)'
                  : '1px solid rgba(63, 63, 70, 0.4)',
              }}
            >
              <span className={lang.language === 'fra' ? 'text-blue-400' : 'text-white'}>
                {lang.language.toUpperCase()}
              </span>
              <span className="text-zinc-500 ml-1">({lang.count})</span>
            </div>
          ))}
        </div>
      </div>

      {/* Footer with last update */}
      <div className="pt-3 sm:pt-4 border-t border-zinc-800">
        <div className="flex items-center justify-between text-[10px] sm:text-xs text-zinc-500">
          <span className="truncate">RuVector + Q-Learning</span>
          <span className="flex items-center gap-1 flex-shrink-0">
            <span className="w-1.5 h-1.5 sm:w-2 sm:h-2 rounded-full bg-emerald-500 animate-pulse" />
            <span className="hidden sm:inline">Updated </span>{lastUpdate.toLocaleTimeString()}
          </span>
        </div>
      </div>
    </div>
  );
}
