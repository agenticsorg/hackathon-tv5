# Demo Recording Quick Reference Card

**Print this or keep on second monitor during recording**

---

## ⚡ Pre-Recording Checklist (2 minutes)

```bash
# Terminal 1: Start Python backend
cd api && python3 main.py
# ✅ Wait for: 🚀 Starting EntertainAI API Server...

# Terminal 2: Start Next.js UI
cd web-ui && npm run dev
# ✅ Wait for: ▲ Next.js 15.5.7 - Local: http://localhost:3000

# Test health
curl http://localhost:8000/health
# ✅ Should return: {"status":"healthy"}
```

**Browser Setup:**
- Open http://localhost:3000 in incognito mode
- Verify NO yellow warning banner
- F12 → Check console for errors (should be clean)
- Zoom: 100%

---

## 🎬 Shot List (2:00 minutes)

| Time | Shot | Screen | Voiceover (Key Words) |
|------|------|--------|----------------------|
| **0:00-0:10** | Opening | Homepage | "6 seconds instead of 45 minutes" |
| **0:10-0:25** | Problem | Search UI | "Traditional algorithms only use watch history" |
| **0:25-0:40** | Questions | 3-question UI | "Who's watching? Energy level? Time?" |
| **0:40-0:55** | Search | Type + click | "Action movies with strong female leads" |
| **0:55-1:15** | Agents | 8-agent panel | "Three run entirely on your device" |
| **1:15-1:35** | Results | Results grid | "Under three seconds, confidence scores" |
| **1:35-1:50** | Privacy | Browser console | "We store ZERO personal data" |
| **1:50-2:00** | Closing | Full UI | "Privacy-first entertainment discovery" |

---

## 🎯 Action Sequence

### Setup (Before Recording)
1. ✅ Both terminals running (Python + Next.js)
2. ✅ Browser open to http://localhost:3000
3. ✅ Recording software ready (OBS/QuickTime)
4. ✅ Microphone tested
5. ✅ Browser console closed (will open later)

### Recording Flow

**START RECORDING →**

**0:00** - Show homepage (3 seconds static)

**0:10** - Begin voiceover, show search interface

**0:25** - Highlight 3-question UI (pan or zoom)

**0:40** - Type: `action movies with strong female leads`

**0:45** - Click search button

**0:50** - Agent activity animation begins (DO NOT INTERRUPT)

**1:15** - Results appear automatically

**1:20** - Hover over one recommendation card

**1:35** - Press F12 to show console

**1:40** - Point to "Live recommendations" log

**1:50** - Close console, show full UI

**2:00** - **STOP RECORDING**

---

## 🎤 Voiceover Script (Condensed)

**Opening (0:00-0:10):**
> "What if you could find the perfect thing to watch in 6 seconds instead of 45 minutes? Meet EntertainAI."

**Problem (0:10-0:25):**
> "Traditional algorithms only use your watch history. We use strategic questions to understand your unique situation."

**Questions (0:25-0:40):**
> "Who's watching? What's your energy level? How much time do you have? Three questions in six seconds."

**Search (0:40-0:55):**
> "Let's try it. 'Action movies with strong female leads.'"

**Agents (0:55-1:15):**
> "Watch our eight agents collaborate in real-time. The StrategicContextAgent understands your query. ResearchAgent finds candidates. Three agents—PersonalizationAgent, MoodDetectionAgent, and AnalysisAgent—run entirely on your device for privacy."

**Results (1:15-1:35):**
> "In under three seconds, we deliver personalized recommendations with confidence scores and reasoning you can actually understand."

**Privacy (1:35-1:50):**
> "TikTok, Netflix, and YouTube store your complete watch history on their servers. We store ZERO personal data. Your watch history, your preferences, your privacy—all stay on your device."

**Closing (1:50-2:00):**
> "EntertainAI. Privacy-first entertainment discovery powered by multi-agent AI. The future of finding what to watch is here."

---

## 🔧 Emergency Fixes

### Yellow Warning Banner Appears
```bash
cd api && python3 main.py
# Wait 5 seconds, refresh browser
```

### Agent Activity Too Fast
```typescript
// web-ui/components/AgentActivity.tsx line 45
// Change: 2800 → 6000 (slows animation)
```

### Results Not Showing
```bash
# Check browser console for error
# Restart both servers
```

### Audio Distorted
- Move 2 inches further from microphone
- Reduce microphone gain by 20%
- Re-record that section

---

## 📊 Key Numbers to Emphasize

- **6 seconds** - Time to answer strategic questions
- **45 minutes** - Average time people spend deciding
- **3 seconds** - Time to get recommendations
- **8 agents** - Total agents in the system
- **3 agents** - On-device (privacy-preserving)
- **90-95%** - Better privacy than TikTok/Netflix/YouTube
- **ZERO** - Personal data sent to servers

---

## ✅ Recording Settings

**OBS Studio:**
- Output: 1920x1080, 30fps, MP4
- Encoder: x264
- Bitrate: 8000 Kbps
- Audio: 192 Kbps AAC

**QuickTime:**
- File → New Screen Recording
- Options → Show Mouse Clicks
- High Quality selected

**Windows Game Bar:**
- Settings → Capturing → 1080p, 30fps
- Audio quality: High

---

## 🎯 Success Checklist

Before you stop recording, verify:

- ✅ Agent activity visualization was visible
- ✅ Results appeared and were readable
- ✅ Privacy message was clear
- ✅ No error messages appeared
- ✅ Total time was 1:50-2:10
- ✅ Audio was clear throughout
- ✅ Mouse clicks were visible

If any are ❌, consider re-recording.

---

## 📤 After Recording

1. **Watch the full video** (2 min)
2. **Check audio** (clear, no distortion)
3. **Verify timing** (under 2:10)
4. **Export** → MP4, 1920x1080, 30fps
5. **Name file** → `EntertainAI_Demo_2min.mp4`
6. **Upload** to submission portal

---

## 🆘 Something Went Wrong?

**Don't panic!** You have options:

1. **Take 2**: Just restart and try again
2. **Edit it**: Use iMovie/DaVinci to fix small issues
3. **Different approach**: Use slides + demo hybrid
4. **Ask for help**: Check docs/DEMO_RECORDING_GUIDE.md

---

**YOU'VE GOT THIS! 🎬**

---

**Tips:**
- Speak clearly, moderate pace
- Don't rush through the agent activity
- Emphasize privacy (our differentiator)
- Show confidence in the product
- Have fun! Your enthusiasm shows

---

**Print this page or keep it visible while recording.**
