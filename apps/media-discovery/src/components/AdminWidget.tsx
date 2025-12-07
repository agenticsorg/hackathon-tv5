'use client';

import { useState, useMemo } from 'react';
import { useDiscoveryStore } from '@/lib/discovery-store';

export function AdminWidget() {
  const [isCollapsed, setIsCollapsed] = useState(false);

  const {
    genres,
    clickSequence,
    mouseVelocity,
    skipEvents,
    totalHovers,
    totalTime,
  } = useDiscoveryStore();

  const sortedGenres = useMemo(() => {
    return Object.values(genres)
      .map(g => ({
        ...g,
        score: g.hoverTime + (g.clickOrder ? (9 - g.clickOrder) * 800 : 0),
      }))
      .sort((a, b) => b.score - a.score);
  }, [genres]);

  const avgVelocity = useMemo(() => {
    if (mouseVelocity.length === 0) return 0;
    return mouseVelocity.reduce((a, b) => a + b, 0) / mouseVelocity.length;
  }, [mouseVelocity]);

  const clickSequenceNames = useMemo(() => {
    return clickSequence.map(id => genres[id]?.name || id).join(' → ');
  }, [clickSequence, genres]);

  return (
    <div className="fixed bottom-4 right-4 w-[340px] bg-bg-primary/95 border border-border-subtle z-50 backdrop-blur-xl rounded-lg overflow-hidden shadow-2xl">
      {/* Header */}
      <div
        className="flex justify-between items-center px-4 py-3 bg-bg-elevated cursor-pointer"
        onClick={() => setIsCollapsed(!isCollapsed)}
      >
        <span className="text-text-secondary text-[0.625rem] uppercase tracking-widest font-medium">
          Genre Leaderboard
        </span>
        <button className="text-text-secondary hover:text-text-primary transition-colors text-lg leading-none">
          {isCollapsed ? '+' : '−'}
        </button>
      </div>

      {/* Body */}
      {!isCollapsed && (
        <div className="p-4 max-h-[450px] overflow-y-auto">
          {/* Total Time */}
          <div className="text-2xl text-accent-cyan text-center py-2 bg-bg-elevated rounded mb-4 mono font-medium">
            {totalTime.toFixed(3)}s
          </div>

          {/* Genre Leaderboard */}
          <div className="mb-4">
            <div className="text-text-secondary text-[0.6rem] uppercase tracking-wide mb-2 font-medium">
              Live Genre Rankings
            </div>
            <div className="space-y-1">
              {sortedGenres.map((genre, index) => (
                <div
                  key={genre.id}
                  className="flex items-center gap-2 py-1.5 border-b border-bg-elevated last:border-0"
                >
                  <span className="w-5 text-text-secondary font-bold text-xs">
                    {index + 1}
                  </span>
                  <span
                    className="w-2 h-2 rounded-sm"
                    style={{ backgroundColor: genre.color }}
                  />
                  <span className="flex-1 text-sm">{genre.name}</span>
                  <span className="text-sm font-medium mono">
                    {genre.hoverTime.toFixed(0)}ms
                  </span>
                </div>
              ))}
            </div>
          </div>

          {/* Click Sequence */}
          <div className="mb-4">
            <div className="text-text-secondary text-[0.6rem] uppercase tracking-wide mb-2 font-medium">
              Click Sequence
            </div>
            <div className="text-sm text-text-primary">
              {clickSequenceNames || 'No clicks yet'}
            </div>
          </div>

          {/* Behavior Signals */}
          <div>
            <div className="text-text-secondary text-[0.6rem] uppercase tracking-wide mb-2 font-medium">
              Behavior Signals
            </div>
            <div className="space-y-1">
              <div className="flex justify-between text-sm">
                <span className="text-text-secondary">Avg Velocity</span>
                <span className="mono">{avgVelocity.toFixed(0)} px/s</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-text-secondary">Skip Events</span>
                <span className="mono">{skipEvents}</span>
              </div>
              <div className="flex justify-between text-sm">
                <span className="text-text-secondary">Hover Events</span>
                <span className="mono">{totalHovers}</span>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
