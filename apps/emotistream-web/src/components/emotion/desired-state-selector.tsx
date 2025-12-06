'use client';

import { motion } from 'framer-motion';
import { useState } from 'react';

interface DesiredStateSelectorProps {
  onSelect: (state: { valence: number; arousal: number; stress: number }) => void;
  selectedPreset?: string;
}

const presets = [
  { id: 'relax', label: 'Relax', emoji: '🧘', valence: 0.6, arousal: -0.5, stress: 0.1 },
  { id: 'energize', label: 'Energize', emoji: '⚡', valence: 0.7, arousal: 0.7, stress: 0.2 },
  { id: 'focus', label: 'Focus', emoji: '🎯', valence: 0.3, arousal: 0.3, stress: 0.2 },
  { id: 'sleep', label: 'Sleep', emoji: '😴', valence: 0.4, arousal: -0.8, stress: 0.0 },
];

export function DesiredStateSelector({ onSelect, selectedPreset }: DesiredStateSelectorProps) {
  const [selected, setSelected] = useState(selectedPreset);

  const handleSelect = (preset: typeof presets[0]) => {
    setSelected(preset.id);
    onSelect({
      valence: preset.valence,
      arousal: preset.arousal,
      stress: preset.stress
    });
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      className="p-6 bg-white dark:bg-gray-800 rounded-2xl shadow-lg"
    >
      <h3 className="text-lg font-semibold mb-4 text-gray-800 dark:text-white">
        How do you want to feel?
      </h3>

      <div className="flex flex-wrap gap-3">
        {presets.map((preset) => (
          <motion.button
            key={preset.id}
            onClick={() => handleSelect(preset)}
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.95 }}
            className={`px-5 py-3 rounded-xl font-medium transition-all flex items-center gap-2
                       ${selected === preset.id
                         ? 'bg-gradient-to-r from-blue-500 to-purple-600 text-white shadow-lg'
                         : 'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 hover:bg-gray-200'
                       }`}
          >
            <span className="text-xl">{preset.emoji}</span>
            <span>{preset.label}</span>
          </motion.button>
        ))}
      </div>
    </motion.div>
  );
}
