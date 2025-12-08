/**
 * VoiceSearch Component Tests
 *
 * Tests the voice search state management fixes including:
 * - Proper state reset after voice queries
 * - Loading state management in SearchBar
 * - Error recovery
 * - Race condition handling
 * - Timeout handling
 */

import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { VoiceSearch } from '../components/VoiceSearch';
import { SearchBar } from '../components/SearchBar';

// Mock Next.js router
jest.mock('next/navigation', () => ({
  useRouter: () => ({
    push: jest.fn(),
  }),
  useSearchParams: () => ({
    get: jest.fn(() => ''),
  }),
}));

// Mock Web Speech API
class MockSpeechRecognition {
  continuous = false;
  interimResults = false;
  lang = 'en-US';
  onstart: (() => void) | null = null;
  onresult: ((event: any) => void) | null = null;
  onerror: ((event: any) => void) | null = null;
  onend: (() => void) | null = null;

  start() {
    if (this.onstart) {
      this.onstart();
    }
  }

  stop() {
    if (this.onend) {
      this.onend();
    }
  }

  abort() {
    if (this.onend) {
      this.onend();
    }
  }

  // Helper methods for testing
  simulateResult(transcript: string, isFinal: boolean = true) {
    if (this.onresult) {
      this.onresult({
        resultIndex: 0,
        results: [
          {
            isFinal,
            0: { transcript, confidence: 0.9 },
            length: 1,
            item: (index: number) => ({ transcript, confidence: 0.9 }),
          },
        ],
        length: 1,
      });
    }
  }

  simulateError(error: string) {
    if (this.onerror) {
      this.onerror({ error });
    }
  }
}

describe('VoiceSearch Component', () => {
  let mockRecognition: MockSpeechRecognition;

  beforeEach(() => {
    mockRecognition = new MockSpeechRecognition();
    (window as any).SpeechRecognition = jest.fn(() => mockRecognition);
    (window as any).webkitSpeechRecognition = jest.fn(() => mockRecognition);
  });

  afterEach(() => {
    delete (window as any).SpeechRecognition;
    delete (window as any).webkitSpeechRecognition;
  });

  describe('State Management', () => {
    test('resets to ready state after successful voice query', async () => {
      const onResult = jest.fn();
      const onListeningChange = jest.fn();

      const { container } = render(
        <VoiceSearch
          onResult={onResult}
          onListeningChange={onListeningChange}
        />
      );

      const button = container.querySelector('.voice-search-btn');
      expect(button).not.toBeNull();

      // Start listening
      fireEvent.click(button!);
      await waitFor(() => {
        expect(onListeningChange).toHaveBeenCalledWith(true);
      });

      // Simulate successful result
      mockRecognition.simulateResult('action movies');

      await waitFor(() => {
        expect(onResult).toHaveBeenCalledWith('action movies');
        expect(onListeningChange).toHaveBeenCalledWith(false);
      });

      // Button should be ready for next use
      expect(button).not.toHaveClass('listening');
      expect(button).not.toBeDisabled();
    });

    test('handles multiple consecutive voice queries', async () => {
      const onResult = jest.fn();
      const { container } = render(
        <VoiceSearch onResult={onResult} />
      );

      const button = container.querySelector('.voice-search-btn');

      // First query
      fireEvent.click(button!);
      mockRecognition.simulateResult('comedy movies');
      await waitFor(() => expect(onResult).toHaveBeenCalledWith('comedy movies'));

      // Second query (this was failing before the fix)
      fireEvent.click(button!);
      mockRecognition.simulateResult('horror movies');
      await waitFor(() => expect(onResult).toHaveBeenCalledWith('horror movies'));

      // Third query
      fireEvent.click(button!);
      mockRecognition.simulateResult('sci-fi movies');
      await waitFor(() => expect(onResult).toHaveBeenCalledWith('sci-fi movies'));

      expect(onResult).toHaveBeenCalledTimes(3);
    });

    test('resets state on error', async () => {
      const onError = jest.fn();
      const onListeningChange = jest.fn();

      const { container } = render(
        <VoiceSearch
          onResult={jest.fn()}
          onError={onError}
          onListeningChange={onListeningChange}
        />
      );

      const button = container.querySelector('.voice-search-btn');

      fireEvent.click(button!);
      mockRecognition.simulateError('no-speech');

      await waitFor(() => {
        expect(onError).toHaveBeenCalled();
        expect(onListeningChange).toHaveBeenCalledWith(false);
      });

      // Should be ready for next attempt
      expect(button).not.toBeDisabled();
    });

    test('prevents race condition when clicking rapidly', async () => {
      const onResult = jest.fn();
      const { container } = render(
        <VoiceSearch onResult={onResult} />
      );

      const button = container.querySelector('.voice-search-btn');

      // Rapid clicks
      fireEvent.click(button!);
      fireEvent.click(button!);
      fireEvent.click(button!);

      // Should only start once
      expect(mockRecognition.onstart).toHaveBeenCalledTimes(1);
    });
  });

  describe('SearchBar Integration', () => {
    test('resets loading state after navigation', async () => {
      const { rerender } = render(<SearchBar />);

      // Simulate navigation completing (searchParams change)
      const mockSearchParams = {
        get: jest.fn(() => 'action'),
      };

      jest.spyOn(require('next/navigation'), 'useSearchParams').mockReturnValue(mockSearchParams);

      rerender(<SearchBar />);

      // Loading state should be false after navigation
      await waitFor(() => {
        const voiceButton = screen.getByLabelText(/voice search/i);
        expect(voiceButton).not.toBeDisabled();
      });
    });

    test('handles voice search error without permanently disabling', async () => {
      render(<SearchBar />);

      const voiceButton = screen.getByLabelText(/voice search/i);

      fireEvent.click(voiceButton);
      mockRecognition.simulateError('no-speech');

      await waitFor(() => {
        expect(voiceButton).not.toBeDisabled();
      });

      // Should be able to try again
      fireEvent.click(voiceButton);
      expect(voiceButton).not.toBeDisabled();
    });

    test('prevents duplicate submissions during loading', async () => {
      const mockPush = jest.fn();
      jest.spyOn(require('next/navigation'), 'useRouter').mockReturnValue({
        push: mockPush,
      });

      render(<SearchBar />);

      const voiceButton = screen.getByLabelText(/voice search/i);

      // First voice query
      fireEvent.click(voiceButton);
      mockRecognition.simulateResult('thriller movies');

      await waitFor(() => {
        expect(mockPush).toHaveBeenCalledTimes(1);
      });

      // Try to submit again while still loading (should be prevented)
      fireEvent.click(voiceButton);
      mockRecognition.simulateResult('drama movies');

      // Should still only have one navigation call
      expect(mockPush).toHaveBeenCalledTimes(1);
    });
  });

  describe('Edge Cases', () => {
    test('handles disabled prop changes', async () => {
      const { rerender, container } = render(
        <VoiceSearch onResult={jest.fn()} disabled={false} />
      );

      const button = container.querySelector('.voice-search-btn');

      // Start listening
      fireEvent.click(button!);
      await waitFor(() => expect(button).toHaveClass('listening'));

      // Disable while listening
      rerender(<VoiceSearch onResult={jest.fn()} disabled={true} />);

      // Should stop listening
      await waitFor(() => {
        expect(button).toBeDisabled();
        expect(button).not.toHaveClass('listening');
      });
    });

    test('cleans up timeout on manual stop', async () => {
      jest.useFakeTimers();

      const { container } = render(
        <VoiceSearch onResult={jest.fn()} />
      );

      const button = container.querySelector('.voice-search-btn');

      // Start listening
      fireEvent.click(button!);

      // Manually stop before timeout
      fireEvent.click(button!);

      // Fast-forward past timeout
      jest.advanceTimersByTime(31000);

      // Should not trigger timeout error since we stopped manually
      expect(button).not.toHaveClass('listening');

      jest.useRealTimers();
    });

    test('handles timeout after 30 seconds', async () => {
      jest.useFakeTimers();

      const onError = jest.fn();
      const { container } = render(
        <VoiceSearch onResult={jest.fn()} onError={onError} />
      );

      const button = container.querySelector('.voice-search-btn');

      fireEvent.click(button!);

      // Fast-forward 30 seconds
      jest.advanceTimersByTime(30000);

      await waitFor(() => {
        expect(onError).toHaveBeenCalledWith(expect.stringContaining('timed out'));
        expect(button).not.toHaveClass('listening');
      });

      jest.useRealTimers();
    });
  });

  describe('Accessibility', () => {
    test('provides proper ARIA attributes', () => {
      const { container } = render(
        <VoiceSearch onResult={jest.fn()} />
      );

      const button = container.querySelector('.voice-search-btn');

      expect(button).toHaveAttribute('aria-label');
      expect(button).toHaveAttribute('title');
    });

    test('updates ARIA pressed state when listening', async () => {
      const { container } = render(
        <VoiceSearch onResult={jest.fn()} />
      );

      const button = container.querySelector('.voice-search-btn');

      expect(button).toHaveAttribute('aria-pressed', 'false');

      fireEvent.click(button!);

      await waitFor(() => {
        expect(button).toHaveAttribute('aria-pressed', 'true');
      });
    });
  });
});
