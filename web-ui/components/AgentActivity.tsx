'use client';

import { useEffect, useState } from 'react';

interface Agent {
  id: string;
  name: string;
  status: 'pending' | 'active' | 'complete';
  description: string;
  emoji: string;
}

export default function AgentActivity() {
  const [agents, setAgents] = useState<Agent[]>([
    { id: '1', name: 'PersonalizationAgent', status: 'pending', description: 'Analyzing your viewing history...', emoji: '👤' },
    { id: '2', name: 'MoodDetectionAgent', status: 'pending', description: 'Understanding your current mood...', emoji: '🎭' },
    { id: '3', name: 'ResearchAgent', status: 'pending', description: 'Searching across 5 platforms...', emoji: '🔍' },
    { id: '4', name: 'ReviewAggregationAgent', status: 'pending', description: 'Collecting reviews from 4 sources...', emoji: '⭐' },
    { id: '5', name: 'TrendAnalysisAgent', status: 'pending', description: 'Analyzing social trends...', emoji: '📈' },
    { id: '6', name: 'ContentFilterAgent', status: 'pending', description: 'Applying safety filters...', emoji: '🛡️' },
    { id: '7', name: 'AnalysisAgent', status: 'pending', description: 'Ranking recommendations...', emoji: '🧠' },
    { id: '8', name: 'RecommendationAgent', status: 'pending', description: 'Generating final picks...', emoji: '🎯' },
  ]);

  useEffect(() => {
    // Simulate agent progression
    const timeline = [
      { time: 500, agentIds: ['1', '2'] },      // Phase 1: Parallel personalization + mood
      { time: 1500, agentIds: ['3'] },          // Phase 2: Research
      { time: 3000, agentIds: ['4', '5'] },     // Phase 3: Parallel reviews + trends
      { time: 4500, agentIds: ['6'] },          // Phase 4: Safety filter
      { time: 5000, agentIds: ['7'] },          // Phase 5: Analysis
      { time: 5500, agentIds: ['8'] },          // Phase 6: Final recommendations
    ];

    timeline.forEach(({ time, agentIds }) => {
      setTimeout(() => {
        setAgents((prev) =>
          prev.map((agent) =>
            agentIds.includes(agent.id)
              ? { ...agent, status: 'active' }
              : agent
          )
        );

        // Mark as complete after 800ms of being active
        setTimeout(() => {
          setAgents((prev) =>
            prev.map((agent) =>
              agentIds.includes(agent.id)
                ? { ...agent, status: 'complete' }
                : agent
            )
          );
        }, 800);
      }, time);
    });
  }, []);

  const getStatusColor = (status: Agent['status']) => {
    switch (status) {
      case 'pending':
        return 'bg-bg-dark-border';
      case 'active':
        return 'bg-youtube-red animate-pulse';
      case 'complete':
        return 'bg-semantic-success';
    }
  };

  const getStatusIcon = (status: Agent['status']) => {
    switch (status) {
      case 'pending':
        return '⏳';
      case 'active':
        return '⚡';
      case 'complete':
        return '✅';
    }
  };

  return (
    <div className="bg-bg-dark-card rounded-card p-6 shadow-card">
      <div className="mb-6">
        <h3 className="text-2xl font-bold text-text-dark-primary mb-2">
          Multi-Agent System at Work
        </h3>
        <p className="text-text-dark-secondary">
          Watch our 8 specialized agents collaborate in real-time to find your perfect recommendations
        </p>
      </div>

      <div className="space-y-3">
        {agents.map((agent) => (
          <div
            key={agent.id}
            className="bg-bg-dark rounded-lg p-4 border border-bg-dark-border transition-all"
          >
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center space-x-3">
                <span className="text-2xl">{agent.emoji}</span>
                <div>
                  <h4 className="font-semibold text-text-dark-primary">
                    {agent.name}
                  </h4>
                  <p className="text-sm text-text-dark-secondary">
                    {agent.description}
                  </p>
                </div>
              </div>
              <div className="flex items-center space-x-2">
                <span className="text-lg">{getStatusIcon(agent.status)}</span>
                <div className={`w-3 h-3 rounded-full ${getStatusColor(agent.status)}`} />
              </div>
            </div>

            {/* Progress Bar */}
            {agent.status === 'active' && (
              <div className="mt-2">
                <div className="h-1 bg-bg-dark-border rounded-full overflow-hidden">
                  <div className="h-full bg-youtube-red animate-[progress_800ms_ease-in-out]"
                       style={{ animation: 'progress 800ms ease-in-out forwards' }} />
                </div>
              </div>
            )}
          </div>
        ))}
      </div>

      <style jsx>{`
        @keyframes progress {
          from { width: 0%; }
          to { width: 100%; }
        }
      `}</style>
    </div>
  );
}
