'use client';

import { motion, useMotionValue, useTransform, useAnimation } from 'framer-motion';
import { Info, X, Heart, ExternalLink } from 'lucide-react';
import { useState } from 'react';
import type { MediaContent } from '@/types/media';

interface SwipeCardProps {
    content: MediaContent;
    onSwipeLeft: () => void;
    onSwipeRight: () => void;
    onSwipeUp: () => void;
}

export function SwipeCard({ content, onSwipeLeft, onSwipeRight, onSwipeUp }: SwipeCardProps) {
    const x = useMotionValue(0);
    const y = useMotionValue(0);
    const controls = useAnimation();
    const [showInfo, setShowInfo] = useState(false);

    // Rotation based on x position
    const rotate = useTransform(x, [-200, 200], [-25, 25]);

    // Opacity for "Like" and "Nope" overlays
    const likeOpacity = useTransform(x, [50, 150], [0, 1]);
    const nopeOpacity = useTransform(x, [-50, -150], [0, 1]);
    const infoOpacity = useTransform(y, [-50, -150], [0, 1]);

    const handleDragEnd = async (_: any, info: any) => {
        const offset = info.offset;
        const velocity = info.velocity;

        // Swipe Right (Like)
        if (offset.x > 100 || velocity.x > 500) {
            await controls.start({ x: 500, opacity: 0 });
            onSwipeRight();
        }
        // Swipe Left (Dislike)
        else if (offset.x < -100 || velocity.x < -500) {
            await controls.start({ x: -500, opacity: 0 });
            onSwipeLeft();
        }
        // Swipe Up (Info)
        else if (offset.y < -100 || velocity.y < -500) {
            onSwipeUp();
            // Reset position after a short delay to allow animation if needed,
            // but for now we just snap back so the card is ready when modal closes
            controls.start({ x: 0, y: 0 });
        }
        // Reset
        else {
            controls.start({ x: 0, y: 0 });
        }
    };

    return (
        <motion.div
            drag
            dragConstraints={{ left: 0, right: 0, top: 0, bottom: 0 }}
            style={{ x, y, rotate }}
            animate={controls}
            onDragEnd={handleDragEnd}
            className="absolute inset-0 w-full h-full cursor-grab active:cursor-grabbing"
        >
            <div className="relative w-full h-full rounded-3xl overflow-hidden shadow-2xl bg-gray-900">
                {/* Poster Image */}
                <img
                    src={`https://image.tmdb.org/t/p/w780${content.posterPath}`}
                    alt={content.title}
                    className="w-full h-full object-cover pointer-events-none"
                />

                {/* Gradient Overlay */}
                <div className="absolute inset-0 bg-gradient-to-b from-black/20 via-transparent to-black/90 pointer-events-none" />

                {/* Content Info */}
                <div className="absolute bottom-0 left-0 right-0 p-6 text-white pointer-events-none">
                    <h2 className="text-3xl font-bold mb-2">{content.title}</h2>
                    <div className="flex items-center gap-3 text-sm text-white/80 mb-3">
                        <span className="bg-yellow-500/20 text-yellow-300 px-2 py-0.5 rounded">
                            ★ {content.voteAverage.toFixed(1)}
                        </span>
                        <span>{content.releaseDate ? new Date(content.releaseDate).getFullYear() : 'N/A'}</span>
                        <span>{content.mediaType === 'movie' ? 'Movie' : 'TV Show'}</span>
                    </div>
                    <p className="text-sm text-white/70 line-clamp-2">{content.overview}</p>
                </div>

                {/* Swipe Overlays */}
                <motion.div
                    style={{ opacity: likeOpacity }}
                    className="absolute top-8 right-8 pointer-events-none"
                >
                    <div className="border-4 border-green-500 text-green-500 font-bold text-4xl px-4 py-2 rounded-xl transform rotate-12 bg-black/20 backdrop-blur-sm">
                        LIKE
                    </div>
                </motion.div>

                <motion.div
                    style={{ opacity: nopeOpacity }}
                    className="absolute top-8 left-8 pointer-events-none"
                >
                    <div className="border-4 border-red-500 text-red-500 font-bold text-4xl px-4 py-2 rounded-xl transform -rotate-12 bg-black/20 backdrop-blur-sm">
                        NOPE
                    </div>
                </motion.div>

                <motion.div
                    style={{ opacity: infoOpacity }}
                    className="absolute bottom-32 left-0 right-0 flex justify-center pointer-events-none"
                >
                    <div className="bg-blue-500/80 text-white px-6 py-2 rounded-full backdrop-blur-md flex items-center gap-2">
                        <ExternalLink className="w-4 h-4" />
                        <span>Release Info</span>
                    </div>
                </motion.div>
            </div>
        </motion.div>
    );
}
