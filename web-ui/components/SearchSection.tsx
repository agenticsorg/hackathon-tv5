'use client';

import { useState } from 'react';

interface SearchSectionProps {
  onSearch: (query: string) => void;
}

export default function SearchSection({ onSearch }: SearchSectionProps) {
  const [query, setQuery] = useState('');
  const [selectedMood, setSelectedMood] = useState<string | null>(null);

  const moods = [
    { id: 'energetic', label: '⚡ Energetic', color: 'bg-semantic-info' },
    { id: 'relaxed', label: '😌 Relaxed', color: 'bg-semantic-success' },
    { id: 'thrilling', label: '🎢 Thrilling', color: 'bg-semantic-warning' },
    { id: 'thoughtful', label: '🤔 Thoughtful', color: 'bg-youtube-red' },
  ];

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (query.trim()) {
      onSearch(query);
    }
  };

  return (
    <section className="py-8">
      {/* Hero Section */}
      <div className="text-center mb-8">
        <h2 className="text-4xl md:text-5xl font-bold text-text-dark-primary mb-4">
          Find What to Watch in <span className="text-youtube-red">6 Seconds</span>
        </h2>
        <p className="text-lg text-text-dark-secondary max-w-2xl mx-auto">
          Our 8 specialized AI agents search across 5 platforms to find personalized recommendations just for you
        </p>
      </div>

      {/* Search Form */}
      <form onSubmit={handleSubmit} className="max-w-3xl mx-auto mb-6">
        <div className="relative">
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="What do you feel like watching? (e.g., 'action movies with strong female leads')"
            className="w-full h-14 px-6 pr-32 rounded-pill bg-bg-dark-card border-2 border-bg-dark-border
                     text-text-dark-primary placeholder-text-dark-secondary
                     focus:border-youtube-red focus:outline-none transition-colors text-base"
          />
          <button
            type="submit"
            className="absolute right-2 top-2 h-10 px-6 bg-youtube-red hover:bg-youtube-red-dark
                     text-white font-medium rounded-pill transition-colors"
          >
            Search
          </button>
        </div>
      </form>

      {/* Quick Mood Filters */}
      <div className="max-w-3xl mx-auto">
        <p className="text-sm text-text-dark-secondary mb-3 text-center">Quick mood filters:</p>
        <div className="flex flex-wrap justify-center gap-2">
          {moods.map((mood) => (
            <button
              key={mood.id}
              onClick={() => {
                setSelectedMood(mood.id);
                setQuery(`I'm feeling ${mood.label.split(' ')[1].toLowerCase()}`);
              }}
              className={`px-4 py-2 rounded-pill text-sm font-medium transition-all
                ${selectedMood === mood.id
                  ? `${mood.color} text-white shadow-card`
                  : 'bg-bg-dark-card text-text-dark-secondary hover:bg-bg-dark-border border border-bg-dark-border'
                }`}
            >
              {mood.label}
            </button>
          ))}
        </div>
      </div>

      {/* Platform Coverage Indicator */}
      <div className="max-w-3xl mx-auto mt-6 flex items-center justify-center space-x-4 text-xs text-text-dark-secondary">
        <span className="flex items-center">
          <span className="w-2 h-2 bg-semantic-success rounded-full mr-2"></span>
          Searching 5 platforms
        </span>
        <span className="flex items-center">
          <span className="w-2 h-2 bg-semantic-info rounded-full mr-2"></span>
          8 AI agents active
        </span>
        <span className="flex items-center">
          <span className="w-2 h-2 bg-semantic-warning rounded-full mr-2"></span>
          Real-time analysis
        </span>
      </div>
    </section>
  );
}
