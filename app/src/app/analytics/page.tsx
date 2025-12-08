'use client';

import { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import Link from 'next/link';
import dynamic from 'next/dynamic';
import {
  LineChart, Line, AreaChart, Area, BarChart, Bar,
  XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer,
  RadarChart, PolarGrid, PolarAngleAxis, PolarRadiusAxis, Radar,
  Treemap, Cell, Legend, ComposedChart, Scatter, PieChart, Pie,
  ScatterChart, ZAxis
} from 'recharts';

// Dynamic import for force graph (SSR incompatible)
const ForceGraph2D = dynamic(() => import('react-force-graph-2d'), { ssr: false });

interface AnalyticsData {
  overview: {
    totalContent: number;
    series: number;
    movies: number;
    avgRating: number;
    embeddingCoverage: number;
    totalFeedback: number;
    positiveRate: number;
  };
  patterns: Array<{
    type: string;
    successRate: number;
    uses: number;
    avgReward: number;
  }>;
  feedbackTimeline: Array<{
    hour: string;
    count: number;
    positive: number;
    avgReward: number;
  }>;
  genreNetwork: {
    nodes: Array<{ id: string; name: string; val: number }>;
    edges: Array<{ source: string; target: string; weight: number }>;
  };
  learningMetrics: Array<{
    pattern: string;
    successRate: number;
    uses: number;
    avgReward: number;
    freshness: number;
  }>;
  algorithms: Array<{
    name: string;
    uses: number;
    successRate: number;
    avgReward: number;
  }>;
  contentByYear: Array<{
    year: number;
    type: string;
    count: number;
  }>;
  recentActivity: Array<{
    id: string;
    success: boolean;
    reward: number;
    action: string;
    time: string;
    title: string;
    contentType: string;
    pattern: string;
  }>;
}

const COLORS = {
  emerald: '#10b981',
  emeraldLight: '#34d399',
  blue: '#3b82f6',
  purple: '#8b5cf6',
  pink: '#ec4899',
  orange: '#f59e0b',
  red: '#ef4444',
  cyan: '#06b6d4',
  zinc: '#71717a',
};

const GRADIENT_COLORS = [
  COLORS.emerald, COLORS.blue, COLORS.purple, COLORS.pink, COLORS.orange, COLORS.cyan
];

// Animated counter component
function AnimatedCounter({ value, suffix = '', decimals = 0, duration = 1000 }: { value: number; suffix?: string; decimals?: number; duration?: number }) {
  const [displayValue, setDisplayValue] = useState(0);

  useEffect(() => {
    const startTime = Date.now();
    const startValue = displayValue;

    const animate = () => {
      const now = Date.now();
      const progress = Math.min((now - startTime) / duration, 1);
      const eased = 1 - Math.pow(1 - progress, 3); // ease-out cubic
      const current = startValue + (value - startValue) * eased;
      setDisplayValue(current);

      if (progress < 1) {
        requestAnimationFrame(animate);
      }
    };

    requestAnimationFrame(animate);
  }, [value, duration]);

  return <span>{displayValue.toFixed(decimals)}{suffix}</span>;
}

// Pulse ring animation component
function PulseRing({ color, size = 'md' }: { color: string; size?: 'sm' | 'md' | 'lg' }) {
  const sizes = { sm: 'w-2 h-2', md: 'w-3 h-3', lg: 'w-4 h-4' };
  return (
    <span className="relative flex">
      <span className={`animate-ping absolute inline-flex h-full w-full rounded-full opacity-75`} style={{ backgroundColor: color }} />
      <span className={`relative inline-flex rounded-full ${sizes[size]}`} style={{ backgroundColor: color }} />
    </span>
  );
}

// Learning state diagram component
function LearningStateDiagram({ explorationRate = 0.3, learningRate = 0.1 }: { explorationRate?: number; learningRate?: number }) {
  const states = [
    { id: 'explore', label: 'Explore', x: 50, y: 30, color: COLORS.blue, active: Math.random() < explorationRate },
    { id: 'exploit', label: 'Exploit', x: 150, y: 30, color: COLORS.emerald, active: Math.random() >= explorationRate },
    { id: 'learn', label: 'Learn', x: 100, y: 100, color: COLORS.purple, active: true },
  ];

  return (
    <div className="relative w-full h-40">
      <svg className="absolute inset-0 w-full h-full">
        {/* Connection lines */}
        <defs>
          <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
            <polygon points="0 0, 10 3.5, 0 7" fill={COLORS.zinc} />
          </marker>
        </defs>
        <line x1="70" y1="50" x2="130" y2="50" stroke={COLORS.zinc} strokeWidth="2" markerEnd="url(#arrowhead)" strokeDasharray="4" />
        <line x1="130" y1="50" x2="70" y2="50" stroke={COLORS.zinc} strokeWidth="2" markerEnd="url(#arrowhead)" strokeDasharray="4" />
        <line x1="60" y1="55" x2="90" y2="90" stroke={COLORS.zinc} strokeWidth="2" markerEnd="url(#arrowhead)" />
        <line x1="140" y1="55" x2="110" y2="90" stroke={COLORS.zinc} strokeWidth="2" markerEnd="url(#arrowhead)" />
      </svg>
      {states.map((state) => (
        <div
          key={state.id}
          className={`absolute transform -translate-x-1/2 -translate-y-1/2 transition-all duration-500 ${state.active ? 'scale-110' : 'scale-100 opacity-60'}`}
          style={{ left: `${state.x}px`, top: `${state.y}px` }}
        >
          <div
            className="w-16 h-16 rounded-full flex items-center justify-center text-xs font-medium text-white shadow-lg"
            style={{
              backgroundColor: state.color,
              boxShadow: state.active ? `0 0 0 4px ${state.color}40` : 'none'
            }}
          >
            {state.label}
          </div>
        </div>
      ))}
      <div className="absolute bottom-0 left-0 right-0 text-center text-xs text-zinc-500">
        ε = {(explorationRate * 100).toFixed(0)}% | α = {learningRate}
      </div>
    </div>
  );
}

// Reward distribution sparkline
function RewardSparkline({ data }: { data: number[] }) {
  const min = Math.min(...data);
  const max = Math.max(...data);
  const range = max - min || 1;

  return (
    <div className="flex items-end gap-0.5 h-8">
      {data.slice(-20).map((value, i) => (
        <div
          key={i}
          className={`flex-1 rounded-t transition-all ${value >= 0 ? 'bg-emerald-500' : 'bg-red-500'}`}
          style={{ height: `${((value - min) / range) * 100}%`, minHeight: 2, opacity: 0.3 + (i / 20) * 0.7 }}
        />
      ))}
    </div>
  );
}

export default function AnalyticsPage() {
  const [data, setData] = useState<AnalyticsData | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [selectedTab, setSelectedTab] = useState<'overview' | 'learning' | 'network' | 'activity'>('overview');
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [animationKey, setAnimationKey] = useState(0);
  const graphRef = useRef<any>(null);

  const fetchData = useCallback(async () => {
    try {
      const response = await fetch('/api/analytics');
      const result = await response.json();
      setData(result);
      setAnimationKey(k => k + 1);
    } catch (error) {
      console.error('Failed to fetch analytics:', error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchData();
    if (autoRefresh) {
      const interval = setInterval(fetchData, 5000);
      return () => clearInterval(interval);
    }
  }, [fetchData, autoRefresh]);

  // Process content by year for stacked area chart
  const contentByYearData = useMemo(() => {
    if (!data?.contentByYear) return [];
    const byYear: Record<number, { year: number; series: number; movie: number }> = {};
    data.contentByYear.forEach(({ year, type, count }) => {
      if (!byYear[year]) byYear[year] = { year, series: 0, movie: 0 };
      byYear[year][type as 'series' | 'movie'] = count;
    });
    return Object.values(byYear).sort((a, b) => a.year - b.year);
  }, [data?.contentByYear]);

  // Process patterns for radar chart
  const radarData = useMemo(() => {
    if (!data?.learningMetrics) return [];
    return data.learningMetrics.slice(0, 8).map(m => ({
      pattern: m.pattern.replace(/_/g, ' ').slice(0, 12),
      successRate: m.successRate * 100,
      uses: Math.min(m.uses / 10, 100),
      reward: (m.avgReward + 1) * 50,
    }));
  }, [data?.learningMetrics]);

  // Graph data for force-directed network
  const graphData = useMemo(() => {
    if (!data?.genreNetwork) return { nodes: [], links: [] };
    return {
      nodes: data.genreNetwork.nodes.map((n, i) => ({
        ...n,
        color: GRADIENT_COLORS[i % GRADIENT_COLORS.length],
      })),
      links: data.genreNetwork.edges.map(e => ({
        source: e.source,
        target: e.target,
        value: e.weight,
      })),
    };
  }, [data?.genreNetwork]);

  // Pattern treemap data
  const treemapData = useMemo(() => {
    if (!data?.patterns) return [];
    return data.patterns.slice(0, 15).map((p, i) => ({
      name: p.type.replace(/_/g, ' '),
      size: p.uses,
      successRate: p.successRate,
      color: GRADIENT_COLORS[i % GRADIENT_COLORS.length],
    }));
  }, [data?.patterns]);

  // Algorithm comparison scatter data
  const algorithmScatterData = useMemo(() => {
    if (!data?.algorithms) return [];
    return data.algorithms.map((a, i) => ({
      name: a.name?.split(':')[0] || 'Unknown',
      successRate: a.successRate * 100,
      uses: a.uses,
      reward: (a.avgReward + 1) * 50,
      color: GRADIENT_COLORS[i % GRADIENT_COLORS.length],
    }));
  }, [data?.algorithms]);

  // Genre pie data
  const genrePieData = useMemo(() => {
    if (!data?.genreNetwork?.nodes) return [];
    return data.genreNetwork.nodes
      .sort((a, b) => b.val - a.val)
      .slice(0, 8)
      .map((n, i) => ({
        name: n.name,
        value: n.val,
        color: GRADIENT_COLORS[i % GRADIENT_COLORS.length],
      }));
  }, [data?.genreNetwork]);

  // Recent rewards for sparkline
  const recentRewards = useMemo(() => {
    if (!data?.recentActivity) return [];
    return data.recentActivity.map(a => a.reward);
  }, [data?.recentActivity]);

  if (isLoading) {
    return (
      <main className="min-h-screen p-6 md:p-12 flex items-center justify-center">
        <div className="text-center">
          <div className="w-16 h-16 rounded-2xl feature-icon flex items-center justify-center mx-auto mb-4 animate-pulse-glow">
            <svg className="w-8 h-8 text-emerald-400 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
            </svg>
          </div>
          <p className="text-zinc-400">Loading analytics...</p>
        </div>
      </main>
    );
  }

  return (
    <main className="min-h-screen relative z-10">
      {/* Sticky Navigation */}
      <nav className="sticky top-0 z-50 glass-card border-b border-zinc-800/50 backdrop-blur-xl">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 py-3 flex items-center justify-between">
          <Link href="/" className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-lg feature-icon flex items-center justify-center">
              <svg className="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
            </div>
            <span className="font-bold text-white hidden sm:inline">TVDB Smart</span>
          </Link>

          <div className="flex items-center gap-2 sm:gap-4">
            <Link
              href="/"
              className="px-3 py-1.5 rounded-lg text-xs sm:text-sm font-medium text-zinc-400 hover:text-white hover:bg-zinc-800 transition-all"
            >
              Dashboard
            </Link>
            <Link
              href="/analytics"
              className="px-3 py-1.5 rounded-lg text-xs sm:text-sm font-medium text-emerald-400 bg-emerald-500/10 border border-emerald-500/20"
            >
              Analytics
            </Link>
            <Link
              href="/about"
              className="px-3 py-1.5 rounded-lg text-xs sm:text-sm font-medium text-zinc-400 hover:text-white hover:bg-zinc-800 transition-all"
            >
              About
            </Link>
          </div>
        </div>
      </nav>

      <div className="p-4 sm:p-6 md:p-12">
      {/* Background effects */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none z-0">
        <div className="glow-orb w-96 h-96 bg-emerald-500 top-20 -left-48 opacity-30" />
        <div className="glow-orb w-80 h-80 bg-blue-500 top-1/3 -right-40 opacity-30" />
        <div className="glow-orb w-64 h-64 bg-purple-500 bottom-20 left-1/4 opacity-30" />
      </div>

      {/* Header */}
      <header className="mb-8">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 mb-6">
          <div className="flex items-center gap-4">
            <div>
              <h1 className="text-2xl sm:text-3xl font-bold gradient-text flex items-center gap-3">
                Analytics Dashboard
                <PulseRing color={COLORS.emerald} size="sm" />
              </h1>
              <p className="text-zinc-500 text-sm">Real-time Q-Learning system performance</p>
            </div>
          </div>

          <div className="flex items-center gap-3">
            <button
              onClick={() => setAutoRefresh(!autoRefresh)}
              className={`flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-medium transition-all ${
                autoRefresh
                  ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
                  : 'glass-card text-zinc-400'
              }`}
            >
              <span className={`w-2 h-2 rounded-full ${autoRefresh ? 'bg-emerald-500 animate-pulse' : 'bg-zinc-500'}`} />
              {autoRefresh ? 'Live' : 'Paused'}
            </button>
            <button
              onClick={fetchData}
              className="p-2 rounded-xl glass-card hover:bg-zinc-700 transition-all"
            >
              <svg className="w-5 h-5 text-zinc-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
            </button>
          </div>
        </div>

        {/* Tab navigation */}
        <div className="flex gap-2 overflow-x-auto pb-2">
          {[
            { id: 'overview', label: 'Overview', icon: 'M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z' },
            { id: 'learning', label: 'Learning', icon: 'M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z' },
            { id: 'network', label: 'Network Graph', icon: 'M14 10l-2 1m0 0l-2-1m2 1v2.5M20 7l-2 1m2-1l-2-1m2 1v2.5M14 4l-2-1-2 1M4 7l2-1M4 7l2 1M4 7v2.5M12 21l-2-1m2 1l2-1m-2 1v-2.5M6 18l-2-1v-2.5M18 18l2-1v-2.5' },
            { id: 'activity', label: 'Activity', icon: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z' },
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setSelectedTab(tab.id as any)}
              className={`flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-medium transition-all whitespace-nowrap ${
                selectedTab === tab.id
                  ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
                  : 'glass-card text-zinc-400 hover:text-white'
              }`}
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={tab.icon} />
              </svg>
              {tab.label}
            </button>
          ))}
        </div>
      </header>

      {/* Overview Tab */}
      {selectedTab === 'overview' && data && (
        <div className="space-y-6" key={animationKey}>
          {/* Key metrics with animated counters */}
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
            {[
              { label: 'Total Content', value: data.overview.totalContent, color: 'emerald', icon: 'M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4' },
              { label: 'Total Feedback', value: data.overview.totalFeedback, color: 'blue', icon: 'M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z' },
              { label: 'Success Rate', value: data.overview.positiveRate * 100, suffix: '%', decimals: 1, color: 'purple', icon: 'M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z' },
              { label: 'Vector Coverage', value: data.overview.embeddingCoverage, suffix: '%', decimals: 0, color: 'orange', icon: 'M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z' },
            ].map((metric) => (
              <div key={metric.label} className="stats-card p-4 group hover:scale-[1.02] transition-transform">
                <div className="flex items-center gap-3 mb-2">
                  <div className={`w-10 h-10 rounded-xl bg-${metric.color}-500/20 flex items-center justify-center group-hover:scale-110 transition-transform`}>
                    <svg className={`w-5 h-5 text-${metric.color}-400`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d={metric.icon} />
                    </svg>
                  </div>
                </div>
                <div className="text-2xl font-bold text-white">
                  <AnimatedCounter value={metric.value} suffix={metric.suffix || ''} decimals={metric.decimals || 0} />
                </div>
                <div className="text-xs text-zinc-500">{metric.label}</div>
              </div>
            ))}
          </div>

          {/* Charts row */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Feedback timeline with gradient */}
            <div className="stats-card p-4">
              <h3 className="text-sm font-medium text-zinc-400 mb-4 flex items-center gap-2">
                <svg className="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
                </svg>
                Learning Progress (7 days)
                <span className="ml-auto flex items-center gap-1">
                  <span className="w-2 h-2 rounded-full bg-emerald-500"></span>
                  <span className="text-xs">Total</span>
                  <span className="w-2 h-2 rounded-full bg-blue-500 ml-2"></span>
                  <span className="text-xs">Positive</span>
                </span>
              </h3>
              <div className="h-64">
                <ResponsiveContainer width="100%" height="100%">
                  <ComposedChart data={data.feedbackTimeline.slice(-48)}>
                    <defs>
                      <linearGradient id="feedbackGradient" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor={COLORS.emerald} stopOpacity={0.4} />
                        <stop offset="50%" stopColor={COLORS.emerald} stopOpacity={0.1} />
                        <stop offset="100%" stopColor={COLORS.emerald} stopOpacity={0} />
                      </linearGradient>
                      <linearGradient id="positiveGradient" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor={COLORS.blue} stopOpacity={0.4} />
                        <stop offset="100%" stopColor={COLORS.blue} stopOpacity={0} />
                      </linearGradient>
                    </defs>
                    <CartesianGrid strokeDasharray="3 3" stroke="#27272a" />
                    <XAxis
                      dataKey="hour"
                      tick={{ fill: '#71717a', fontSize: 10 }}
                      tickFormatter={(v) => new Date(v).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                    />
                    <YAxis tick={{ fill: '#71717a', fontSize: 10 }} />
                    <Tooltip
                      content={({ payload, label }) => {
                        if (!payload || payload.length === 0) return null;
                        const data = payload[0]?.payload;
                        return (
                          <div className="bg-zinc-900 border border-zinc-700 rounded-lg p-3 shadow-xl">
                            <div className="text-xs text-zinc-400 mb-2">
                              {new Date(label).toLocaleString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })}
                            </div>
                            <div className="flex items-center gap-2 text-sm">
                              <span className="w-2 h-2 rounded-full bg-emerald-500" />
                              <span className="text-zinc-400">Total:</span>
                              <span className="text-emerald-400 font-bold">{data?.count}</span>
                            </div>
                            <div className="flex items-center gap-2 text-sm">
                              <span className="w-2 h-2 rounded-full bg-blue-500" />
                              <span className="text-zinc-400">Positive:</span>
                              <span className="text-blue-400 font-bold">{data?.positive}</span>
                            </div>
                            <div className="text-xs text-zinc-500 mt-1">
                              Success: {data?.count > 0 ? ((data?.positive / data?.count) * 100).toFixed(0) : 0}%
                            </div>
                          </div>
                        );
                      }}
                    />
                    <Area
                      type="monotone"
                      dataKey="count"
                      stroke={COLORS.emerald}
                      fill="url(#feedbackGradient)"
                      strokeWidth={2}
                      name="Total Feedback"
                      dot={{ fill: COLORS.emerald, strokeWidth: 0, r: 4 }}
                      activeDot={{ fill: COLORS.emerald, strokeWidth: 2, stroke: '#fff', r: 6 }}
                    />
                    <Area
                      type="monotone"
                      dataKey="positive"
                      stroke={COLORS.blue}
                      fill="url(#positiveGradient)"
                      strokeWidth={2}
                      name="Positive"
                      dot={{ fill: COLORS.blue, strokeWidth: 0, r: 3 }}
                      activeDot={{ fill: COLORS.blue, strokeWidth: 2, stroke: '#fff', r: 5 }}
                    />
                  </ComposedChart>
                </ResponsiveContainer>
              </div>
            </div>

            {/* Genre distribution pie */}
            <div className="stats-card p-4">
              <h3 className="text-sm font-medium text-zinc-400 mb-4 flex items-center gap-2">
                <svg className="w-4 h-4 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 3.055A9.001 9.001 0 1020.945 13H11V3.055z" />
                </svg>
                Top Genres Distribution
              </h3>
              <div className="h-64">
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie
                      data={genrePieData}
                      cx="50%"
                      cy="50%"
                      innerRadius={60}
                      outerRadius={80}
                      paddingAngle={3}
                      dataKey="value"
                      animationBegin={0}
                      animationDuration={800}
                    >
                      {genrePieData.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={entry.color} stroke="transparent" />
                      ))}
                    </Pie>
                    <Tooltip
                      contentStyle={{ background: '#18181b', border: '1px solid #27272a', borderRadius: 8 }}
                    />
                    <Legend
                      layout="vertical"
                      align="right"
                      verticalAlign="middle"
                      iconType="circle"
                      iconSize={8}
                      formatter={(value: string) => <span className="text-xs text-zinc-400">{value}</span>}
                    />
                  </PieChart>
                </ResponsiveContainer>
              </div>
            </div>
          </div>

          {/* Content timeline & Pattern performance */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Content by year */}
            <div className="stats-card p-4">
              <h3 className="text-sm font-medium text-zinc-400 mb-4 flex items-center gap-2">
                <svg className="w-4 h-4 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 4v16M17 4v16M3 8h4m10 0h4M3 12h18M3 16h4m10 0h4" />
                </svg>
                Content Distribution by Year
              </h3>
              <div className="h-64">
                <ResponsiveContainer width="100%" height="100%">
                  <AreaChart data={contentByYearData}>
                    <defs>
                      <linearGradient id="seriesGradient" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor={COLORS.emerald} stopOpacity={0.5} />
                        <stop offset="100%" stopColor={COLORS.emerald} stopOpacity={0} />
                      </linearGradient>
                      <linearGradient id="movieGradient" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor={COLORS.blue} stopOpacity={0.5} />
                        <stop offset="100%" stopColor={COLORS.blue} stopOpacity={0} />
                      </linearGradient>
                    </defs>
                    <CartesianGrid strokeDasharray="3 3" stroke="#27272a" />
                    <XAxis dataKey="year" tick={{ fill: '#71717a', fontSize: 10 }} />
                    <YAxis tick={{ fill: '#71717a', fontSize: 10 }} />
                    <Tooltip contentStyle={{ background: '#18181b', border: '1px solid #27272a', borderRadius: 8 }} />
                    <Legend />
                    <Area type="monotone" dataKey="series" stackId="1" stroke={COLORS.emerald} fill="url(#seriesGradient)" />
                    <Area type="monotone" dataKey="movie" stackId="1" stroke={COLORS.blue} fill="url(#movieGradient)" />
                  </AreaChart>
                </ResponsiveContainer>
              </div>
            </div>

            {/* Pattern performance with bars */}
            <div className="stats-card p-4">
              <h3 className="text-sm font-medium text-zinc-400 mb-4 flex items-center gap-2">
                <svg className="w-4 h-4 text-orange-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                </svg>
                Pattern Performance
              </h3>
              <div className="h-64">
                <ResponsiveContainer width="100%" height="100%">
                  <ComposedChart data={data.patterns.slice(0, 8)} layout="vertical">
                    <CartesianGrid strokeDasharray="3 3" stroke="#27272a" />
                    <XAxis type="number" tick={{ fill: '#71717a', fontSize: 10 }} domain={[0, 'auto']} />
                    <YAxis
                      dataKey="type"
                      type="category"
                      width={100}
                      tick={{ fill: '#71717a', fontSize: 10 }}
                      tickFormatter={(v) => v.replace(/_/g, ' ').slice(0, 12)}
                    />
                    <Tooltip contentStyle={{ background: '#18181b', border: '1px solid #27272a', borderRadius: 8 }} />
                    <Bar dataKey="uses" fill={COLORS.blue} barSize={12} radius={[0, 4, 4, 0]} name="Uses">
                      {data.patterns.slice(0, 8).map((_, index) => (
                        <Cell key={`cell-${index}`} fill={GRADIENT_COLORS[index % GRADIENT_COLORS.length]} />
                      ))}
                    </Bar>
                  </ComposedChart>
                </ResponsiveContainer>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Learning Tab */}
      {selectedTab === 'learning' && data && (
        <div className="space-y-6">
          {/* Learning state diagram + sparkline */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <div className="stats-card p-4">
              <h3 className="text-sm font-medium text-zinc-400 mb-4 flex items-center gap-2">
                <svg className="w-4 h-4 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
                </svg>
                Q-Learning State Machine
              </h3>
              <LearningStateDiagram explorationRate={0.3} learningRate={0.1} />
            </div>

            <div className="stats-card p-4">
              <h3 className="text-sm font-medium text-zinc-400 mb-4">Reward Distribution</h3>
              <RewardSparkline data={recentRewards} />
              <div className="mt-4 space-y-3">
                {/* Reward breakdown bars */}
                <div className="flex items-center gap-2">
                  <span className="text-xs text-zinc-500 w-16">Positive</span>
                  <div className="flex-1 h-4 bg-zinc-800 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-gradient-to-r from-emerald-600 to-emerald-400 rounded-full transition-all duration-500"
                      style={{ width: `${recentRewards.length > 0 ? (recentRewards.filter(r => r > 0).length / recentRewards.length) * 100 : 0}%` }}
                    />
                  </div>
                  <span className="text-sm font-bold text-emerald-400 w-10 text-right">{recentRewards.filter(r => r > 0).length}</span>
                </div>
                <div className="flex items-center gap-2">
                  <span className="text-xs text-zinc-500 w-16">Negative</span>
                  <div className="flex-1 h-4 bg-zinc-800 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-gradient-to-r from-red-600 to-red-400 rounded-full transition-all duration-500"
                      style={{ width: `${recentRewards.length > 0 ? (recentRewards.filter(r => r < 0).length / recentRewards.length) * 100 : 0}%` }}
                    />
                  </div>
                  <span className="text-sm font-bold text-red-400 w-10 text-right">{recentRewards.filter(r => r < 0).length}</span>
                </div>
                {/* Min/Max/Avg stats */}
                <div className="grid grid-cols-3 gap-2 text-center pt-2 border-t border-zinc-800">
                  <div>
                    <div className="text-sm font-bold text-emerald-400">
                      +{recentRewards.length > 0 ? Math.max(...recentRewards.filter(r => r > 0), 0).toFixed(2) : '0.00'}
                    </div>
                    <div className="text-xs text-zinc-500">Max +</div>
                  </div>
                  <div>
                    <div className="text-sm font-bold text-red-400">
                      {recentRewards.length > 0 ? Math.min(...recentRewards.filter(r => r < 0), 0).toFixed(2) : '0.00'}
                    </div>
                    <div className="text-xs text-zinc-500">Max -</div>
                  </div>
                  <div>
                    <div className={`text-sm font-bold ${recentRewards.length > 0 && recentRewards.reduce((a, b) => a + b, 0) / recentRewards.length >= 0 ? 'text-emerald-400' : 'text-red-400'}`}>
                      {recentRewards.length > 0 ? (recentRewards.reduce((a, b) => a + b, 0) / recentRewards.length).toFixed(3) : '0.000'}
                    </div>
                    <div className="text-xs text-zinc-500">Net Avg</div>
                  </div>
                </div>
              </div>
            </div>

            <div className="stats-card p-4">
              <h3 className="text-sm font-medium text-zinc-400 mb-4">Algorithm Status</h3>
              <div className="space-y-3">
                {['Q-Learning', 'Thompson', 'UCB1', 'LinUCB'].map((algo, i) => (
                  <div key={algo} className="flex items-center gap-3">
                    <PulseRing color={GRADIENT_COLORS[i]} size="sm" />
                    <span className="text-sm text-white flex-1">{algo}</span>
                    <span className="text-xs px-2 py-1 rounded-full bg-emerald-500/20 text-emerald-400">Active</span>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Algorithm scatter plot */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <div className="stats-card p-4">
              <h3 className="text-sm font-medium text-zinc-400 mb-4 flex items-center gap-2">
                <svg className="w-4 h-4 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 12l3-3 3 3 4-4M8 21l4-4 4 4M3 4h18M4 4h16v12a1 1 0 01-1 1H5a1 1 0 01-1-1V4z" />
                </svg>
                Algorithm Comparison (Success vs Usage)
              </h3>
              <div className="h-64">
                <ResponsiveContainer width="100%" height="100%">
                  <ScatterChart>
                    <CartesianGrid strokeDasharray="3 3" stroke="#27272a" />
                    <XAxis type="number" dataKey="uses" name="Uses" tick={{ fill: '#71717a', fontSize: 10 }} />
                    <YAxis type="number" dataKey="successRate" name="Success %" domain={[0, 100]} tick={{ fill: '#71717a', fontSize: 10 }} />
                    <ZAxis type="number" dataKey="reward" range={[50, 400]} />
                    <Tooltip
                      contentStyle={{ background: '#18181b', border: '1px solid #27272a', borderRadius: 8 }}
                      formatter={(value: any, name: string) => [name === 'successRate' ? `${value.toFixed(1)}%` : value, name]}
                    />
                    <Scatter data={algorithmScatterData} name="Algorithms">
                      {algorithmScatterData.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={entry.color} />
                      ))}
                    </Scatter>
                  </ScatterChart>
                </ResponsiveContainer>
              </div>
            </div>

            {/* Radar chart for pattern metrics */}
            <div className="stats-card p-4">
              <h3 className="text-sm font-medium text-zinc-400 mb-4 flex items-center gap-2">
                <svg className="w-4 h-4 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 3.055A9.001 9.001 0 1020.945 13H11V3.055z" />
                </svg>
                Pattern Metrics Radar
              </h3>
              <div className="h-64">
                <ResponsiveContainer width="100%" height="100%">
                  <RadarChart data={radarData}>
                    <PolarGrid stroke="#27272a" />
                    <PolarAngleAxis dataKey="pattern" tick={{ fill: '#71717a', fontSize: 9 }} />
                    <PolarRadiusAxis angle={30} domain={[0, 100]} tick={{ fill: '#71717a', fontSize: 8 }} />
                    <Radar name="Success %" dataKey="successRate" stroke={COLORS.emerald} fill={COLORS.emerald} fillOpacity={0.3} />
                    <Radar name="Usage" dataKey="uses" stroke={COLORS.blue} fill={COLORS.blue} fillOpacity={0.3} />
                    <Legend />
                    <Tooltip contentStyle={{ background: '#18181b', border: '1px solid #27272a', borderRadius: 8 }} />
                  </RadarChart>
                </ResponsiveContainer>
              </div>
            </div>
          </div>

          {/* Pattern treemap */}
          <div className="stats-card p-4">
            <h3 className="text-sm font-medium text-zinc-400 mb-4 flex items-center gap-2">
              <svg className="w-4 h-4 text-pink-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" />
              </svg>
              Pattern Usage Treemap (size = usage, hover for success rate)
            </h3>
            <div className="h-80">
              <ResponsiveContainer width="100%" height="100%">
                <Treemap data={treemapData} dataKey="size" aspectRatio={4 / 3} stroke="#18181b">
                  <Tooltip
                    content={({ payload }) => {
                      if (!payload || payload.length === 0) return null;
                      const data = payload[0]?.payload;
                      if (!data) return null;
                      return (
                        <div className="bg-zinc-900 border border-zinc-700 rounded-lg p-3 shadow-xl">
                          <div className="font-medium text-white mb-1">{data.name}</div>
                          <div className="text-sm text-zinc-400">
                            <span className="text-emerald-400 font-medium">{data.size}</span> uses
                          </div>
                          <div className="text-sm text-zinc-400">
                            <span className={`font-medium ${data.successRate >= 0.7 ? 'text-emerald-400' : data.successRate >= 0.5 ? 'text-yellow-400' : 'text-red-400'}`}>
                              {(data.successRate * 100).toFixed(1)}%
                            </span> success rate
                          </div>
                        </div>
                      );
                    }}
                  />
                  {treemapData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Treemap>
              </ResponsiveContainer>
            </div>
          </div>

          {/* Learning metrics table */}
          <div className="stats-card p-4">
            <h3 className="text-sm font-medium text-zinc-400 mb-4">Learning Metrics Detail</h3>
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="text-left text-zinc-500 border-b border-zinc-800">
                    <th className="pb-3 font-medium">Pattern</th>
                    <th className="pb-3 font-medium text-right">Success Rate</th>
                    <th className="pb-3 font-medium text-right">Uses</th>
                    <th className="pb-3 font-medium text-right">Avg Reward</th>
                    <th className="pb-3 font-medium text-right">Freshness</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-zinc-800">
                  {data.learningMetrics.map((m) => (
                    <tr key={m.pattern} className="hover:bg-zinc-800/50 transition-colors">
                      <td className="py-3 text-white">{m.pattern.replace(/_/g, ' ')}</td>
                      <td className="py-3 text-right">
                        <div className="flex items-center justify-end gap-2">
                          <div className="w-16 h-2 bg-zinc-800 rounded-full overflow-hidden">
                            <div
                              className={`h-full rounded-full ${m.successRate >= 0.7 ? 'bg-emerald-500' : m.successRate >= 0.5 ? 'bg-yellow-500' : 'bg-red-500'}`}
                              style={{ width: `${m.successRate * 100}%` }}
                            />
                          </div>
                          <span className={`font-medium ${m.successRate >= 0.7 ? 'text-emerald-400' : m.successRate >= 0.5 ? 'text-yellow-400' : 'text-red-400'}`}>
                            {(m.successRate * 100).toFixed(1)}%
                          </span>
                        </div>
                      </td>
                      <td className="py-3 text-right text-zinc-400">{m.uses}</td>
                      <td className="py-3 text-right">
                        <span className={m.avgReward >= 0 ? 'text-emerald-400' : 'text-red-400'}>
                          {m.avgReward >= 0 ? '+' : ''}{m.avgReward.toFixed(3)}
                        </span>
                      </td>
                      <td className="py-3 text-right text-zinc-500">{m.freshness.toFixed(1)}h ago</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}

      {/* Network Graph Tab */}
      {selectedTab === 'network' && data && (
        <div className="space-y-6">
          {/* Network stats - moved to top */}
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
            <div className="stats-card p-4 text-center group hover:scale-[1.02] transition-transform">
              <div className="w-10 h-10 rounded-xl bg-purple-500/20 flex items-center justify-center mx-auto mb-2 group-hover:scale-110 transition-transform">
                <svg className="w-5 h-5 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
                </svg>
              </div>
              <div className="text-2xl font-bold text-white">{graphData.nodes.length}</div>
              <div className="text-xs text-zinc-500">Genres</div>
            </div>
            <div className="stats-card p-4 text-center group hover:scale-[1.02] transition-transform">
              <div className="w-10 h-10 rounded-xl bg-blue-500/20 flex items-center justify-center mx-auto mb-2 group-hover:scale-110 transition-transform">
                <svg className="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                </svg>
              </div>
              <div className="text-2xl font-bold text-white">{graphData.links.length}</div>
              <div className="text-xs text-zinc-500">Connections</div>
            </div>
            <div className="stats-card p-4 text-center group hover:scale-[1.02] transition-transform">
              <div className="w-10 h-10 rounded-xl bg-emerald-500/20 flex items-center justify-center mx-auto mb-2 group-hover:scale-110 transition-transform">
                <svg className="w-5 h-5 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
                </svg>
              </div>
              <div className="text-2xl font-bold text-emerald-400">
                {(graphData.links.reduce((sum, l) => sum + (l.value || 0), 0) / Math.max(graphData.links.length, 1)).toFixed(1)}
              </div>
              <div className="text-xs text-zinc-500">Avg Strength</div>
            </div>
            <div className="stats-card p-4 text-center group hover:scale-[1.02] transition-transform">
              <div className="w-10 h-10 rounded-xl bg-orange-500/20 flex items-center justify-center mx-auto mb-2 group-hover:scale-110 transition-transform">
                <svg className="w-5 h-5 text-orange-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" />
                </svg>
              </div>
              <div className="text-2xl font-bold text-orange-400">
                {graphData.nodes.length > 0 ? ((graphData.links.length * 2) / (graphData.nodes.length * (graphData.nodes.length - 1)) * 100).toFixed(1) : 0}%
              </div>
              <div className="text-xs text-zinc-500">Density</div>
            </div>
          </div>

          {/* Main network graph */}
          <div className="stats-card p-4">
            <div className="flex items-center justify-between mb-4">
              <div>
                <h3 className="text-sm font-medium text-zinc-400 flex items-center gap-2">
                  <svg className="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14 10l-2 1m0 0l-2-1m2 1v2.5M20 7l-2 1m2-1l-2-1m2 1v2.5M14 4l-2-1-2 1M4 7l2-1M4 7l2 1M4 7v2.5M12 21l-2-1m2 1l2-1m-2 1v-2.5M6 18l-2-1v-2.5M18 18l2-1v-2.5" />
                  </svg>
                  Genre Relationship Network
                  <PulseRing color={COLORS.emerald} size="sm" />
                </h3>
                <p className="text-xs text-zinc-500 mt-1">Click nodes to focus, drag to explore, scroll to zoom</p>
              </div>
              {/* Legend */}
              <div className="flex items-center gap-4 text-xs">
                <div className="flex items-center gap-2">
                  <div className="w-4 h-4 rounded-full bg-emerald-500"></div>
                  <span className="text-zinc-400">Strong (50+)</span>
                </div>
                <div className="flex items-center gap-2">
                  <div className="w-3 h-3 rounded-full bg-blue-500"></div>
                  <span className="text-zinc-400">Medium</span>
                </div>
                <div className="flex items-center gap-2">
                  <div className="w-2 h-2 rounded-full bg-zinc-500"></div>
                  <span className="text-zinc-400">Weak</span>
                </div>
              </div>
            </div>
            <div className="h-[550px] glass-card rounded-xl overflow-hidden relative">
              {typeof window !== 'undefined' && graphData.nodes.length > 0 && (
                <ForceGraph2D
                  ref={graphRef}
                  graphData={graphData}
                  nodeLabel={(node: any) => `${node.name}: ${node.val} content items`}
                  nodeColor={(node: any) => node.color || COLORS.emerald}
                  nodeRelSize={8}
                  nodeVal={(node: any) => Math.sqrt(node.val || 1) * 1.5}
                  linkWidth={(link: any) => Math.max(1, Math.sqrt(link.value || 1) * 0.8)}
                  linkColor={(link: any) => {
                    const strength = link.value || 0;
                    if (strength >= 50) return 'rgba(16, 185, 129, 0.6)'; // emerald
                    if (strength >= 20) return 'rgba(59, 130, 246, 0.5)'; // blue
                    if (strength >= 10) return 'rgba(139, 92, 246, 0.4)'; // purple
                    return 'rgba(113, 113, 122, 0.3)'; // zinc
                  }}
                  linkDirectionalParticles={(link: any) => link.value >= 30 ? 2 : 0}
                  linkDirectionalParticleWidth={2}
                  linkDirectionalParticleSpeed={0.005}
                  linkDirectionalParticleColor={(link: any) => {
                    const strength = link.value || 0;
                    if (strength >= 50) return COLORS.emerald;
                    return COLORS.blue;
                  }}
                  backgroundColor="transparent"
                  width={typeof window !== 'undefined' ? Math.min(window.innerWidth - 100, 1200) : 800}
                  height={550}
                  cooldownTicks={100}
                  d3AlphaDecay={0.02}
                  d3VelocityDecay={0.3}
                  onNodeClick={(node: any) => {
                    if (graphRef.current) {
                      graphRef.current.centerAt(node.x, node.y, 1000);
                      graphRef.current.zoom(2.5, 1000);
                    }
                  }}
                  onNodeHover={(node: any) => {
                    document.body.style.cursor = node ? 'pointer' : 'default';
                  }}
                  nodeCanvasObject={(node: any, ctx, globalScale) => {
                    const nodeSize = Math.sqrt(node.val || 1) * 1.5;
                    const fontSize = Math.max(10, 14 / globalScale);

                    // Draw node glow effect
                    ctx.beginPath();
                    ctx.arc(node.x, node.y, nodeSize + 4, 0, 2 * Math.PI);
                    ctx.fillStyle = `${node.color}30`;
                    ctx.fill();

                    // Draw node
                    ctx.beginPath();
                    ctx.arc(node.x, node.y, nodeSize, 0, 2 * Math.PI);
                    ctx.fillStyle = node.color;
                    ctx.fill();
                    ctx.strokeStyle = 'rgba(255,255,255,0.3)';
                    ctx.lineWidth = 1;
                    ctx.stroke();

                    // Draw label
                    ctx.font = `bold ${fontSize}px -apple-system, BlinkMacSystemFont, sans-serif`;
                    ctx.textAlign = 'center';
                    ctx.textBaseline = 'middle';
                    ctx.fillStyle = 'white';
                    ctx.shadowColor = 'rgba(0,0,0,0.8)';
                    ctx.shadowBlur = 4;
                    ctx.fillText(node.name, node.x, node.y + nodeSize + fontSize);
                    ctx.shadowBlur = 0;
                  }}
                  nodeCanvasObjectMode={() => 'replace'}
                />
              )}
              {/* Zoom controls */}
              <div className="absolute bottom-4 right-4 flex flex-col gap-2">
                <button
                  onClick={() => graphRef.current?.zoom(graphRef.current.zoom() * 1.5, 300)}
                  className="w-8 h-8 rounded-lg bg-zinc-800/80 hover:bg-zinc-700 flex items-center justify-center text-white transition-colors"
                >
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                  </svg>
                </button>
                <button
                  onClick={() => graphRef.current?.zoom(graphRef.current.zoom() / 1.5, 300)}
                  className="w-8 h-8 rounded-lg bg-zinc-800/80 hover:bg-zinc-700 flex items-center justify-center text-white transition-colors"
                >
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 12H4" />
                  </svg>
                </button>
                <button
                  onClick={() => {
                    graphRef.current?.centerAt(0, 0, 500);
                    graphRef.current?.zoom(1, 500);
                  }}
                  className="w-8 h-8 rounded-lg bg-zinc-800/80 hover:bg-zinc-700 flex items-center justify-center text-white transition-colors"
                  title="Reset view"
                >
                  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  </svg>
                </button>
              </div>
            </div>
          </div>

          {/* Top connections list */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <div className="stats-card p-4">
              <h3 className="text-sm font-medium text-zinc-400 mb-4 flex items-center gap-2">
                <svg className="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
                </svg>
                Strongest Connections
              </h3>
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {[...graphData.links]
                  .sort((a, b) => (b.value || 0) - (a.value || 0))
                  .slice(0, 10)
                  .map((link, idx) => {
                    const sourceNode = graphData.nodes.find(n => n.id === link.source);
                    const targetNode = graphData.nodes.find(n => n.id === link.target);
                    return (
                      <div key={idx} className="flex items-center gap-3 p-2 rounded-lg bg-zinc-800/50 hover:bg-zinc-800 transition-colors">
                        <span className="text-xs text-zinc-500 w-5">{idx + 1}.</span>
                        <span className="text-white text-sm">{sourceNode?.name || link.source}</span>
                        <svg className="w-4 h-4 text-emerald-400 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
                        </svg>
                        <span className="text-white text-sm">{targetNode?.name || link.target}</span>
                        <span className="ml-auto text-xs font-bold text-emerald-400">{link.value}</span>
                      </div>
                    );
                  })}
              </div>
            </div>

            <div className="stats-card p-4">
              <h3 className="text-sm font-medium text-zinc-400 mb-4 flex items-center gap-2">
                <svg className="w-4 h-4 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
                </svg>
                Top Genres by Content
              </h3>
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {[...graphData.nodes]
                  .sort((a, b) => (b.val || 0) - (a.val || 0))
                  .slice(0, 10)
                  .map((node, idx) => (
                    <div key={node.id} className="flex items-center gap-3 p-2 rounded-lg bg-zinc-800/50 hover:bg-zinc-800 transition-colors">
                      <span className="text-xs text-zinc-500 w-5">{idx + 1}.</span>
                      <div
                        className="w-3 h-3 rounded-full flex-shrink-0"
                        style={{ backgroundColor: node.color || GRADIENT_COLORS[idx % GRADIENT_COLORS.length] }}
                      />
                      <span className="text-white text-sm flex-1">{node.name}</span>
                      <div className="flex items-center gap-2">
                        <div className="w-20 h-2 bg-zinc-700 rounded-full overflow-hidden">
                          <div
                            className="h-full rounded-full"
                            style={{
                              width: `${(node.val / Math.max(...graphData.nodes.map(n => n.val || 0))) * 100}%`,
                              backgroundColor: node.color || GRADIENT_COLORS[idx % GRADIENT_COLORS.length]
                            }}
                          />
                        </div>
                        <span className="text-xs font-bold text-zinc-300 w-10 text-right">{node.val}</span>
                      </div>
                    </div>
                  ))}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Activity Tab */}
      {selectedTab === 'activity' && data && (
        <div className="space-y-6">
          {/* Activity stream */}
          <div className="stats-card p-4">
            <h3 className="text-sm font-medium text-zinc-400 mb-4 flex items-center gap-2">
              <svg className="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              Recent Activity Stream
              <PulseRing color={COLORS.emerald} size="sm" />
              <span className="ml-auto text-xs text-zinc-500">{data.recentActivity.length} events</span>
            </h3>
            <div className="space-y-3 max-h-[600px] overflow-y-auto">
              {data.recentActivity.map((activity, idx) => (
                <div
                  key={activity.id}
                  className={`flex items-start gap-3 p-3 rounded-lg transition-all animate-fade-in ${
                    activity.success ? 'bg-emerald-500/10 hover:bg-emerald-500/15' : 'bg-red-500/10 hover:bg-red-500/15'
                  }`}
                  style={{ animationDelay: `${idx * 50}ms` }}
                >
                  <div className={`w-8 h-8 rounded-lg flex items-center justify-center flex-shrink-0 ${
                    activity.success ? 'bg-emerald-500/20' : 'bg-red-500/20'
                  }`}>
                    {activity.success ? (
                      <svg className="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                      </svg>
                    ) : (
                      <svg className="w-4 h-4 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z" />
                      </svg>
                    )}
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap">
                      <span className="text-white font-medium truncate">{activity.title || 'Unknown Content'}</span>
                      <span className={`text-xs px-2 py-0.5 rounded-full ${
                        activity.contentType === 'series' ? 'bg-emerald-500/20 text-emerald-400' : 'bg-blue-500/20 text-blue-400'
                      }`}>
                        {activity.contentType}
                      </span>
                    </div>
                    <div className="flex items-center gap-3 mt-1 text-xs text-zinc-500">
                      <span>{activity.action}</span>
                      <span>•</span>
                      <span className={activity.reward >= 0 ? 'text-emerald-400' : 'text-red-400'}>
                        {activity.reward >= 0 ? '+' : ''}{activity.reward.toFixed(3)} reward
                      </span>
                      {activity.pattern && (
                        <>
                          <span>•</span>
                          <span className="text-purple-400">{activity.pattern.replace(/_/g, ' ')}</span>
                        </>
                      )}
                    </div>
                  </div>
                  <div className="text-xs text-zinc-500 flex-shrink-0">
                    {new Date(activity.time).toLocaleTimeString()}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Activity summary */}
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <div className="stats-card p-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-zinc-400 text-sm">Watch Rate</span>
                <span className="text-emerald-400 font-medium">
                  {((data.recentActivity.filter(a => a.success).length / Math.max(data.recentActivity.length, 1)) * 100).toFixed(1)}%
                </span>
              </div>
              <div className="h-2 bg-zinc-800 rounded-full overflow-hidden">
                <div
                  className="h-full bg-gradient-to-r from-emerald-500 to-emerald-400 rounded-full transition-all duration-500"
                  style={{ width: `${(data.recentActivity.filter(a => a.success).length / Math.max(data.recentActivity.length, 1)) * 100}%` }}
                />
              </div>
            </div>
            <div className="stats-card p-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-zinc-400 text-sm">Avg Reward</span>
                <span className={`font-medium ${
                  data.recentActivity.reduce((sum, a) => sum + a.reward, 0) / Math.max(data.recentActivity.length, 1) >= 0
                    ? 'text-emerald-400' : 'text-red-400'
                }`}>
                  {(data.recentActivity.reduce((sum, a) => sum + a.reward, 0) / Math.max(data.recentActivity.length, 1)).toFixed(3)}
                </span>
              </div>
              <div className="text-xs text-zinc-500">Based on {data.recentActivity.length} recent interactions</div>
            </div>
            <div className="stats-card p-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-zinc-400 text-sm">Top Content Type</span>
                <span className="text-blue-400 font-medium">
                  {data.recentActivity.filter(a => a.contentType === 'series').length >=
                   data.recentActivity.filter(a => a.contentType === 'movie').length
                    ? 'Series' : 'Movies'}
                </span>
              </div>
              <div className="flex gap-2 text-xs">
                <span className="text-emerald-400">{data.recentActivity.filter(a => a.contentType === 'series').length} series</span>
                <span className="text-zinc-500">|</span>
                <span className="text-blue-400">{data.recentActivity.filter(a => a.contentType === 'movie').length} movies</span>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Footer */}
      <footer className="mt-12 pt-6 border-t border-zinc-800">
        <div className="flex flex-col sm:flex-row items-center justify-between gap-4 text-xs text-zinc-500">
          <div className="flex items-center gap-2">
            <PulseRing color={COLORS.emerald} size="sm" />
            <span>Last updated: {data ? new Date().toLocaleTimeString() : 'Loading...'}</span>
          </div>
          <div className="flex items-center gap-4">
            <span>RuVector + Q-Learning + Thompson Sampling + UCB1 + LinUCB</span>
            <Link href="/about" className="text-emerald-400 hover:underline">Learn More</Link>
          </div>
        </div>
      </footer>
      </div>
    </main>
  );
}
