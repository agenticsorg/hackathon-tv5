'use client';

/**
 * Voice Search Component
 *
 * Provides hands-free voice search using Web Speech API.
 * Features:
 * - Push-to-talk and continuous listening modes
 * - Real-time transcription feedback
 * - Visual microphone indicator
 * - Fallback for unsupported browsers
 */

import { useState, useEffect, useCallback, useRef } from 'react';

// Speech Recognition types (Web Speech API)
interface SpeechRecognitionEvent {
  resultIndex: number;
  results: SpeechRecognitionResultList;
}

interface SpeechRecognitionResultList {
  length: number;
  item(index: number): SpeechRecognitionResult;
  [index: number]: SpeechRecognitionResult;
}

interface SpeechRecognitionResult {
  isFinal: boolean;
  length: number;
  item(index: number): SpeechRecognitionAlternative;
  [index: number]: SpeechRecognitionAlternative;
}

interface SpeechRecognitionAlternative {
  transcript: string;
  confidence: number;
}

interface SpeechRecognition extends EventTarget {
  continuous: boolean;
  interimResults: boolean;
  lang: string;
  start(): void;
  stop(): void;
  abort(): void;
  onresult: ((event: SpeechRecognitionEvent) => void) | null;
  onerror: ((event: { error: string }) => void) | null;
  onend: (() => void) | null;
  onstart: (() => void) | null;
}

declare global {
  interface Window {
    SpeechRecognition?: new () => SpeechRecognition;
    webkitSpeechRecognition?: new () => SpeechRecognition;
  }
}

export interface VoiceSearchProps {
  onResult: (transcript: string) => void;
  onInterimResult?: (transcript: string) => void;
  onError?: (error: string) => void;
  onListeningChange?: (isListening: boolean) => void;
  language?: string;
  continuous?: boolean;
  className?: string;
  disabled?: boolean;
}

export function VoiceSearch({
  onResult,
  onInterimResult,
  onError,
  onListeningChange,
  language = 'en-US',
  continuous = false,
  className = '',
  disabled = false,
}: VoiceSearchProps) {
  const [isListening, setIsListening] = useState(false);
  const [isSupported, setIsSupported] = useState(false);
  const [interimTranscript, setInterimTranscript] = useState('');
  const [permissionDenied, setPermissionDenied] = useState(false);

  const recognitionRef = useRef<SpeechRecognition | null>(null);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Check for browser support
  useEffect(() => {
    const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
    setIsSupported(!!SpeechRecognition);
  }, []);

  // Initialize speech recognition
  useEffect(() => {
    if (!isSupported) return;

    const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
    if (!SpeechRecognition) return;

    const recognition = new SpeechRecognition();
    recognition.continuous = continuous;
    recognition.interimResults = true;
    recognition.lang = language;

    recognition.onstart = () => {
      setIsListening(true);
      onListeningChange?.(true);
    };

    recognition.onresult = (event: SpeechRecognitionEvent) => {
      let interim = '';
      let final = '';

      for (let i = event.resultIndex; i < event.results.length; i++) {
        const result = event.results[i];
        const transcript = result[0].transcript;

        if (result.isFinal) {
          final += transcript;
        } else {
          interim += transcript;
        }
      }

      if (interim) {
        setInterimTranscript(interim);
        onInterimResult?.(interim);
      }

      if (final) {
        setInterimTranscript('');
        onResult(final.trim());

        // Auto-stop in non-continuous mode
        if (!continuous) {
          recognition.stop();
        }
      }
    };

    recognition.onerror = (event: { error: string }) => {
      console.error('Speech recognition error:', event.error);

      if (event.error === 'not-allowed' || event.error === 'permission-denied') {
        setPermissionDenied(true);
        onError?.('Microphone permission denied. Please allow microphone access.');
      } else if (event.error === 'no-speech') {
        onError?.('No speech detected. Please try again.');
      } else if (event.error !== 'aborted') {
        onError?.(`Speech recognition error: ${event.error}`);
      }

      setIsListening(false);
      onListeningChange?.(false);
    };

    recognition.onend = () => {
      setIsListening(false);
      setInterimTranscript('');
      onListeningChange?.(false);

      // Clear any pending timeout
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
        timeoutRef.current = null;
      }
    };

    recognitionRef.current = recognition;

    return () => {
      recognition.abort();
    };
  }, [isSupported, language, continuous, onResult, onInterimResult, onError, onListeningChange]);

  const startListening = useCallback(() => {
    if (!recognitionRef.current || disabled || permissionDenied) return;

    try {
      recognitionRef.current.start();

      // Auto-stop after 30 seconds to prevent indefinite listening
      timeoutRef.current = setTimeout(() => {
        if (recognitionRef.current && isListening) {
          recognitionRef.current.stop();
        }
      }, 30000);
    } catch (error) {
      console.error('Failed to start recognition:', error);
    }
  }, [disabled, permissionDenied, isListening]);

  const stopListening = useCallback(() => {
    if (!recognitionRef.current) return;

    try {
      recognitionRef.current.stop();
    } catch (error) {
      console.error('Failed to stop recognition:', error);
    }
  }, []);

  const toggleListening = useCallback(() => {
    if (isListening) {
      stopListening();
    } else {
      startListening();
    }
  }, [isListening, startListening, stopListening]);

  // Not supported fallback
  if (!isSupported) {
    return (
      <button
        className={`voice-search-btn unsupported ${className}`}
        disabled
        title="Voice search not supported in this browser"
        aria-label="Voice search not supported"
      >
        <MicOffIcon />
      </button>
    );
  }

  // Permission denied state
  if (permissionDenied) {
    return (
      <button
        className={`voice-search-btn permission-denied ${className}`}
        disabled
        title="Microphone permission denied"
        aria-label="Microphone permission denied"
      >
        <MicOffIcon />
      </button>
    );
  }

  return (
    <div className="voice-search-container">
      <button
        className={`voice-search-btn ${isListening ? 'listening' : ''} ${className}`}
        onClick={toggleListening}
        disabled={disabled}
        title={isListening ? 'Stop listening' : 'Start voice search'}
        aria-label={isListening ? 'Stop voice search' : 'Start voice search'}
        aria-pressed={isListening}
      >
        {isListening ? <MicActiveIcon /> : <MicIcon />}
        {isListening && <PulseRing />}
      </button>

      {/* Interim transcript display */}
      {interimTranscript && (
        <div className="voice-transcript" role="status" aria-live="polite">
          <span className="transcript-text">{interimTranscript}</span>
          <span className="transcript-dots">...</span>
        </div>
      )}

      <style jsx>{`
        .voice-search-container {
          position: relative;
          display: inline-flex;
          align-items: center;
          gap: 0.5rem;
        }

        .voice-search-btn {
          position: relative;
          width: 48px;
          height: 48px;
          border-radius: 50%;
          border: 2px solid #374151;
          background: #1f2937;
          color: #9ca3af;
          cursor: pointer;
          display: flex;
          align-items: center;
          justify-content: center;
          transition: all 0.2s ease;
        }

        .voice-search-btn:hover:not(:disabled) {
          border-color: #4b5563;
          color: #e5e7eb;
          background: #374151;
        }

        .voice-search-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .voice-search-btn.listening {
          border-color: #ef4444;
          background: rgba(239, 68, 68, 0.1);
          color: #ef4444;
        }

        .voice-search-btn.unsupported,
        .voice-search-btn.permission-denied {
          border-color: #4b5563;
          color: #6b7280;
        }

        .voice-transcript {
          position: absolute;
          left: 100%;
          margin-left: 0.75rem;
          padding: 0.5rem 0.75rem;
          background: #1f2937;
          border: 1px solid #374151;
          border-radius: 0.5rem;
          white-space: nowrap;
          font-size: 0.875rem;
          color: #e5e7eb;
          box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
        }

        .transcript-text {
          color: #60a5fa;
        }

        .transcript-dots {
          animation: blink 1s infinite;
        }

        @keyframes blink {
          0%, 50% { opacity: 1; }
          51%, 100% { opacity: 0; }
        }
      `}</style>
    </div>
  );
}

// Icon components
function MicIcon() {
  return (
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
      <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
      <line x1="12" x2="12" y1="19" y2="22" />
    </svg>
  );
}

function MicActiveIcon() {
  return (
    <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor" stroke="currentColor" strokeWidth="0">
      <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
      <path d="M19 10v2a7 7 0 0 1-14 0v-2" fill="none" stroke="currentColor" strokeWidth="2" />
      <line x1="12" x2="12" y1="19" y2="22" stroke="currentColor" strokeWidth="2" />
    </svg>
  );
}

function MicOffIcon() {
  return (
    <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <line x1="2" x2="22" y1="2" y2="22" />
      <path d="M18.89 13.23A7.12 7.12 0 0 0 19 12v-2" />
      <path d="M5 10v2a7 7 0 0 0 12 5" />
      <path d="M15 9.34V5a3 3 0 0 0-5.68-1.33" />
      <path d="M9 9v3a3 3 0 0 0 5.12 2.12" />
      <line x1="12" x2="12" y1="19" y2="22" />
    </svg>
  );
}

function PulseRing() {
  return (
    <>
      <span className="pulse-ring" />
      <span className="pulse-ring delay" />
      <style jsx>{`
        .pulse-ring {
          position: absolute;
          width: 100%;
          height: 100%;
          border-radius: 50%;
          border: 2px solid #ef4444;
          animation: pulse 1.5s ease-out infinite;
          pointer-events: none;
        }

        .pulse-ring.delay {
          animation-delay: 0.5s;
        }

        @keyframes pulse {
          0% {
            transform: scale(1);
            opacity: 0.8;
          }
          100% {
            transform: scale(1.8);
            opacity: 0;
          }
        }
      `}</style>
    </>
  );
}

export default VoiceSearch;
