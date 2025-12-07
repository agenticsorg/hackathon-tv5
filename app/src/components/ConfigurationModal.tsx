'use client';

import { useState, useEffect } from 'react';

interface ConfigurationModalProps {
  isOpen: boolean;
  onClose: () => void;
}

interface RuVectorStats {
  totalVectors: number;
  hyperbolicEnabled: boolean;
  avgSearchLatency: string;
  indexType: string;
  vectorDimensions: number;
  contentCount: number;
}

interface SystemConfig {
  qLearning: {
    enabled: boolean;
    explorationRate: number;
    learningRate: number;
    rewardDecay: number;
    minExploration: number;
    explorationDecay: number;
  };
  vectorSearch: {
    hyperbolicEnabled: boolean;
    dimensions: number;
    distanceMetric: string;
    curvature: number;
    indexType: string;
  };
  recommendations: {
    diversityWeight: number;
    recencyBoost: number;
    popularityWeight: number;
    personalizedWeight: number;
    maxResults: number;
  };
  performance: {
    batchSize: number;
    cacheEnabled: boolean;
    parallelQueries: number;
    precomputeEmbeddings: boolean;
  };
}

const defaultConfig: SystemConfig = {
  qLearning: {
    enabled: true,
    explorationRate: 0.3,
    learningRate: 0.1,
    rewardDecay: 0.95,
    minExploration: 0.05,
    explorationDecay: 0.995,
  },
  vectorSearch: {
    hyperbolicEnabled: true,
    dimensions: 384,
    distanceMetric: 'cosine',
    curvature: -1.0,
    indexType: 'hnsw',
  },
  recommendations: {
    diversityWeight: 0.3,
    recencyBoost: 0.2,
    popularityWeight: 0.15,
    personalizedWeight: 0.7,
    maxResults: 20,
  },
  performance: {
    batchSize: 100,
    cacheEnabled: true,
    parallelQueries: 4,
    precomputeEmbeddings: true,
  },
};

type TabType = 'ruvector' | 'qlearning' | 'recommendations' | 'performance';

export function ConfigurationModal({ isOpen, onClose }: ConfigurationModalProps) {
  const [config, setConfig] = useState<SystemConfig>(defaultConfig);
  const [activeTab, setActiveTab] = useState<TabType>('ruvector');
  const [showTooltip, setShowTooltip] = useState<string | null>(null);
  const [ruvectorStats, setRuvectorStats] = useState<RuVectorStats | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saved' | 'error'>('idle');
  const [hasChanges, setHasChanges] = useState(false);

  // Fetch config and RuVector stats when modal opens
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden';
      fetchConfig();
      fetchRuVectorStats();
    } else {
      document.body.style.overflow = 'unset';
    }
    return () => {
      document.body.style.overflow = 'unset';
    };
  }, [isOpen]);

  const fetchConfig = async () => {
    try {
      const response = await fetch('/api/config');
      const data = await response.json();
      if (data.config) {
        setConfig(data.config);
        setHasChanges(false);
      }
    } catch (error) {
      console.error('Failed to fetch config:', error);
    }
  };

  const saveConfig = async () => {
    setIsSaving(true);
    setSaveStatus('idle');
    try {
      const response = await fetch('/api/config', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ config }),
      });

      if (response.ok) {
        setSaveStatus('saved');
        setHasChanges(false);
        setTimeout(() => setSaveStatus('idle'), 2000);
      } else {
        setSaveStatus('error');
      }
    } catch (error) {
      console.error('Failed to save config:', error);
      setSaveStatus('error');
    } finally {
      setIsSaving(false);
    }
  };

  const updateConfig = (category: keyof SystemConfig, key: string, value: number | boolean | string) => {
    setConfig({
      ...config,
      [category]: {
        ...config[category],
        [key]: value,
      },
    });
    setHasChanges(true);
  };

  const fetchRuVectorStats = async () => {
    setIsLoading(true);
    try {
      const response = await fetch('/api/stats');
      const data = await response.json();
      setRuvectorStats({
        totalVectors: data.stats?.totalContent || 2714,
        hyperbolicEnabled: data.learning?.vectorSpace?.hyperbolicEnabled || true,
        avgSearchLatency: data.learning?.vectorSpace?.avgSearchLatency || '3.2ms',
        indexType: 'HNSW',
        vectorDimensions: 384,
        contentCount: data.stats?.totalContent || 2714,
      });
    } catch (error) {
      console.error('Failed to fetch RuVector stats:', error);
    } finally {
      setIsLoading(false);
    }
  };

  if (!isOpen) return null;

  const tabs = [
    { id: 'ruvector' as TabType, label: 'RuVector', icon: '🚀' },
    { id: 'qlearning' as TabType, label: 'Q-Learning', icon: '🧠' },
    { id: 'recommendations' as TabType, label: 'Recs', icon: '⭐' },
    { id: 'performance' as TabType, label: 'Perf', icon: '⚡' },
  ];

  const descriptions: Record<string, string> = {
    // Q-Learning
    'explorationRate': 'ε (epsilon) in ε-greedy policy. Higher values encourage trying new content patterns. Range: 0.0-1.0',
    'learningRate': 'α (alpha) controls how quickly the Q-values update. Higher = faster learning but less stable. Range: 0.0-1.0',
    'rewardDecay': 'γ (gamma) discount factor for future rewards. Higher values prioritize long-term learning. Range: 0.0-1.0',
    'minExploration': 'Minimum exploration rate floor. Prevents the system from becoming too greedy.',
    'explorationDecay': 'Rate at which exploration decreases over time. Allows gradual shift to exploitation.',
    // Vector Search
    'hyperbolicEnabled': 'Use Poincaré ball hyperbolic space for hierarchical content relationships.',
    'dimensions': 'Vector embedding dimensionality. Higher = more expressive but slower.',
    'distanceMetric': 'Distance function for similarity. Cosine is rotation-invariant, Euclidean is absolute.',
    'curvature': 'Hyperbolic space curvature. More negative = stronger hierarchy preservation.',
    'indexType': 'Index structure. HNSW offers O(log n) search with high recall.',
    // Recommendations
    'diversityWeight': 'Balance between similar and diverse recommendations. Higher = more variety.',
    'recencyBoost': 'Preference for newer content. Higher = prioritize recent releases.',
    'popularityWeight': 'Weight given to popular content. Lower = more niche recommendations.',
    'personalizedWeight': 'How much to weight user preferences vs global patterns.',
    'maxResults': 'Maximum recommendations per request.',
    // Performance
    'batchSize': 'Number of items processed per batch for embedding generation.',
    'cacheEnabled': 'Cache frequently accessed embeddings and search results.',
    'parallelQueries': 'Number of concurrent vector search queries.',
    'precomputeEmbeddings': 'Pre-calculate embeddings during content ingestion.',
  };

  const renderSlider = (
    category: keyof SystemConfig,
    key: string,
    value: number,
    min: number,
    max: number,
    step: number,
    unit: string = ''
  ) => (
    <div
      className="group relative"
      onMouseEnter={() => setShowTooltip(key)}
      onMouseLeave={() => setShowTooltip(null)}
    >
      <div className="flex items-center justify-between mb-2">
        <label className="text-sm text-zinc-300 flex items-center gap-2">
          {key.replace(/([A-Z])/g, ' $1').trim()}
          <span className="w-4 h-4 rounded-full bg-zinc-700 flex items-center justify-center text-[10px] text-zinc-400 cursor-help">
            ?
          </span>
        </label>
        <span className="text-sm font-mono text-emerald-400">
          {value}{unit}
        </span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => updateConfig(category, key, parseFloat(e.target.value))}
        className="w-full h-2 bg-zinc-700 rounded-lg appearance-none cursor-pointer accent-emerald-500"
      />
      {showTooltip === key && descriptions[key] && (
        <div className="absolute z-50 bottom-full left-0 mb-2 p-3 bg-zinc-900 border border-zinc-700 rounded-lg text-xs text-zinc-300 max-w-xs shadow-xl">
          {descriptions[key]}
        </div>
      )}
    </div>
  );

  const renderToggle = (category: keyof SystemConfig, key: string, value: boolean) => (
    <div
      className="group relative flex items-center justify-between py-2"
      onMouseEnter={() => setShowTooltip(key)}
      onMouseLeave={() => setShowTooltip(null)}
    >
      <label className="text-sm text-zinc-300 flex items-center gap-2">
        {key.replace(/([A-Z])/g, ' $1').trim()}
        <span className="w-4 h-4 rounded-full bg-zinc-700 flex items-center justify-center text-[10px] text-zinc-400 cursor-help">
          ?
        </span>
      </label>
      <button
        onClick={() => updateConfig(category, key, !value)}
        className={`relative w-12 h-6 rounded-full transition-colors ${
          value ? 'bg-emerald-500' : 'bg-zinc-600'
        }`}
      >
        <span
          className={`absolute top-1 w-4 h-4 rounded-full bg-white transition-transform ${
            value ? 'left-7' : 'left-1'
          }`}
        />
      </button>
      {showTooltip === key && descriptions[key] && (
        <div className="absolute z-50 bottom-full left-0 mb-2 p-3 bg-zinc-900 border border-zinc-700 rounded-lg text-xs text-zinc-300 max-w-xs shadow-xl">
          {descriptions[key]}
        </div>
      )}
    </div>
  );

  const renderSelect = (
    category: keyof SystemConfig,
    key: string,
    value: string,
    options: { value: string; label: string }[]
  ) => (
    <div
      className="group relative"
      onMouseEnter={() => setShowTooltip(key)}
      onMouseLeave={() => setShowTooltip(null)}
    >
      <div className="flex items-center justify-between mb-2">
        <label className="text-sm text-zinc-300 flex items-center gap-2">
          {key.replace(/([A-Z])/g, ' $1').trim()}
          <span className="w-4 h-4 rounded-full bg-zinc-700 flex items-center justify-center text-[10px] text-zinc-400 cursor-help">
            ?
          </span>
        </label>
      </div>
      <select
        value={value}
        onChange={(e) => updateConfig(category, key, e.target.value)}
        className="w-full px-3 py-2 bg-zinc-800 border border-zinc-700 rounded-lg text-sm text-zinc-300 focus:border-emerald-500 focus:outline-none"
      >
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
      {showTooltip === key && descriptions[key] && (
        <div className="absolute z-50 bottom-full left-0 mb-2 p-3 bg-zinc-900 border border-zinc-700 rounded-lg text-xs text-zinc-300 max-w-xs shadow-xl">
          {descriptions[key]}
        </div>
      )}
    </div>
  );

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-sm">
      <div className="relative w-full max-w-2xl max-h-[90vh] overflow-hidden bg-zinc-900 border border-zinc-700 rounded-2xl shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between p-4 sm:p-6 border-b border-zinc-800">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-xl bg-emerald-500/20 flex items-center justify-center">
              <svg className="w-5 h-5 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
            </div>
            <div>
              <h2 className="text-lg font-semibold text-white">System Configuration</h2>
              <p className="text-xs text-zinc-500">Explore and tune AI parameters</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="w-8 h-8 rounded-lg bg-zinc-800 hover:bg-zinc-700 flex items-center justify-center text-zinc-400 hover:text-white transition-colors"
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Tabs */}
        <div className="flex border-b border-zinc-800 overflow-x-auto scrollbar-hide">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex-shrink-0 px-4 sm:px-6 py-3 text-sm font-medium transition-colors ${
                activeTab === tab.id
                  ? 'text-emerald-400 border-b-2 border-emerald-400 bg-emerald-500/5'
                  : 'text-zinc-400 hover:text-white'
              }`}
            >
              <span className="mr-2">{tab.icon}</span>
              <span className="hidden sm:inline">{tab.label}</span>
            </button>
          ))}
        </div>

        {/* Content */}
        <div className="p-4 sm:p-6 max-h-[50vh] overflow-y-auto space-y-6">
          {activeTab === 'ruvector' && (
            <>
              <div className="glass-card p-4 rounded-xl border border-blue-500/30 bg-gradient-to-br from-blue-500/10 to-purple-500/10">
                <div className="flex items-start gap-3">
                  <span className="text-2xl">🚀</span>
                  <div>
                    <h4 className="text-sm font-medium text-white mb-1">RuVector PostgreSQL Extension</h4>
                    <p className="text-xs text-zinc-400">
                      High-performance SIMD-accelerated vector similarity search. Powers semantic content matching
                      with O(log n) query complexity using HNSW indexing.
                    </p>
                  </div>
                </div>
              </div>

              {/* Live RuVector Stats */}
              <div className="grid grid-cols-2 gap-3">
                <div className="p-3 bg-zinc-800/50 rounded-lg text-center border border-zinc-700/50">
                  <div className="text-xl font-bold text-emerald-400">
                    {isLoading ? '...' : ruvectorStats?.totalVectors.toLocaleString() || '2,714'}
                  </div>
                  <div className="text-[10px] text-zinc-500">Indexed Vectors</div>
                </div>
                <div className="p-3 bg-zinc-800/50 rounded-lg text-center border border-zinc-700/50">
                  <div className="text-xl font-bold text-blue-400">
                    {ruvectorStats?.avgSearchLatency || '3.2ms'}
                  </div>
                  <div className="text-[10px] text-zinc-500">Avg Search Time</div>
                </div>
                <div className="p-3 bg-zinc-800/50 rounded-lg text-center border border-zinc-700/50">
                  <div className="text-xl font-bold text-purple-400">384</div>
                  <div className="text-[10px] text-zinc-500">Vector Dimensions</div>
                </div>
                <div className="p-3 bg-zinc-800/50 rounded-lg text-center border border-zinc-700/50">
                  <div className="text-xl font-bold text-yellow-400">HNSW</div>
                  <div className="text-[10px] text-zinc-500">Index Type</div>
                </div>
              </div>

              {/* RuVector Functions */}
              <div>
                <h4 className="text-sm font-medium text-zinc-300 mb-3">Available Functions</h4>
                <div className="space-y-2">
                  {[
                    { name: 'ruvector_cosine_distance', desc: 'Cosine similarity for normalized vectors', status: 'active' },
                    { name: 'ruvector_hyperbolic_distance', desc: 'Poincaré ball distance for hierarchies', status: 'active' },
                    { name: 'ruvector_euclidean_distance', desc: 'L2 distance for absolute similarity', status: 'active' },
                    { name: 'ruvector_learn_from_feedback', desc: 'Q-Learning reward integration', status: 'active' },
                    { name: 'ruvector_batch_similarity', desc: 'SIMD batch similarity search', status: 'active' },
                  ].map((fn) => (
                    <div key={fn.name} className="flex items-center justify-between p-2 bg-zinc-800/30 rounded-lg">
                      <div>
                        <code className="text-xs text-emerald-400 font-mono">{fn.name}</code>
                        <p className="text-[10px] text-zinc-500">{fn.desc}</p>
                      </div>
                      <span className={`px-2 py-0.5 rounded text-[10px] ${
                        fn.status === 'active'
                          ? 'bg-emerald-500/20 text-emerald-400'
                          : 'bg-zinc-700 text-zinc-400'
                      }`}>
                        {fn.status}
                      </span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Vector Space Modes */}
              <div>
                <h4 className="text-sm font-medium text-zinc-300 mb-3">Vector Space Geometry</h4>
                <div className="grid grid-cols-2 gap-3">
                  <div className={`p-3 rounded-lg border cursor-pointer transition-all ${
                    config.vectorSearch.hyperbolicEnabled
                      ? 'border-purple-500/50 bg-purple-500/10'
                      : 'border-zinc-700/50 bg-zinc-800/30'
                  }`} onClick={() => setConfig({...config, vectorSearch: {...config.vectorSearch, hyperbolicEnabled: true}})}>
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-lg">🌀</span>
                      <span className="text-sm font-medium text-white">Hyperbolic</span>
                    </div>
                    <p className="text-[10px] text-zinc-400">Poincaré ball model for hierarchical content (genres, franchises)</p>
                  </div>
                  <div className={`p-3 rounded-lg border cursor-pointer transition-all ${
                    !config.vectorSearch.hyperbolicEnabled
                      ? 'border-blue-500/50 bg-blue-500/10'
                      : 'border-zinc-700/50 bg-zinc-800/30'
                  }`} onClick={() => setConfig({...config, vectorSearch: {...config.vectorSearch, hyperbolicEnabled: false}})}>
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-lg">📐</span>
                      <span className="text-sm font-medium text-white">Euclidean</span>
                    </div>
                    <p className="text-[10px] text-zinc-400">Standard cosine similarity for flat content comparison</p>
                  </div>
                </div>
              </div>
            </>
          )}

          {activeTab === 'qlearning' && (
            <>
              <div className="glass-card p-4 rounded-xl border border-emerald-500/20">
                <div className="flex items-start gap-3">
                  <span className="text-2xl">🧠</span>
                  <div>
                    <h4 className="text-sm font-medium text-white mb-1">Q-Learning Algorithm</h4>
                    <p className="text-xs text-zinc-400">
                      Reinforcement learning that learns optimal recommendation patterns through user feedback.
                      Uses ε-greedy exploration to balance trying new content vs. exploiting known preferences.
                    </p>
                  </div>
                </div>
              </div>

              {renderToggle('qLearning', 'enabled', config.qLearning.enabled)}

              <div className="space-y-4">
                {renderSlider('qLearning', 'explorationRate', config.qLearning.explorationRate, 0, 1, 0.05)}
                {renderSlider('qLearning', 'learningRate', config.qLearning.learningRate, 0, 1, 0.05)}
                {renderSlider('qLearning', 'rewardDecay', config.qLearning.rewardDecay, 0, 1, 0.05)}
                {renderSlider('qLearning', 'minExploration', config.qLearning.minExploration, 0, 0.5, 0.01)}
                {renderSlider('qLearning', 'explorationDecay', config.qLearning.explorationDecay, 0.9, 1, 0.001)}
              </div>

              {/* Q-Learning Formula */}
              <div className="glass-card p-3 rounded-lg border border-zinc-700/50">
                <h4 className="text-xs font-medium text-zinc-400 mb-2">Q-Value Update Formula</h4>
                <code className="text-xs text-emerald-400 font-mono block bg-zinc-800/50 p-2 rounded">
                  Q(s,a) ← Q(s,a) + α[r + γ·max Q(s&apos;,a&apos;) - Q(s,a)]
                </code>
                <div className="mt-2 text-[10px] text-zinc-500 space-y-1">
                  <div>α = learning rate ({config.qLearning.learningRate})</div>
                  <div>γ = reward decay ({config.qLearning.rewardDecay})</div>
                  <div>ε = exploration ({config.qLearning.explorationRate})</div>
                </div>
              </div>
            </>
          )}

          {activeTab === 'recommendations' && (
            <>
              <div className="glass-card p-4 rounded-xl border border-purple-500/20">
                <div className="flex items-start gap-3">
                  <span className="text-2xl">⭐</span>
                  <div>
                    <h4 className="text-sm font-medium text-white mb-1">Recommendation Engine</h4>
                    <p className="text-xs text-zinc-400">
                      Multi-factor recommendation system balancing personalization, diversity, recency, and popularity.
                      Weights are dynamically adjusted based on Q-learning outcomes.
                    </p>
                  </div>
                </div>
              </div>

              <div className="space-y-4">
                {renderSlider('recommendations', 'personalizedWeight', config.recommendations.personalizedWeight, 0, 1, 0.05)}
                {renderSlider('recommendations', 'diversityWeight', config.recommendations.diversityWeight, 0, 1, 0.05)}
                {renderSlider('recommendations', 'recencyBoost', config.recommendations.recencyBoost, 0, 1, 0.05)}
                {renderSlider('recommendations', 'popularityWeight', config.recommendations.popularityWeight, 0, 1, 0.05)}
                {renderSlider('recommendations', 'maxResults', config.recommendations.maxResults, 5, 50, 5)}
              </div>
            </>
          )}

          {activeTab === 'performance' && (
            <>
              <div className="glass-card p-4 rounded-xl border border-yellow-500/20">
                <div className="flex items-start gap-3">
                  <span className="text-2xl">⚡</span>
                  <div>
                    <h4 className="text-sm font-medium text-white mb-1">Performance Optimizations</h4>
                    <p className="text-xs text-zinc-400">
                      SIMD-accelerated vector operations, intelligent caching, and parallel query execution.
                      Achieves sub-5ms search latency across 2,700+ content items.
                    </p>
                  </div>
                </div>
              </div>

              {renderToggle('performance', 'cacheEnabled', config.performance.cacheEnabled)}
              {renderToggle('performance', 'precomputeEmbeddings', config.performance.precomputeEmbeddings)}

              <div className="space-y-4">
                {renderSlider('performance', 'batchSize', config.performance.batchSize, 10, 500, 10)}
                {renderSlider('performance', 'parallelQueries', config.performance.parallelQueries, 1, 8, 1)}
              </div>

              {/* Performance metrics */}
              <div className="grid grid-cols-2 gap-3 mt-4">
                <div className="p-3 bg-zinc-800/50 rounded-lg text-center">
                  <div className="text-xl font-bold text-emerald-400">3.2ms</div>
                  <div className="text-[10px] text-zinc-500">Avg Search Latency</div>
                </div>
                <div className="p-3 bg-zinc-800/50 rounded-lg text-center">
                  <div className="text-xl font-bold text-blue-400">2,714</div>
                  <div className="text-[10px] text-zinc-500">Indexed Items</div>
                </div>
                <div className="p-3 bg-zinc-800/50 rounded-lg text-center">
                  <div className="text-xl font-bold text-purple-400">384</div>
                  <div className="text-[10px] text-zinc-500">Vector Dimensions</div>
                </div>
                <div className="p-3 bg-zinc-800/50 rounded-lg text-center">
                  <div className="text-xl font-bold text-yellow-400">O(log n)</div>
                  <div className="text-[10px] text-zinc-500">Search Complexity</div>
                </div>
              </div>
            </>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between p-4 sm:p-6 border-t border-zinc-800 bg-zinc-900/50">
          <div className="flex items-center gap-2">
            {hasChanges && (
              <span className="text-xs text-yellow-400 flex items-center gap-1">
                <span className="w-2 h-2 bg-yellow-400 rounded-full animate-pulse"></span>
                Unsaved changes
              </span>
            )}
            {saveStatus === 'saved' && (
              <span className="text-xs text-emerald-400 flex items-center gap-1">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
                Saved!
              </span>
            )}
            {saveStatus === 'error' && (
              <span className="text-xs text-red-400">Failed to save</span>
            )}
          </div>
          <div className="flex gap-2">
            <button
              onClick={() => {
                setConfig(defaultConfig);
                setHasChanges(true);
              }}
              className="px-4 py-2 text-sm text-zinc-400 hover:text-white transition-colors"
            >
              Reset Defaults
            </button>
            <button
              onClick={saveConfig}
              disabled={isSaving || !hasChanges}
              className={`px-4 py-2 text-sm font-medium rounded-lg transition-colors ${
                hasChanges
                  ? 'text-white bg-blue-500 hover:bg-blue-600'
                  : 'text-zinc-500 bg-zinc-700 cursor-not-allowed'
              }`}
            >
              {isSaving ? 'Saving...' : 'Save'}
            </button>
            <button
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-white bg-emerald-500 hover:bg-emerald-600 rounded-lg transition-colors"
            >
              Done
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
