'use client';

import { motion } from 'framer-motion';
import { useMemo } from 'react';

interface MoodRingProps {
  valence: number;  // -1 to 1
  arousal: number;  // -1 to 1
  stress: number;   // 0 to 1
  size?: 'sm' | 'md' | 'lg';
  animate?: boolean;
}

export function MoodRing({ valence, arousal, stress, size = 'md', animate = true }: MoodRingProps) {
  const sizeClasses = {
    sm: 'w-24 h-24',
    md: 'w-40 h-40',
    lg: 'w-56 h-56',
  };

  const gradient = useMemo(() => {
    // Map emotion quadrants to colors
    if (valence > 0 && arousal > 0) return 'from-orange-400 via-yellow-400 to-amber-300';
    if (valence > 0 && arousal <= 0) return 'from-blue-400 via-cyan-400 to-teal-300';
    if (valence <= 0 && arousal > 0) return 'from-red-400 via-pink-500 to-purple-500';
    return 'from-slate-400 via-blue-500 to-indigo-600';
  }, [valence, arousal]);

  const pulseIntensity = stress * 1.5 + 1; // 1 to 2.5

  return (
    <div className="relative flex items-center justify-center">
      {/* Outer glow */}
      <motion.div
        className={`absolute ${sizeClasses[size]} rounded-full bg-gradient-to-br ${gradient} opacity-30 blur-xl`}
        animate={animate ? {
          scale: [1, pulseIntensity, 1],
        } : {}}
        transition={{
          duration: 2 / pulseIntensity,
          repeat: Infinity,
          ease: "easeInOut",
        }}
      />

      {/* Main ring */}
      <motion.div
        className={`${sizeClasses[size]} rounded-full bg-gradient-to-br ${gradient} shadow-2xl
                    flex items-center justify-center`}
        initial={{ scale: 0.8, opacity: 0 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ type: "spring", damping: 20 }}
      >
        {/* Inner circle */}
        <div className="w-3/4 h-3/4 rounded-full bg-white/20 backdrop-blur-sm flex items-center justify-center">
          <motion.span
            className="text-4xl"
            animate={animate ? { scale: [1, 1.1, 1] } : {}}
            transition={{ duration: 1, repeat: Infinity }}
          >
            {getEmotionEmoji(valence, arousal)}
          </motion.span>
        </div>
      </motion.div>
    </div>
  );
}

function getEmotionEmoji(valence: number, arousal: number): string {
  if (valence > 0.3 && arousal > 0.3) return '🤩';
  if (valence > 0.3 && arousal < -0.3) return '😌';
  if (valence < -0.3 && arousal > 0.3) return '😰';
  if (valence < -0.3 && arousal < -0.3) return '😔';
  if (valence > 0.3) return '🙂';
  if (valence < -0.3) return '😟';
  return '😐';
}
