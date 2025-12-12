'use client';

import { useState, useEffect, useRef } from 'react';
import Link from 'next/link';
import { ArrowLeft, Send, Sparkles, MessageSquare } from 'lucide-react';
import { useMatchStore, ChatMessage } from '@/store/match-store';
import { motion, AnimatePresence } from 'framer-motion';
import { ChatRecommendationCard } from '@/components/ChatRecommendationCard';
import { DetailsModal } from '@/components/DetailsModal';
import type { Movie, TVShow } from '@/types/media';

const PLACEHOLDERS = [
    "What should I watch ?",
    "Any content you would suggest ?",
    "What media is worth watching ?"
];

export default function ChatPage() {
    const {
        preferences,
        likedContent,
        dislikedContentIds,
        likeContent,
        unlikeContent,
        chatHistory,
        addChatMessage
    } = useMatchStore();

    const [input, setInput] = useState('');
    const [isLoading, setIsLoading] = useState(false);
    const [placeholderIndex, setPlaceholderIndex] = useState(0);
    const [selectedContent, setSelectedContent] = useState<Movie | TVShow | null>(null);
    const messagesEndRef = useRef<HTMLDivElement>(null);

    // Auto-scroll to bottom
    const scrollToBottom = () => {
        messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    };

    useEffect(() => {
        scrollToBottom();
    }, [chatHistory, isLoading]);

    // Rotate placeholders
    useEffect(() => {
        const interval = setInterval(() => {
            setPlaceholderIndex((prev) => (prev + 1) % PLACEHOLDERS.length);
        }, 3000);
        return () => clearInterval(interval);
    }, []);

    const handleSubmit = async (e?: React.FormEvent) => {
        e?.preventDefault();
        if (isLoading) return;

        const messageText = input.trim() || PLACEHOLDERS[placeholderIndex];
        if (!messageText) return;

        // Add user message
        const userMsg: ChatMessage = { role: 'user', content: messageText };
        addChatMessage(userMsg);
        setInput('');
        setIsLoading(true);

        try {
            // Prepare messages for API (convert store messages to API format)
            const apiMessages = chatHistory.concat(userMsg).map(m => ({
                role: m.role,
                content: m.content
            }));

            const response = await fetch('/api/chat', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    messages: apiMessages,
                    context: {
                        preferences,
                        likedContent: likedContent.map(c => c.title),
                        dislikedContent: dislikedContentIds // IDs only usually, but let's send what we have
                    }
                }),
            });

            if (!response.ok) {
                const errorData = await response.json().catch(() => ({}));
                throw new Error(errorData.error || 'Failed to fetch response');
            }

            const data = await response.json();

            // Add assistant message with recommendations
            addChatMessage({
                role: 'assistant',
                content: data.message,
                recommendations: data.recommendations
            });

        } catch (error) {
            console.error('Chat error:', error);
            let errorMessage = error instanceof Error ? error.message : String(error);
            if (errorMessage.includes('429') || errorMessage.includes('Quota')) {
                errorMessage = "Please wait a minute and try again.";
            }
            addChatMessage({ role: 'assistant', content: errorMessage });
        } finally {
            setIsLoading(false);
        }
    };

    const handleToggleLike = (content: Movie | TVShow) => {
        const isLiked = likedContent.some(c => c.id === content.id);
        if (isLiked) {
            unlikeContent(content.id);
        } else {
            likeContent(content);
        }
    };

    const handleDetails = (content: Movie | TVShow) => {
        setSelectedContent(content);
    };

    return (
        <main className="min-h-screen bg-black text-white flex flex-col">
            {/* Header */}
            <header className="p-4 flex items-center gap-4 border-b border-white/10 bg-black/50 backdrop-blur-md sticky top-0 z-10">
                <Link href="/" className="p-2 rounded-full hover:bg-white/10 transition-colors">
                    <ArrowLeft className="w-6 h-6" />
                </Link>
                <div className="flex items-center gap-2">
                    <Sparkles className="w-5 h-5 text-pink-500" />
                    <h1 className="text-xl font-bold">CineMatch AI</h1>
                </div>
            </header>

            {/* Chat Area */}
            <div className="flex-1 overflow-y-auto p-4 space-y-6">
                {chatHistory.length === 0 ? (
                    <div className="flex flex-col items-center justify-center h-full text-center text-gray-500 space-y-4 py-20">
                        <MessageSquare className="w-12 h-12 text-gray-700" />
                        <p>Start chatting to get recommendations!</p>
                        <p className="text-sm max-w-xs">I know what you like and dislike, so I can give personalized recommendations.</p>
                    </div>
                ) : (
                    chatHistory.map((msg, idx) => (
                        <div
                            key={idx}
                            className={`flex flex-col ${msg.role === 'user' ? 'items-end' : 'items-start'} gap-2`}
                        >
                            <div
                                className={`max-w-[85%] p-4 rounded-2xl ${msg.role === 'user'
                                    ? 'bg-pink-600 text-white rounded-tr-none'
                                    : 'bg-gray-800 text-gray-200 rounded-tl-none'
                                    }`}
                            >
                                <p className="whitespace-pre-wrap">{msg.content}</p>
                            </div>

                            {/* Recommendations Carousel */}
                            {msg.recommendations && msg.recommendations.length > 0 && (
                                <div className="w-full overflow-x-auto pb-4 pt-2 px-1 -mx-1 scrollbar-hide">
                                    <div className="flex gap-4">
                                        {msg.recommendations.map((rec) => (
                                            <ChatRecommendationCard
                                                key={rec.id}
                                                content={rec}
                                                isLiked={likedContent.some(c => c.id === rec.id)}
                                                onToggleLike={handleToggleLike}
                                                onDetails={handleDetails}
                                            />
                                        ))}
                                    </div>
                                </div>
                            )}
                        </div>
                    ))
                )}

                {isLoading && (
                    <div className="flex justify-start">
                        <div className="bg-gray-800 p-4 rounded-2xl rounded-tl-none flex gap-1">
                            <span className="w-2 h-2 bg-gray-500 rounded-full animate-bounce" style={{ animationDelay: '0ms' }} />
                            <span className="w-2 h-2 bg-gray-500 rounded-full animate-bounce" style={{ animationDelay: '150ms' }} />
                            <span className="w-2 h-2 bg-gray-500 rounded-full animate-bounce" style={{ animationDelay: '300ms' }} />
                        </div>
                    </div>
                )}
                <div ref={messagesEndRef} />
            </div>

            {/* Input Area */}
            <div className="p-4 border-t border-white/10 bg-black/50 backdrop-blur-md">
                <form onSubmit={handleSubmit} className="flex gap-2 max-w-md mx-auto relative">
                    <div className="relative flex-1">
                        <input
                            type="text"
                            value={input}
                            onChange={(e) => setInput(e.target.value)}
                            className="w-full bg-gray-900 border border-white/10 rounded-full px-4 py-3 pr-12 focus:outline-none focus:border-pink-500 transition-colors"
                        />
                        {!input && (
                            <div className="absolute inset-0 flex items-center px-4 pointer-events-none text-gray-500 truncate">
                                <AnimatePresence mode='wait'>
                                    <motion.span
                                        key={placeholderIndex}
                                        initial={{ opacity: 0, y: 10 }}
                                        animate={{ opacity: 1, y: 0 }}
                                        exit={{ opacity: 0, y: -10 }}
                                        transition={{ duration: 0.2 }}
                                    >
                                        {PLACEHOLDERS[placeholderIndex]}
                                    </motion.span>
                                </AnimatePresence>
                            </div>
                        )}
                    </div>
                    <button
                        type="submit"
                        disabled={isLoading}
                        className="p-3 bg-pink-600 rounded-full hover:bg-pink-500 disabled:opacity-50 transition-colors"
                    >
                        <Send className="w-5 h-5" />
                    </button>
                </form>
            </div>

            {/* Details Modal */}
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
