import React from 'react';
import { motion } from 'framer-motion';

export function RecommendationSkeleton() {
  return (
    <div className="flex-shrink-0 w-72">
      <div className="overflow-hidden rounded-lg bg-gray-800 shadow-lg">
        {/* Thumbnail Skeleton */}
        <div className="relative h-40 bg-gray-700 animate-pulse">
          <div className="absolute top-2 left-2 w-12 h-6 bg-gray-600 rounded-full" />
        </div>

        {/* Content Skeleton */}
        <div className="p-4 space-y-3">
          {/* Title */}
          <div className="space-y-2">
            <div className="h-5 bg-gray-700 rounded animate-pulse w-3/4" />
            <div className="flex gap-2">
              <div className="h-4 bg-gray-700 rounded animate-pulse w-16" />
              <div className="h-4 bg-gray-700 rounded animate-pulse w-12" />
            </div>
          </div>

          {/* Button */}
          <div className="h-10 bg-gray-700 rounded-lg animate-pulse" />
        </div>
      </div>

      {/* Confidence bar skeleton */}
      <div className="mt-1 h-1 bg-gray-700 rounded-full animate-pulse" />
    </div>
  );
}

interface RecommendationSkeletonGridProps {
  count?: number;
}

export function RecommendationSkeletonGrid({ count = 5 }: RecommendationSkeletonGridProps) {
  return (
    <div className="flex gap-4 overflow-x-auto pb-4 scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-transparent">
      {Array.from({ length: count }).map((_, i) => (
        <motion.div
          key={i}
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: i * 0.05 }}
        >
          <RecommendationSkeleton />
        </motion.div>
      ))}
    </div>
  );
}
