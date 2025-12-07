'use client';

interface MoodSelectorProps {
  selectedMood: string | null;
  onMoodChange: (mood: string | null) => void;
}

const MOODS = [
  { id: 'funny', emoji: '😄', label: 'Funny', color: 'from-yellow-500/20 to-orange-500/20', border: 'border-yellow-500/30' },
  { id: 'exciting', emoji: '🔥', label: 'Exciting', color: 'from-red-500/20 to-orange-500/20', border: 'border-red-500/30' },
  { id: 'romantic', emoji: '💕', label: 'Romantic', color: 'from-pink-500/20 to-rose-500/20', border: 'border-pink-500/30' },
  { id: 'thoughtful', emoji: '🤔', label: 'Thoughtful', color: 'from-purple-500/20 to-indigo-500/20', border: 'border-purple-500/30' },
  { id: 'scary', emoji: '😱', label: 'Scary', color: 'from-zinc-600/30 to-zinc-800/30', border: 'border-zinc-500/30' },
  { id: 'relaxing', emoji: '😌', label: 'Relaxing', color: 'from-emerald-500/20 to-teal-500/20', border: 'border-emerald-500/30' },
];

export function MoodSelector({ selectedMood, onMoodChange }: MoodSelectorProps) {
  return (
    <div className="flex flex-wrap gap-3 justify-center">
      {MOODS.map((mood) => {
        const isSelected = selectedMood === mood.id;
        return (
          <button
            key={mood.id}
            onClick={() => onMoodChange(isSelected ? null : mood.id)}
            className={`
              group relative px-5 py-3 rounded-2xl font-medium transition-all duration-300
              ${isSelected
                ? `bg-gradient-to-br ${mood.color} border ${mood.border} shadow-lg scale-105`
                : 'bg-zinc-800/50 border border-zinc-700/50 hover:border-zinc-600 hover:bg-zinc-800'
              }
            `}
          >
            {/* Glow effect when selected */}
            {isSelected && (
              <div className="absolute inset-0 rounded-2xl bg-gradient-to-br opacity-30 blur-xl -z-10"
                   style={{ background: `linear-gradient(to bottom right, var(--accent-glow), transparent)` }} />
            )}

            <span className="flex items-center gap-2">
              <span className={`text-xl transition-transform duration-300 ${isSelected ? 'scale-110' : 'group-hover:scale-110'}`}>
                {mood.emoji}
              </span>
              <span className={`text-sm ${isSelected ? 'text-white' : 'text-zinc-400 group-hover:text-white'}`}>
                {mood.label}
              </span>
            </span>

            {/* Subtle indicator when selected */}
            {isSelected && (
              <span className="absolute -top-1 -right-1 w-3 h-3 rounded-full bg-emerald-500 border-2 border-zinc-900 animate-pulse" />
            )}
          </button>
        );
      })}
    </div>
  );
}
