'use client';

import { motion } from 'framer-motion';
import { Heart, Info } from 'lucide-react';
import type { Movie, TVShow } from '@/types/media';

interface ChatRecommendationCardProps {
    content: Movie | TVShow;
    isLiked: boolean;
    onToggleLike: (content: Movie | TVShow) => void;
    onDetails: (content: Movie | TVShow) => void;
}

export function ChatRecommendationCard({ content, isLiked, onToggleLike, onDetails }: ChatRecommendationCardProps) {
    const posterUrl = content.posterPath
        ? `https://image.tmdb.org/t/p/w500${content.posterPath}`
        : null;

    return (
        <motion.div
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            className="flex-shrink-0 w-40 bg-white/5 border border-white/10 rounded-xl overflow-hidden flex flex-col"
        >
            <div className="relative aspect-[2/3] bg-gray-800">
                {posterUrl ? (
                    <img
                        src={posterUrl}
                        alt={content.title}
                        className="w-full h-full object-cover"
                    />
                ) : (
                    <div className="w-full h-full flex items-center justify-center text-white/20">
                        <span className="text-xs">No Image</span>
                    </div>
                )}
                <button
                    onClick={() => onToggleLike(content)}
                    className="absolute top-2 right-2 p-1.5 bg-black/50 backdrop-blur-sm rounded-full hover:bg-black/70 transition-colors"
                >
                    <Heart className={`w-4 h-4 ${isLiked ? 'fill-red-500 text-red-500' : 'text-white'}`} />
                </button>
            </div>

            <div className="p-3 flex flex-col flex-grow">
                <h4 className="font-medium text-white text-sm line-clamp-2 mb-1" title={content.title}>
                    {content.title}
                </h4>
                <div className="text-xs text-white/40 mb-3">
                    {content.releaseDate ? new Date(content.releaseDate).getFullYear() : 'N/A'}
                </div>

                <button
                    onClick={() => onDetails(content)}
                    className="mt-auto w-full py-1.5 px-3 bg-white/10 hover:bg-white/20 rounded-lg text-xs font-medium text-white transition-colors flex items-center justify-center gap-1.5"
                >
                    <Info className="w-3 h-3" />
                    Details
                </button>
            </div>
        </motion.div>
    );
}
