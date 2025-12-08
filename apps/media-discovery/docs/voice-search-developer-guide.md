# Voice Search Developer Guide

## Quick Reference

### Component Hierarchy
```
SearchBar (Parent)
  └── VoiceSearch (Child)
```

### State Management Rules

#### SearchBar Component
```typescript
// ✅ DO: Reset loading state after navigation
useEffect(() => {
  setIsLoading(false);
}, [searchParams]);

// ✅ DO: Add error recovery
const handleVoiceResult = useCallback((transcript: string) => {
  if (isLoading) return; // Prevent duplicates
  setIsLoading(true);
  try {
    router.push(`/search?q=${transcript}`);
  } catch (error) {
    setIsLoading(false); // Reset on error
  }
}, [router, isLoading]);

// ✅ DO: Handle voice errors
const handleVoiceError = useCallback((error: string) => {
  setIsLoading(false); // Reset loading state
}, []);

// ❌ DON'T: Forget to connect error handler
<VoiceSearch
  onError={handleVoiceError} // Required!
  disabled={isLoading}
/>
```

#### VoiceSearch Component
```typescript
// ✅ DO: Guard against race conditions
const startListening = useCallback(() => {
  if (isListening) return; // Prevent re-entry

  // Clear existing timeout
  if (timeoutRef.current) {
    clearTimeout(timeoutRef.current);
    timeoutRef.current = null;
  }

  try {
    recognitionRef.current.start();
  } catch (error) {
    // Reset state on error
    setIsListening(false);
    onListeningChange?.(false);
  }
}, [isListening, onListeningChange]);

// ✅ DO: Clean up timeouts everywhere
const stopListening = useCallback(() => {
  if (timeoutRef.current) {
    clearTimeout(timeoutRef.current);
    timeoutRef.current = null;
  }

  try {
    recognitionRef.current.stop();
  } catch (error) {
    // Force state reset
    setIsListening(false);
    setInterimTranscript('');
    onListeningChange?.(false);
  }
}, [onListeningChange]);

// ✅ DO: Reset ALL related state together
recognition.onerror = (event) => {
  // Clear timeout
  if (timeoutRef.current) {
    clearTimeout(timeoutRef.current);
    timeoutRef.current = null;
  }

  // Reset ALL state
  setIsListening(false);
  setInterimTranscript('');
  onListeningChange?.(false);
};

// ✅ DO: Clean up when disabled changes
useEffect(() => {
  if (disabled && isListening) {
    recognitionRef.current?.stop();
  }
}, [disabled, isListening]);

// ❌ DON'T: Leave state variables out of sync
// ❌ DON'T: Forget to clear timeouts
// ❌ DON'T: Allow multiple concurrent recognitions
```

---

## Common Pitfalls

### 1. Loading State Never Resets
```typescript
// ❌ BAD
const handleVoiceResult = (transcript: string) => {
  setIsLoading(true);
  router.push(`/search?q=${transcript}`);
  // Missing: setIsLoading(false)
};

// ✅ GOOD
useEffect(() => {
  setIsLoading(false); // Reset on navigation
}, [searchParams]);

const handleVoiceResult = (transcript: string) => {
  setIsLoading(true);
  try {
    router.push(`/search?q=${transcript}`);
  } catch (error) {
    setIsLoading(false); // Also reset on error
  }
};
```

### 2. Race Condition on Rapid Clicks
```typescript
// ❌ BAD
const startListening = () => {
  recognitionRef.current.start(); // Can fail if already started
};

// ✅ GOOD
const startListening = () => {
  if (isListening) return; // Guard
  recognitionRef.current.start();
};
```

### 3. Timeout Not Cleared
```typescript
// ❌ BAD
const startListening = () => {
  recognitionRef.current.start();
  timeoutRef.current = setTimeout(() => {
    recognitionRef.current.stop();
  }, 30000);
  // Timeout never cleared if stopped manually
};

// ✅ GOOD
const startListening = () => {
  if (timeoutRef.current) {
    clearTimeout(timeoutRef.current);
  }

  recognitionRef.current.start();
  timeoutRef.current = setTimeout(() => {
    recognitionRef.current.stop();
  }, 30000);
};

const stopListening = () => {
  if (timeoutRef.current) {
    clearTimeout(timeoutRef.current);
    timeoutRef.current = null;
  }
  recognitionRef.current.stop();
};
```

### 4. Incomplete State Reset
```typescript
// ❌ BAD
recognition.onerror = (event) => {
  setIsListening(false);
  // Missing: interimTranscript, timeout cleanup
};

// ✅ GOOD
recognition.onerror = (event) => {
  if (timeoutRef.current) {
    clearTimeout(timeoutRef.current);
    timeoutRef.current = null;
  }

  setIsListening(false);
  setInterimTranscript('');
  onListeningChange?.(false);
};
```

### 5. No Cleanup on Disabled
```typescript
// ❌ BAD
// No effect to handle disabled prop changes

// ✅ GOOD
useEffect(() => {
  if (disabled && isListening) {
    recognitionRef.current?.stop();
  }
}, [disabled, isListening]);
```

---

## Debugging Checklist

When voice search button gets stuck, check:

- [ ] Is `isLoading` being reset when navigation completes?
  - **Fix**: Add `useEffect(() => setIsLoading(false), [searchParams])`

- [ ] Is `isListening` being reset in all error handlers?
  - **Fix**: Add `setIsListening(false)` to all error paths

- [ ] Are timeouts being cleared in all code paths?
  - **Fix**: Clear `timeoutRef.current` in start, stop, end, error

- [ ] Is `interimTranscript` being cleared?
  - **Fix**: Add `setInterimTranscript('')` to end/error handlers

- [ ] Can recognition start while already active?
  - **Fix**: Add `if (isListening) return` guard

- [ ] Is there error recovery for `router.push`?
  - **Fix**: Wrap in try-catch with `setIsLoading(false)` in catch

- [ ] Is component cleaned up when disabled?
  - **Fix**: Add useEffect watching `disabled` and `isListening`

---

## Testing Guide

### Manual Testing Script
```
1. Open app in browser
2. Click microphone button
3. Speak: "action movies"
4. Verify: Results appear
5. Click microphone button AGAIN
6. Verify: Button is clickable (not spinning/disabled) ✅
7. Speak: "comedy shows"
8. Verify: New results appear
9. Repeat steps 5-8 five more times
10. All attempts should work ✅

Error Testing:
11. Click microphone
12. Don't speak for 5 seconds
13. Verify: "No speech detected" error
14. Click microphone again
15. Verify: Button works ✅

Rapid Click Testing:
16. Click microphone 5 times rapidly
17. Verify: No console errors
18. Speak: "thriller movies"
19. Verify: Works correctly ✅

Timeout Testing:
20. Click microphone
21. Wait 30 seconds without speaking
22. Verify: Timeout error appears
23. Click microphone again
24. Verify: Button works ✅
```

### Automated Testing
```bash
# Run unit tests
npm test -- VoiceSearch.test.tsx

# Expected output:
# ✓ Resets to ready state after successful query
# ✓ Handles multiple consecutive queries
# ✓ Resets state on error
# ✓ Prevents race condition on rapid clicks
# ✓ Resets loading state after navigation
# ✓ Handles voice error without permanent disable
# ✓ Prevents duplicate submissions during loading
# ✓ Handles disabled prop changes
# ✓ Cleans up timeout on manual stop
# ✓ Handles timeout after 30 seconds
```

---

## Architecture Decisions

### Why useEffect for Loading Reset?
```typescript
// Option 1: Reset in handleVoiceResult (❌ Doesn't work)
const handleVoiceResult = (transcript: string) => {
  setIsLoading(true);
  router.push(`/search?q=${transcript}`);
  setIsLoading(false); // ❌ Runs before navigation completes
};

// Option 2: Reset on searchParams change (✅ Works!)
useEffect(() => {
  setIsLoading(false); // ✅ Runs AFTER navigation completes
}, [searchParams]);
```

**Reasoning**: `router.push()` is asynchronous, so setting `isLoading(false)` immediately after doesn't work. We need to wait for the navigation to complete, which is signaled by `searchParams` changing.

### Why Ref for Cleanup Flag?
```typescript
const isCleaningUpRef = useRef(false);

useEffect(() => {
  if (disabled && isListening && !isCleaningUpRef.current) {
    isCleaningUpRef.current = true;
    recognitionRef.current?.stop();
    setTimeout(() => {
      isCleaningUpRef.current = false;
    }, 100);
  }
}, [disabled, isListening]);
```

**Reasoning**: Prevents race condition where effect fires multiple times in rapid succession. The ref acts as a lock to ensure cleanup only runs once per state change.

### Why Clear Timeout Everywhere?
```typescript
// startListening, stopListening, onerror, onend
if (timeoutRef.current) {
  clearTimeout(timeoutRef.current);
  timeoutRef.current = null;
}
```

**Reasoning**: Timeouts can trigger after component unmounts or state changes, causing memory leaks and unexpected behavior. Clearing in all code paths ensures no stale timeouts fire.

---

## Performance Considerations

### Minimal Re-renders
```typescript
// ✅ GOOD: Only re-create when dependencies change
const handleVoiceResult = useCallback((transcript) => {
  // ...
}, [router, isLoading]);

// ❌ BAD: Re-creates on every render
const handleVoiceResult = (transcript) => {
  // ...
};
```

### Cleanup on Unmount
```typescript
useEffect(() => {
  // Setup
  const recognition = new SpeechRecognition();

  // Cleanup
  return () => {
    recognition.abort(); // ✅ Clean up on unmount
  };
}, []);
```

---

## Browser Support

### Web Speech API Support
| Browser | Support | Notes |
|---------|---------|-------|
| Chrome | ✅ Full | `window.webkitSpeechRecognition` |
| Edge | ✅ Full | `window.webkitSpeechRecognition` |
| Safari | ✅ Full | `window.webkitSpeechRecognition` |
| Firefox | ❌ None | Gracefully degrades to disabled state |
| Opera | ✅ Full | `window.webkitSpeechRecognition` |

### Fallback Handling
```typescript
if (!isSupported) {
  return (
    <button disabled title="Voice search not supported">
      <MicOffIcon />
    </button>
  );
}
```

---

## File Structure

```
apps/media-discovery/
├── src/
│   ├── components/
│   │   ├── SearchBar.tsx          # Parent component
│   │   └── VoiceSearch.tsx        # Voice search component
│   └── tests/
│       └── VoiceSearch.test.tsx   # Comprehensive tests
└── docs/
    ├── voice-search-fix.md              # Fix documentation
    ├── voice-search-state-flow.md       # State flow diagrams
    └── voice-search-developer-guide.md  # This file
```

---

## Additional Resources

- [Web Speech API Documentation](https://developer.mozilla.org/en-US/docs/Web/API/Web_Speech_API)
- [React useEffect Hook](https://react.dev/reference/react/useEffect)
- [React useCallback Hook](https://react.dev/reference/react/useCallback)
- [Next.js App Router](https://nextjs.org/docs/app)

---

## Support

If you encounter issues:

1. Check console for errors
2. Run automated tests
3. Verify browser support
4. Review state flow documentation
5. Check this debugging checklist

**Most Common Issue**: Button stuck after first use
**Solution**: Ensure `useEffect(() => setIsLoading(false), [searchParams])` is present in SearchBar

---

## Version History

- **v1.0.0** (Initial) - Basic voice search with state bug
- **v1.1.0** (Current) - Complete state management fix
  - Added loading state reset on navigation
  - Added race condition guards
  - Added comprehensive timeout cleanup
  - Added disabled state cleanup
  - Added error recovery
  - Added comprehensive tests
