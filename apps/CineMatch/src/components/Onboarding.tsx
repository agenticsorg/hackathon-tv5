'use client';

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Search, Users, Clock, Film, Tv, Clapperboard, Video, Zap } from 'lucide-react';
import { useMatchStore } from '@/store/match-store';
import { searchMulti } from '@/lib/tmdb';
import type { MediaContent } from '@/types/media';

export function Onboarding() {
    const [step, setStep] = useState(0);
    const [searchQuery, setSearchQuery] = useState('');
    const [searchResults, setSearchResults] = useState<MediaContent[]>([]);
    const { setPreferences, completeOnboarding } = useMatchStore();

    const handleSearch = async (query: string) => {
        setSearchQuery(query);
        if (query.length > 2) {
            const { results } = await searchMulti(query);
            setSearchResults(results.slice(0, 5));
        } else {
            setSearchResults([]);
        }
    };

    const selectMovie = (movie: MediaContent) => {
        setPreferences({
            favoriteMovieId: movie.id,
            favoriteMediaType: movie.mediaType
        });
        setStep(3); // Go to Intent
    };

    const steps = [
        // Step 0: Age
        <div key="step0" className="space-y-6">
            <div className="text-center space-y-2">
                <h2 className="text-2xl font-bold text-white">How old are you?</h2>
                <p className="text-white/60">We'll filter content based on this.</p>
            </div>
            <div className="grid grid-cols-1 gap-3">
                {[
                    { id: 'under_18', label: 'Under 18' },
                    { id: '18_30', label: '18 - 30' },
                    { id: '30_50', label: '30 - 50' },
                    { id: '50_plus', label: '50+' },
                ].map((option) => (
                    <button
                        key={option.id}
                        onClick={() => { setPreferences({ age: option.id as any }); setStep(1); }}
                        className="p-4 bg-white/5 border border-white/10 rounded-xl hover:bg-white/10 transition-all text-left font-medium text-white"
                    >
                        {option.label}
                    </button>
                ))}
            </div>
        </div>,

        // Step 1: Content Type
        <div key="step1" className="space-y-6">
            <div className="text-center space-y-2">
                <h2 className="text-2xl font-bold text-white">What do you want to watch?</h2>
            </div>
            <div className="grid grid-cols-2 gap-3">
                {[
                    { id: 'movie', label: 'Movies', icon: Film },
                    { id: 'tv', label: 'TV Shows', icon: Tv },
                    { id: 'animation', label: 'Animation', icon: Zap },
                    { id: 'anime', label: 'Anime', icon: Zap }, // Using Zap for now, maybe find better icon
                    { id: 'spectacle', label: 'Spectacle', icon: Clapperboard },
                    { id: 'short_film', label: 'Short Film', icon: Video },
                ].map((option) => (
                    <button
                        key={option.id}
                        onClick={() => { setPreferences({ contentType: option.id as any }); setStep(2); }}
                        className="p-4 bg-white/5 border border-white/10 rounded-xl hover:bg-white/10 transition-all flex flex-col items-center justify-center gap-2 aspect-square"
                    >
                        <option.icon className="w-8 h-8 text-white/60" />
                        <span className="font-medium text-white text-sm">{option.label}</span>
                    </button>
                ))}
            </div>
        </div>,

        // Step 2: Favorite Movie
        <div key="step2" className="space-y-6">
            <div className="text-center space-y-2">
                <h2 className="text-2xl font-bold text-white">What's a title you love?</h2>
                <p className="text-white/60">We'll use this to understand your taste.</p>
            </div>
            <div className="relative">
                <div className="relative">
                    <Search className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-white/40" />
                    <input
                        type="text"
                        value={searchQuery}
                        onChange={(e) => handleSearch(e.target.value)}
                        placeholder="Search for a movie or show..."
                        className="w-full bg-white/10 border border-white/10 rounded-xl py-4 pl-12 pr-4 text-white placeholder:text-white/40 focus:outline-none focus:ring-2 focus:ring-primary-500"
                    />
                </div>
                {searchResults.length > 0 && (
                    <div className="absolute z-10 w-full mt-2 bg-gray-900 border border-white/10 rounded-xl shadow-xl max-h-[300px] overflow-y-auto">
                        {searchResults.map((movie) => (
                            <button
                                key={movie.id}
                                onClick={() => selectMovie(movie)}
                                className="w-full p-3 flex items-center gap-3 hover:bg-white/5 transition-colors text-left border-b border-white/5 last:border-0"
                            >
                                {movie.posterPath && (
                                    <img
                                        src={`https://image.tmdb.org/t/p/w92${movie.posterPath}`}
                                        alt={movie.title}
                                        className="w-10 h-14 object-cover rounded"
                                    />
                                )}
                                <div>
                                    <div className="font-medium text-white">{movie.title}</div>
                                    <div className="text-sm text-white/40">
                                        {movie.releaseDate ? new Date(movie.releaseDate).getFullYear() : 'N/A'}
                                    </div>
                                </div>
                            </button>
                        ))}
                    </div>
                )}
            </div>
        </div>,

        // Step 3: Intent
        <div key="step3" className="space-y-6">
            <div className="text-center space-y-2">
                <h2 className="text-2xl font-bold text-white">What's the vibe tonight?</h2>
            </div>
            <div className="grid grid-cols-1 gap-4">
                <button
                    onClick={() => { setPreferences({ intent: 'learn' }); setStep(4); }}
                    className="p-6 bg-white/5 border border-white/10 rounded-2xl hover:bg-white/10 transition-all text-left group"
                >
                    <div className="w-12 h-12 rounded-full bg-blue-500/20 flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
                        <Film className="w-6 h-6 text-blue-400" />
                    </div>
                    <h3 className="text-lg font-bold text-white">I want to learn something</h3>
                    <p className="text-white/60 text-sm mt-1">Documentaries, Biopics, History</p>
                </button>
                <button
                    onClick={() => { setPreferences({ intent: 'kill_time' }); setStep(4); }}
                    className="p-6 bg-white/5 border border-white/10 rounded-2xl hover:bg-white/10 transition-all text-left group"
                >
                    <div className="w-12 h-12 rounded-full bg-purple-500/20 flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
                        <Clock className="w-6 h-6 text-purple-400" />
                    </div>
                    <h3 className="text-lg font-bold text-white">Just kill some time</h3>
                    <p className="text-white/60 text-sm mt-1">Comedy, Action, Easy watching</p>
                </button>
            </div>
        </div>,

        // Step 4: Social
        <div key="step4" className="space-y-6">
            <div className="text-center space-y-2">
                <h2 className="text-2xl font-bold text-white">Who are you watching with?</h2>
            </div>
            <div className="grid grid-cols-2 gap-4">
                {[
                    { id: 'alone', label: 'Just Me', icon: Users },
                    { id: 'partner', label: 'My Partner', icon: Users },
                    { id: 'friends', label: 'Friends', icon: Users },
                    { id: 'family', label: 'Family', icon: Users },
                ].map((option) => (
                    <button
                        key={option.id}
                        onClick={() => { setPreferences({ social: option.id as any }); setStep(5); }}
                        className="p-4 bg-white/5 border border-white/10 rounded-2xl hover:bg-white/10 transition-all flex flex-col items-center justify-center gap-3 aspect-square"
                    >
                        <option.icon className="w-8 h-8 text-white/60" />
                        <span className="font-medium text-white">{option.label}</span>
                    </button>
                ))}
            </div>
        </div>,

        // Step 5: Era
        <div key="step5" className="space-y-6">
            <div className="text-center space-y-2">
                <h2 className="text-2xl font-bold text-white">Old school or New school?</h2>
            </div>
            <div className="grid grid-cols-1 gap-4">
                <button
                    onClick={() => { setPreferences({ era: 'new' }); completeOnboarding(); }}
                    className="p-6 bg-white/5 border border-white/10 rounded-2xl hover:bg-white/10 transition-all flex items-center justify-between group"
                >
                    <div className="text-left">
                        <h3 className="text-lg font-bold text-white">New Releases</h3>
                        <p className="text-white/60 text-sm">Latest hits and modern classics</p>
                    </div>
                    <div className="w-8 h-8 rounded-full bg-white/10 flex items-center justify-center group-hover:bg-white/20">
                        →
                    </div>
                </button>
                <button
                    onClick={() => { setPreferences({ era: 'classic' }); completeOnboarding(); }}
                    className="p-6 bg-white/5 border border-white/10 rounded-2xl hover:bg-white/10 transition-all flex items-center justify-between group"
                >
                    <div className="text-left">
                        <h3 className="text-lg font-bold text-white">Golden Archives</h3>
                        <p className="text-white/60 text-sm">Cult classics and timeless movies</p>
                    </div>
                    <div className="w-8 h-8 rounded-full bg-white/10 flex items-center justify-center group-hover:bg-white/20">
                        →
                    </div>
                </button>
            </div>
        </div>
    ];

    return (
        <div className="w-full max-w-md mx-auto p-6">
            <AnimatePresence mode="wait">
                <motion.div
                    key={step}
                    initial={{ opacity: 0, x: 20 }}
                    animate={{ opacity: 1, x: 0 }}
                    exit={{ opacity: 0, x: -20 }}
                    transition={{ duration: 0.3 }}
                >
                    {steps[step]}
                </motion.div>
            </AnimatePresence>

            <div className="mt-8 flex justify-center gap-2">
                {steps.map((_, i) => (
                    <div
                        key={i}
                        className={`w-2 h-2 rounded-full transition-colors ${i === step ? 'bg-white' : 'bg-white/20'
                            }`}
                    />
                ))}
            </div>

            {/* Navigation Buttons */}
            <div className="mt-8 flex justify-between items-center px-2">
                <div className="w-16">
                    {step > 0 && (
                        <button
                            onClick={() => setStep(prev => Math.max(0, prev - 1))}
                            className="text-white/60 hover:text-white text-sm font-medium transition-colors px-4 py-2"
                        >
                            Back
                        </button>
                    )}
                </div>
                <button
                    onClick={() => {
                        if (step === steps.length - 1) {
                            completeOnboarding();
                        } else {
                            setStep(prev => Math.min(steps.length - 1, prev + 1));
                        }
                    }}
                    className="text-white/60 hover:text-white text-sm font-medium transition-colors px-4 py-2"
                >
                    Skip
                </button>
            </div>
        </div>
    );
}
