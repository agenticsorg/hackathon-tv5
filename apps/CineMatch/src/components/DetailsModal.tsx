'use client';

import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { X, Star, Calendar, Clock, PlayCircle, ExternalLink } from 'lucide-react';
import type { MediaContent } from '@/types/media';
import { getWatchProviders } from '@/lib/tmdb';

interface DetailsModalProps {
    content: MediaContent;
    onClose: () => void;
}

export function DetailsModal({ content, onClose }: DetailsModalProps) {
    const [providers, setProviders] = useState<{
        flatrate: { provider_name: string; logo_path: string }[];
        link: string;
    }>({ flatrate: [], link: '' });

    useEffect(() => {
        const fetchProviders = async () => {
            try {
                const data = await getWatchProviders(content.id, content.mediaType);
                setProviders(data);
            } catch (error) {
                console.error('Failed to fetch providers:', error);
            }
        };
        fetchProviders();
    }, [content.id, content.mediaType]);

    return (
        <motion.div
            initial={{ opacity: 0, y: '100%' }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: '100%' }}
            transition={{ type: 'spring', damping: 25, stiffness: 200 }}
            className="fixed inset-0 z-50 flex items-end sm:items-center justify-center pointer-events-none"
        >
            {/* Backdrop */}
            <div
                className="absolute inset-0 bg-black/60 backdrop-blur-sm pointer-events-auto"
                onClick={onClose}
            />

            {/* Modal Content */}
            <motion.div
                className="relative w-full max-w-lg bg-gray-900 rounded-t-3xl sm:rounded-3xl overflow-hidden shadow-2xl pointer-events-auto max-h-[90vh] overflow-y-auto"
            >
                {/* Close Button */}
                <button
                    onClick={onClose}
                    className="absolute top-4 right-4 z-10 p-2 bg-black/20 hover:bg-black/40 rounded-full text-white transition-colors"
                >
                    <X className="w-6 h-6" />
                </button>

                {/* Hero Image */}
                <div className="relative aspect-video w-full">
                    <img
                        src={`https://image.tmdb.org/t/p/w780${content.backdropPath || content.posterPath}`}
                        alt={content.title}
                        className="w-full h-full object-cover"
                    />
                    <div className="absolute inset-0 bg-gradient-to-t from-gray-900 to-transparent" />

                    <div className="absolute bottom-0 left-0 p-6">
                        <h2 className="text-3xl font-bold text-white mb-2">{content.title}</h2>
                        <div className="flex items-center gap-4 text-sm text-white/80">
                            <div className="flex items-center gap-1">
                                <Star className="w-4 h-4 text-yellow-500 fill-yellow-500" />
                                <span>{content.voteAverage.toFixed(1)}</span>
                            </div>
                            <div className="flex items-center gap-1">
                                <Calendar className="w-4 h-4" />
                                <span>{new Date(content.releaseDate).getFullYear()}</span>
                            </div>
                            <div className="px-2 py-0.5 bg-white/10 rounded text-xs uppercase">
                                {content.mediaType === 'movie' ? 'Movie' : 'TV Series'}
                            </div>
                        </div>
                    </div>
                </div>

                {/* Details Body */}
                <div className="p-6 space-y-6">
                    {/* Overview */}
                    <div>
                        <h3 className="text-lg font-bold text-white mb-2">Overview</h3>
                        <p className="text-white/70 leading-relaxed">
                            {content.overview}
                        </p>
                    </div>

                    {/* Availability */}
                    <div>
                        <h3 className="text-lg font-bold text-white mb-3">Where to Watch</h3>
                        {providers.flatrate.length > 0 ? (
                            <div className="flex gap-3 overflow-x-auto pb-2">
                                {providers.flatrate.slice(0, 3).map((provider) => (
                                    <a
                                        key={provider.provider_name}
                                        href={providers.link}
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        className="flex-shrink-0 px-4 py-3 bg-white/5 border border-white/10 rounded-xl flex items-center gap-3 hover:bg-white/10 transition-colors cursor-pointer group"
                                        title={`Watch on ${provider.provider_name}`}
                                    >
                                        <img
                                            src={`https://image.tmdb.org/t/p/original${provider.logo_path}`}
                                            alt={provider.provider_name}
                                            className="w-6 h-6 rounded group-hover:scale-110 transition-transform"
                                        />
                                        <span className="text-white font-medium">{provider.provider_name}</span>
                                        <ExternalLink className="w-3 h-3 text-white/40 group-hover:text-white transition-colors" />
                                    </a>
                                ))}
                            </div>
                        ) : (
                            <p className="text-white/40 text-sm">No streaming information available.</p>
                        )}

                        {providers.link && (
                            <a
                                href={providers.link}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="inline-flex items-center gap-2 text-primary-400 hover:text-primary-300 text-sm mt-3"
                            >
                                <span>View all options on TMDB</span>
                                <ExternalLink className="w-4 h-4" />
                            </a>
                        )}
                    </div>
                </div>
            </motion.div>
        </motion.div>
    );
}
