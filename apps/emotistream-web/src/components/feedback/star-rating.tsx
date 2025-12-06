'use client';

import { motion } from 'framer-motion';
import { Star } from 'lucide-react';

interface StarRatingProps {
  value: number;
  onChange: (value: number) => void;
  max?: number;
  size?: 'sm' | 'md' | 'lg';
}

export function StarRating({ value, onChange, max = 5, size = 'md' }: StarRatingProps) {
  const sizeClasses = {
    sm: 'w-6 h-6',
    md: 'w-10 h-10',
    lg: 'w-14 h-14',
  };

  return (
    <div className="flex gap-2">
      {Array.from({ length: max }).map((_, index) => {
        const starValue = index + 1;
        return (
          <motion.button
            key={index}
            whileHover={{ scale: 1.1 }}
            whileTap={{ scale: 0.9 }}
            onClick={() => onChange(starValue)}
            className="p-1"
            type="button"
          >
            <Star
              className={`${sizeClasses[size]} transition-colors ${
                starValue <= value
                  ? 'fill-yellow-400 text-yellow-400'
                  : 'text-gray-300 dark:text-gray-600 hover:text-yellow-200'
              }`}
            />
          </motion.button>
        );
      })}
    </div>
  );
}
