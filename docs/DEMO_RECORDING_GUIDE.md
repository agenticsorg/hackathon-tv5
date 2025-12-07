# Demo Video Recording Guide

Complete step-by-step guide for recording the 2-minute EntertainAI hackathon demo video.

---

## 📋 Pre-Recording Checklist

### ✅ System Setup

- [ ] **Python backend running** on port 8000
  ```bash
  cd api
  python3 main.py
  # Should show: 🚀 Starting EntertainAI API Server...
  ```

- [ ] **Next.js UI running** on port 3000
  ```bash
  cd web-ui
  npm run dev
  # Should show: ▲ Next.js 15.5.7 - Local: http://localhost:3000
  ```

- [ ] **Health check passes**
  ```bash
  curl http://localhost:8000/health
  # Should return: {"status":"healthy","service":"EntertainAI API"}
  ```

- [ ] **Browser clean state**
  - Open Chrome/Firefox in incognito mode
  - Navigate to http://localhost:3000
  - Verify no yellow warning banner appears
  - Check browser console for errors (should be clean)

### ✅ Recording Software Setup

**Option 1: OBS Studio (Recommended)**
- [ ] OBS Studio installed (https://obsproject.com/)
- [ ] Screen capture source added
- [ ] Audio sources configured (system audio + microphone)
- [ ] Recording settings: 1080p, 30fps, MP4 format
- [ ] Output folder set to a known location

**Option 2: macOS QuickTime**
- [ ] QuickTime Player open
- [ ] File → New Screen Recording
- [ ] Options → Show Mouse Clicks enabled
- [ ] Microphone selected

**Option 3: Windows Game Bar**
- [ ] Windows Key + G to open
- [ ] Capture settings: 1080p, 30fps
- [ ] Audio settings verified

### ✅ Browser Setup

- [ ] Browser window at 1920x1080 resolution
- [ ] Zoom level at 100%
- [ ] Bookmarks bar hidden
- [ ] Extensions disabled (or incognito mode)
- [ ] Dark mode enabled (matches EntertainAI theme)
- [ ] Browser console open (F12) for "live logs" shot
- [ ] Network tab cleared

### ✅ Demo Data Prepared

Have these search queries ready to type:
1. **Main demo**: "action movies with strong female leads"
2. **Backup 1**: "sci-fi thriller for movie night"
3. **Backup 2**: "feel-good comedy series"

### ✅ Script Ready

- [ ] docs/DEMO_VIDEO_SCRIPT.md printed or visible on second monitor
- [ ] Voiceover script rehearsed 2-3 times
- [ ] Timing tested (should be ~2:00 minutes)
- [ ] Key talking points memorized

---

## 🎬 Recording Workflow

### Phase 1: Pre-Recording Test (5 minutes)

1. **Test full flow**
   ```bash
   # Terminal 1: Start Python backend
   cd api && python3 main.py

   # Terminal 2: Start Next.js UI
   cd web-ui && npm run dev
   ```

2. **Test search**
   - Open http://localhost:3000
   - Type "action movies with strong female leads"
   - Click search
   - Wait for results (should appear in ~3 seconds)
   - Verify agent activity animation plays
   - Check browser console for logs

3. **Verify output**
   - Should see 8 agents processing
   - Should see results grid appear
   - Should NOT see yellow warning banner
   - Browser console should show: `✅ Live recommendations from Python agents`

### Phase 2: Screen Recording (10 minutes)

**Start Recording:**

1. **Launch recording software**
   - OBS: Click "Start Recording"
   - QuickTime: Click red record button
   - Windows: Win+Alt+R

2. **Record opening sequence** (0:00-0:10)
   - Show EntertainAI homepage
   - Pause for 3 seconds to let intro load
   - Begin voiceover: *"What if you could find the perfect thing to watch in 6 seconds instead of 45 minutes?"*

3. **Record strategic questions** (0:10-0:40)
   - Show the 3-question UI
   - Highlight each question as you mention it
   - No need to interact - just visually show them

4. **Record search interaction** (0:40-0:55)
   - Type query slowly (readable on screen)
   - Click search button
   - Begin agent activity visualization

5. **Record agent activity** (0:55-1:15)
   - Let the 8-agent animation play completely
   - Voiceover highlights 3 on-device agents
   - Zoom in slightly on agent visualization if possible

6. **Record results** (1:15-1:35)
   - Show results grid appearing
   - Briefly hover over one recommendation card
   - Highlight confidence score and reasoning

7. **Record privacy feature** (1:35-1:50)
   - Quick cut to browser console showing logs
   - Show "Live recommendations from Python agents" message
   - Highlight execution time

8. **Record closing** (1:50-2:00)
   - Return to full UI view
   - Show final tagline on screen
   - Fade to black (or cut)

**Stop Recording:**
- OBS: Click "Stop Recording"
- QuickTime: Command+Control+Escape
- Windows: Win+Alt+R

### Phase 3: Voiceover Recording (15 minutes)

**If doing voiceover separately:**

1. **Set up microphone**
   - Quiet room, minimal background noise
   - Microphone 6-12 inches from mouth
   - Test recording 5 seconds, play back

2. **Record voiceover**
   - Use Audacity (free) or GarageBand (macOS)
   - Read from docs/DEMO_VIDEO_SCRIPT.md
   - Speak clearly, moderate pace (~150 words/minute)
   - Leave 0.5s pause between sentences

3. **Export audio**
   - Export as WAV or MP4 (44.1kHz, 16-bit)
   - Save to same folder as video

### Phase 4: Video Editing (20 minutes)

**Option 1: Quick Edit (No editing software)**
- If recording with voiceover live, skip this
- Just trim start/end in QuickTime or Windows Photos

**Option 2: Professional Edit (iMovie/DaVinci Resolve)**

1. **Import assets**
   - Screen recording video
   - Voiceover audio (if separate)
   - Optional: Background music (very low volume)

2. **Sync voiceover**
   - Align voiceover with visual actions
   - Use timestamps from DEMO_VIDEO_SCRIPT.md

3. **Add text overlays** (Optional but recommended)
   - 0:05: "EntertainAI - Privacy-First Discovery"
   - 0:40: "Natural Language Search"
   - 1:00: "8 AI Agents Collaborating"
   - 1:15: "3 Agents Run On-Device"
   - 1:55: "Zero Personal Data Sent to Server"

4. **Export video**
   - Format: MP4 (H.264 codec)
   - Resolution: 1920x1080
   - Framerate: 30fps
   - Bitrate: 8-10 Mbps
   - Output filename: `EntertainAI_Demo_2min.mp4`

---

## 🎯 Shot-by-Shot Reference

### Shot 1: Opening (0:00-0:10)
**Screen:** EntertainAI homepage with search bar
**Voiceover:** *"What if you could find the perfect thing to watch in 6 seconds instead of 45 minutes? Meet EntertainAI."*

**Camera Action:**
- Static shot of full UI
- No interaction
- Let visuals breathe

### Shot 2: Problem Statement (0:10-0:25)
**Screen:** Zoom slightly on search interface
**Voiceover:** *"Traditional algorithms only use your watch history. We use strategic questions to understand your unique situation."*

**Camera Action:**
- Highlight the 3-question UI (Who's Watching, Energy Level, Duration)
- Subtle zoom or pan

### Shot 3: Quick Questions (0:25-0:40)
**Screen:** Show each question UI element
**Voiceover:** *"Who's watching? What's your energy level? How much time do you have? Three questions in six seconds."*

**Camera Action:**
- Quick cuts between each question (2-3 seconds each)
- Or slow pan across all three

### Shot 4: Search Demo (0:40-0:55)
**Screen:** Type query and click search
**Voiceover:** *"Let's try it. 'Action movies with strong female leads.'"*

**Camera Action:**
- Type query slowly (readable)
- Click search button
- Agent activity animation begins

### Shot 5: Agent Visualization (0:55-1:15)
**Screen:** 8-agent activity panel
**Voiceover:** *"Watch our eight agents collaborate in real-time. The StrategicContextAgent understands your query. ResearchAgent finds candidates. Three agents—PersonalizationAgent, MoodDetectionAgent, and AnalysisAgent—run entirely on your device for privacy."*

**Camera Action:**
- Let animation play completely
- Zoom in slightly on agent list
- Highlight "on-device" labels

### Shot 6: Results (1:15-1:35)
**Screen:** Results grid appearing
**Voiceover:** *"In under three seconds, we deliver personalized recommendations with confidence scores and reasoning you can actually understand."*

**Camera Action:**
- Show results appearing
- Hover over one card to show details
- Highlight confidence score ("Very High")

### Shot 7: Privacy (1:35-1:50)
**Screen:** Browser console with live logs
**Voiceover:** *"TikTok, Netflix, and YouTube store your complete watch history on their servers. We store ZERO personal data. Your watch history, your preferences, your privacy—all stay on your device."*

**Camera Action:**
- Quick cut to browser console
- Show "Live recommendations from Python agents" log
- Highlight execution time

### Shot 8: Closing (1:50-2:00)
**Screen:** Return to full UI with results
**Voiceover:** *"EntertainAI. Privacy-first entertainment discovery powered by multi-agent AI. The future of finding what to watch is here."*

**Camera Action:**
- Slow zoom out
- Fade to black (optional)
- Show text: "EntertainAI - Agentics Foundation TV5 Hackathon"

---

## 🔧 Troubleshooting

### Problem: Yellow Warning Banner Appears

**Symptoms:**
- UI shows: "⚠️ Using mock data. Python backend not available."

**Fix:**
```bash
# Check if Python backend is running
curl http://localhost:8000/health

# If not running, start it:
cd api
python3 main.py

# Wait for: 🚀 Starting EntertainAI API Server...
# Then refresh browser
```

### Problem: Agent Activity Doesn't Show

**Symptoms:**
- Search completes but no agent visualization appears

**Fix:**
- This is expected! The agent activity component only shows DURING the search
- It disappears when results appear
- To capture it: Start recording BEFORE clicking search
- Or slow down the animation in `web-ui/components/AgentActivity.tsx` (line 45: change 2800ms to 6000ms)

### Problem: Results Load Too Quickly

**Symptoms:**
- Agent activity disappears in <1 second
- Can't capture the animation

**Fix Option 1: Add Artificial Delay**
```typescript
// In web-ui/app/page.tsx, line 52
setUseMockData(false);
setRecommendations(data.recommendations || []);

// Add this before line 68:
await new Promise(resolve => setTimeout(resolve, 3000)); // Add 3s delay
```

**Fix Option 2: Use Mock Data Mode**
- Comment out Python backend health check in `.env.local`
- This forces UI to use mock data with intentional delay
- Still shows agent activity animation

### Problem: Audio Quality Poor

**Symptoms:**
- Voiceover has background noise, echo, or is too quiet

**Fix:**
1. **Background Noise:**
   - Record in quiet room
   - Close windows, turn off AC/fans
   - Use noise cancellation in Audacity (Effect → Noise Reduction)

2. **Echo:**
   - Move closer to microphone (6-8 inches)
   - Record in room with soft surfaces (curtains, carpet)

3. **Too Quiet:**
   - Increase microphone gain (not too much, causes distortion)
   - Use Audacity's Amplify effect after recording
   - Normalize to -3dB

### Problem: Video Resolution Wrong

**Symptoms:**
- Video appears blurry or pixelated
- Aspect ratio is wrong (black bars)

**Fix:**
1. **Set browser window to exact size:**
   ```javascript
   // Paste in browser console:
   window.resizeTo(1920, 1080);
   ```

2. **OBS Settings:**
   - Settings → Video → Base Resolution: 1920x1080
   - Output Resolution: 1920x1080
   - Downscale Filter: Lanczos

3. **Recording area:**
   - Capture only the browser window, not entire screen
   - Use OBS Window Capture instead of Display Capture

### Problem: Mouse Cursor Not Visible

**Symptoms:**
- Can't see where user is clicking

**Fix:**
- **OBS**: Settings → Output → Show Cursor (enable)
- **QuickTime**: Options → Show Mouse Clicks in Recording
- **Windows**: Game Bar captures cursor by default

---

## 📦 Alternative Recording Approaches

### Approach 1: Live Voiceover (Easiest)
**Pros:** Single take, no editing needed
**Cons:** Harder to sync perfectly

**Workflow:**
1. Start screen recording with microphone enabled
2. Read voiceover script while performing actions
3. One take, done in 2-3 minutes
4. No editing required

### Approach 2: Separate Recording + Editing (Best Quality)
**Pros:** Perfect sync, professional quality
**Cons:** Requires editing software

**Workflow:**
1. Record screen actions silently (no voiceover)
2. Record voiceover separately in quiet room
3. Edit together in iMovie/DaVinci Resolve
4. Add text overlays and transitions

### Approach 3: Slide Deck + Screen Recording Hybrid
**Pros:** Easy to control timing
**Cons:** Less engaging

**Workflow:**
1. Create 5-6 slides with key points
2. Screen record EntertainAI demo
3. Intercut slides with live demo footage
4. Add voiceover in editing

---

## 🎬 Post-Production Checklist

After recording:

- [ ] **Watch full video** - Check for errors, awkward pauses
- [ ] **Verify audio** - Clear, no distortion, consistent volume
- [ ] **Check timing** - Should be 1:50-2:10 (aim for 2:00)
- [ ] **Test playback** - Works on multiple devices/browsers
- [ ] **Export settings** - MP4, 1920x1080, 30fps, 8-10 Mbps
- [ ] **File size** - Should be 40-80 MB for 2 minutes
- [ ] **Filename** - `EntertainAI_Demo_2min.mp4`

---

## 📤 Distribution Preparation

### File Formats

**Primary Version:**
- Format: MP4 (H.264)
- Resolution: 1920x1080
- Framerate: 30fps
- Bitrate: 8-10 Mbps
- Filename: `EntertainAI_Demo_2min.mp4`

**Social Media Versions** (Optional):

1. **LinkedIn** (1:1 square, 30s version)
   - Resolution: 1080x1080
   - Duration: 30 seconds
   - Filename: `EntertainAI_LinkedIn_30s.mp4`

2. **Twitter/X** (16:9, 1min version)
   - Resolution: 1280x720
   - Duration: 60 seconds
   - Filename: `EntertainAI_Twitter_1min.mp4`

3. **YouTube** (full 2min version)
   - Same as primary
   - Add thumbnail image (1280x720 JPG)
   - Add description with links

### Upload Locations

- [ ] **Hackathon submission portal** (check requirements)
- [ ] **YouTube** (unlisted or public)
- [ ] **LinkedIn** (your profile or company page)
- [ ] **Twitter/X** (thread with screenshots)
- [ ] **GitHub** (link in README.md)
- [ ] **Discord** (Agentics Foundation community)

### Video Description Template

```markdown
# EntertainAI - Privacy-First Entertainment Discovery

🎬 **What:** AI-powered entertainment discovery that respects your privacy
🔒 **Privacy:** 90-95% better than TikTok, Netflix, or YouTube
🤖 **Technology:** 8-agent system with 3 on-device agents
⚡ **Speed:** Recommendations in under 3 seconds
🏆 **Built for:** Agentics Foundation TV5 Hackathon

## Key Features:
- Natural language search
- Strategic context questions (6 seconds)
- Multi-agent collaboration (8 agents)
- On-device personalization (zero data sent to servers)
- Real-time agent visualization

## Links:
- GitHub: [link to repo]
- Live Demo: [if deployed]
- Hackathon: https://agentics.org/hackathon

## Tech Stack:
- Next.js 15 + React 19 + TypeScript
- FastAPI + Python asyncio
- Google ADK + Vertex AI
- 8-agent parallel execution system

#AI #Privacy #Hackathon #AgenticAI #EntertainmentDiscovery
```

---

## 🎯 Final Tips

### Recording Best Practices

1. **Practice 2-3 times** before recording
   - Know exactly what to click and when
   - Rehearse voiceover with a timer
   - Identify potential stumbling points

2. **Keep it simple**
   - One take is better than heavily edited
   - Natural voice is better than overly scripted
   - Show confidence in the product

3. **Focus on value**
   - Highlight privacy as THE differentiator
   - Show the agent visualization (unique visual)
   - Demonstrate speed (under 3 seconds)

4. **Have backups**
   - Record 2-3 takes if possible
   - Keep raw footage until final is approved
   - Save project file if using editing software

### Common Mistakes to Avoid

❌ **Don't:**
- Rush through the script (speak clearly)
- Forget to show agent activity (key visual)
- Skip the privacy message (main differentiator)
- Use filler words ("um", "uh", "so", "like")
- Record with system notifications enabled

✅ **Do:**
- Speak at moderate pace (~150 words/minute)
- Pause briefly between sentences
- Show personality and enthusiasm
- Emphasize key numbers (6 seconds, 3 agents, 90-95% better)
- Test everything before hitting record

---

## 🆘 Last-Minute Issues

If something goes wrong 5 minutes before recording:

### Python Backend Won't Start
**Quick Fix:**
```bash
# Kill any process on port 8000
lsof -ti:8000 | xargs kill -9

# Restart
cd api && python3 main.py
```

### Next.js Won't Start
**Quick Fix:**
```bash
# Kill any process on port 3000
lsof -ti:3000 | xargs kill -9

# Clear cache and restart
cd web-ui
rm -rf .next
npm run dev
```

### Results Not Showing
**Quick Fix:**
- Use mock data mode (acceptable for demo)
- Shows same agent activity animation
- Still demonstrates the concept

### Audio Not Working
**Quick Fix:**
- Skip voiceover, add text overlays instead
- Use subtitle track
- Add voiceover in post-production

---

## 📊 Success Criteria

Your demo is ready to submit when:

- ✅ Video is 1:50-2:10 minutes long
- ✅ Audio is clear with no background noise
- ✅ Shows full search flow (query → agents → results)
- ✅ Agent activity visualization is visible
- ✅ Privacy message is clearly communicated
- ✅ Resolution is 1920x1080 at 30fps
- ✅ File size is reasonable (40-80 MB)
- ✅ Works on multiple devices/platforms

---

**Ready to record?**

1. Run through pre-recording checklist
2. Do one practice run
3. Hit record
4. Ship it! 🚀

**Good luck! You've got this! 🎬**

---

**Document Version:** 1.0
**Last Updated:** 2024-12-06
**Status:** Production-Ready Recording Guide
