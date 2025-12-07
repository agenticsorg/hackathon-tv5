'use client';

import { Card, CardBody, Progress } from '@heroui/react';
import type { LearningStats } from '@/lib/db';

interface StatsPanelProps {
  stats: LearningStats | null;
  isLoading?: boolean;
}

export function StatsPanel({ stats, isLoading }: StatsPanelProps) {
  if (isLoading) {
    return (
      <div className="stats-card p-6 animate-pulse">
        <div className="h-6 bg-gray-700 rounded w-1/2 mb-4" />
        <div className="space-y-3">
          <div className="h-4 bg-gray-700 rounded w-full" />
          <div className="h-4 bg-gray-700 rounded w-3/4" />
          <div className="h-4 bg-gray-700 rounded w-1/2" />
        </div>
      </div>
    );
  }

  if (!stats) return null;

  return (
    <div className="stats-card p-6">
      <h3 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
        <svg className="w-5 h-5 text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
        </svg>
        Self-Learning Stats
      </h3>

      <div className="grid grid-cols-2 gap-4 mb-6">
        <div className="text-center">
          <p className="text-3xl font-bold text-primary-400">{stats.totalContent.toLocaleString()}</p>
          <p className="text-xs text-gray-400">Total Content</p>
        </div>
        <div className="text-center">
          <p className="text-3xl font-bold text-green-400">{(stats.avgSuccessRate * 100).toFixed(1)}%</p>
          <p className="text-xs text-gray-400">Learning Rate</p>
        </div>
        <div className="text-center">
          <p className="text-2xl font-bold text-blue-400">{stats.totalSeries}</p>
          <p className="text-xs text-gray-400">Series</p>
        </div>
        <div className="text-center">
          <p className="text-2xl font-bold text-purple-400">{stats.totalMovies}</p>
          <p className="text-xs text-gray-400">Movies</p>
        </div>
      </div>

      <div className="mb-6">
        <h4 className="text-sm font-medium text-gray-300 mb-3">Top Patterns</h4>
        <div className="space-y-2">
          {stats.topPatterns.slice(0, 5).map((pattern) => (
            <div key={pattern.type} className="flex items-center gap-2">
              <span className="text-xs text-gray-400 w-28 truncate">{pattern.type}</span>
              <Progress
                value={pattern.rate * 100}
                color="primary"
                size="sm"
                className="flex-1"
              />
              <span className="text-xs text-primary-400 w-12 text-right">
                {(pattern.rate * 100).toFixed(0)}%
              </span>
            </div>
          ))}
        </div>
      </div>

      <div>
        <h4 className="text-sm font-medium text-gray-300 mb-3">Languages</h4>
        <div className="flex flex-wrap gap-2">
          {stats.languageDistribution.slice(0, 6).map((lang) => (
            <div
              key={lang.language}
              className="px-3 py-1 bg-primary-500/20 rounded-full text-xs"
            >
              <span className="text-white">{lang.language.toUpperCase()}</span>
              <span className="text-gray-400 ml-1">({lang.count})</span>
            </div>
          ))}
        </div>
      </div>

      <div className="mt-4 pt-4 border-t border-gray-700">
        <p className="text-xs text-gray-500 text-center">
          {stats.totalFeedback} learning interactions • {stats.totalPatterns} active patterns
        </p>
      </div>
    </div>
  );
}
