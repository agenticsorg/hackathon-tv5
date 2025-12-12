'use client';

import { useState } from 'react';
import { useMatchStore } from '@/store/match-store';
import { ArrowLeft } from 'lucide-react';
import Link from 'next/link';
import { motion, AnimatePresence } from 'framer-motion';
import { DetailsModal } from '@/components/DetailsModal';
import type { MediaContent } from '@/types/media';

export default function HistoryPage() {
    const { likedContent } = useMatchStore();
    const [selectedContent, setSelectedContent] = useState<MediaContent | null>(null);

    return (
        <main className="min-h-screen bg-black text-white p-6">
            <header className="flex items-center gap-4 mb-8">
                <Link href="/" className="p-2 rounded-full bg-white/10 hover:bg-white/20 transition-colors">
                    <ArrowLeft className="w-6 h-6" />
                </Link>
                <h1 className="text-2xl font-bold">Your History</h1>
            </header>

            {likedContent.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-[60vh] text-center opacity-60">
                    <p className="text-xl mb-2">No liked content yet</p>
                    <p className="text-sm">Start swiping to build your history!</p>
                </div>
            ) : (
                <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
                    {likedContent.slice().reverse().map((item) => (
                        <motion.div
                            key={item.id}
                            initial={{ opacity: 0, y: 20 }}
                            animate={{ opacity: 1, y: 0 }}
                            className="relative aspect-[2/3] rounded-xl overflow-hidden bg-gray-900 group cursor-pointer"
                            onClick={() => setSelectedContent(item)}
                            whileHover={{ scale: 1.02 }}
                            whileTap={{ scale: 0.98 }}
                        >
                            {item.posterPath ? (
                                <img
                                    src={`https://image.tmdb.org/t/p/w500${item.posterPath}`}
                                    alt={item.title}
                                    className="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
                                />
                            ) : (
                                <div className="w-full h-full flex items-center justify-center bg-gray-800">
                                    <span className="text-xs text-center p-2">{item.title}</span>
                                </div>
                            )}
                            <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity flex flex-col justify-end p-3">
                                <h3 className="font-bold text-sm line-clamp-2">{item.title}</h3>
                                <p className="text-xs text-white/60">
                                    {new Date(item.releaseDate || '').getFullYear() || 'N/A'}
                                </p>
                            </div>
                        </motion.div>
                    ))}
                </div>
            )}

            <AnimatePresence>
                {selectedContent && (
                    <DetailsModal
                        content={selectedContent}
                        onClose={() => setSelectedContent(null)}
                    />
                )}
            </AnimatePresence>
        </main>
    );
}
