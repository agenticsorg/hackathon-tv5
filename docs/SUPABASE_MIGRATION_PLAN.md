# EmotiStream: Supabase-Native Architecture Migration Plan

## Overview

This document outlines the migration from the current Express.js backend with JSON file storage to a fully Supabase-native architecture optimized for deployment on Lovable + Supabase stack.

## Current Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Next.js 15    │────▶│  Express.js     │────▶│  JSON Files     │
│   Frontend      │     │  Backend        │     │  (data/*.json)  │
│   (Port 3000)   │     │  (Port 4000)    │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

**Current Components:**
- Custom JWT authentication (bcrypt + jsonwebtoken)
- In-memory Q-table for RL policy
- JSON file persistence (users.json, feedback.json, qtable.json)
- Express.js API routes

## Target Architecture (Supabase-Native)

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Next.js 15    │────▶│  Supabase       │────▶│  PostgreSQL     │
│   Frontend      │     │  (Auth + DB +   │     │  (Managed)      │
│                 │     │   Edge Funcs)   │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

**Target Components:**
- Supabase Auth (replaces custom JWT)
- Supabase PostgreSQL (replaces JSON files)
- Supabase Edge Functions (for RL computations)
- Row Level Security (RLS) for data isolation
- Real-time subscriptions for live updates

---

## Phase 1: Database Schema Design

### 1.1 Users Table (Extended from Supabase Auth)

```sql
-- Supabase Auth handles the base auth.users table
-- We extend it with a profiles table

CREATE TABLE public.profiles (
  id UUID REFERENCES auth.users(id) ON DELETE CASCADE PRIMARY KEY,
  display_name TEXT,
  date_of_birth DATE,
  avatar_url TEXT,
  preferences JSONB DEFAULT '{}',
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Enable RLS
ALTER TABLE public.profiles ENABLE ROW LEVEL SECURITY;

-- Users can only read/update their own profile
CREATE POLICY "Users can view own profile" ON public.profiles
  FOR SELECT USING (auth.uid() = id);

CREATE POLICY "Users can update own profile" ON public.profiles
  FOR UPDATE USING (auth.uid() = id);

-- Trigger to create profile on signup
CREATE OR REPLACE FUNCTION public.handle_new_user()
RETURNS TRIGGER AS $$
BEGIN
  INSERT INTO public.profiles (id, display_name)
  VALUES (NEW.id, NEW.raw_user_meta_data->>'display_name');
  RETURN NEW;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

CREATE TRIGGER on_auth_user_created
  AFTER INSERT ON auth.users
  FOR EACH ROW EXECUTE FUNCTION public.handle_new_user();
```

### 1.2 Emotional States Table

```sql
CREATE TABLE public.emotion_analyses (
  id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
  user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE NOT NULL,
  input_text TEXT NOT NULL,
  valence REAL NOT NULL CHECK (valence >= -1 AND valence <= 1),
  arousal REAL NOT NULL CHECK (arousal >= -1 AND arousal <= 1),
  stress_level REAL NOT NULL CHECK (stress_level >= 0 AND stress_level <= 1),
  primary_emotion TEXT NOT NULL,
  confidence REAL NOT NULL CHECK (confidence >= 0 AND confidence <= 1),
  emotion_vector REAL[] DEFAULT ARRAY[]::REAL[],
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Index for user queries
CREATE INDEX idx_emotion_analyses_user_id ON public.emotion_analyses(user_id);
CREATE INDEX idx_emotion_analyses_created_at ON public.emotion_analyses(created_at DESC);

-- Enable RLS
ALTER TABLE public.emotion_analyses ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own emotions" ON public.emotion_analyses
  FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own emotions" ON public.emotion_analyses
  FOR INSERT WITH CHECK (auth.uid() = user_id);
```

### 1.3 Content Catalog Table

```sql
CREATE TABLE public.content (
  id TEXT PRIMARY KEY, -- e.g., 'peaceful-meditation-001'
  title TEXT NOT NULL,
  category TEXT NOT NULL,
  duration_minutes INTEGER NOT NULL,
  description TEXT,
  thumbnail_url TEXT,
  emotional_targets JSONB DEFAULT '{}', -- Expected emotional outcomes
  tags TEXT[] DEFAULT ARRAY[]::TEXT[],
  is_active BOOLEAN DEFAULT true,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Seed some initial content
INSERT INTO public.content (id, title, category, duration_minutes, description, emotional_targets) VALUES
  ('peaceful-meditation-001', 'Peaceful Morning Meditation', 'Wellness', 15, 'Start your day with calm', '{"targetValence": 0.6, "targetArousal": -0.3, "targetStress": 0.1}'),
  ('uplifting-comedy-001', 'Feel-Good Comedy Special', 'Comedy', 45, 'Laugh your stress away', '{"targetValence": 0.8, "targetArousal": 0.4, "targetStress": 0.1}'),
  ('focus-ambient-001', 'Deep Focus Ambient Sounds', 'Focus', 60, 'Enhance concentration', '{"targetValence": 0.3, "targetArousal": -0.2, "targetStress": 0.2}'),
  ('energizing-workout-001', 'High Energy Workout Mix', 'Fitness', 30, 'Get pumped up', '{"targetValence": 0.7, "targetArousal": 0.8, "targetStress": 0.3}'),
  ('calming-nature-001', 'Nature Sounds for Relaxation', 'Wellness', 20, 'Connect with nature', '{"targetValence": 0.5, "targetArousal": -0.5, "targetStress": 0.05}');

-- Public read access for content catalog
ALTER TABLE public.content ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Anyone can view active content" ON public.content
  FOR SELECT USING (is_active = true);
```

### 1.4 Feedback/Experiences Table

```sql
CREATE TABLE public.feedback (
  id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
  user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE NOT NULL,
  content_id TEXT REFERENCES public.content(id) NOT NULL,

  -- Emotional states
  emotion_before JSONB NOT NULL, -- {valence, arousal, stressLevel, primaryEmotion, confidence}
  emotion_after JSONB NOT NULL,
  desired_state JSONB, -- User's target emotional state

  -- Engagement metrics
  watch_duration_ms INTEGER NOT NULL,
  total_duration_ms INTEGER NOT NULL,
  completed BOOLEAN NOT NULL DEFAULT false,
  star_rating INTEGER CHECK (star_rating >= 1 AND star_rating <= 5),

  -- RL metrics
  reward REAL NOT NULL,
  q_value_before REAL,
  q_value_after REAL,
  was_exploration BOOLEAN DEFAULT false,

  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for efficient queries
CREATE INDEX idx_feedback_user_id ON public.feedback(user_id);
CREATE INDEX idx_feedback_content_id ON public.feedback(content_id);
CREATE INDEX idx_feedback_created_at ON public.feedback(created_at DESC);
CREATE INDEX idx_feedback_user_content ON public.feedback(user_id, content_id);

-- Enable RLS
ALTER TABLE public.feedback ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own feedback" ON public.feedback
  FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can insert own feedback" ON public.feedback
  FOR INSERT WITH CHECK (auth.uid() = user_id);
```

### 1.5 Q-Table for RL Policy

```sql
CREATE TABLE public.q_values (
  id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
  user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE NOT NULL,
  state_key TEXT NOT NULL, -- Discretized emotional state key
  content_id TEXT REFERENCES public.content(id) NOT NULL,
  q_value REAL NOT NULL DEFAULT 0.5,
  visit_count INTEGER NOT NULL DEFAULT 0,
  last_updated TIMESTAMPTZ DEFAULT NOW(),

  UNIQUE(user_id, state_key, content_id)
);

-- Index for efficient lookups
CREATE INDEX idx_q_values_user_state ON public.q_values(user_id, state_key);

-- Enable RLS
ALTER TABLE public.q_values ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view own q_values" ON public.q_values
  FOR SELECT USING (auth.uid() = user_id);

CREATE POLICY "Users can manage own q_values" ON public.q_values
  FOR ALL USING (auth.uid() = user_id);
```

### 1.6 User Progress/Analytics View

```sql
-- Materialized view for fast progress queries
CREATE MATERIALIZED VIEW public.user_progress AS
SELECT
  user_id,
  COUNT(*) as total_experiences,
  COUNT(*) FILTER (WHERE completed) as completed_content,
  ROUND(AVG(reward)::numeric, 3) as average_reward,
  ROUND(STDDEV(reward)::numeric, 3) as reward_variance,
  COUNT(*) FILTER (WHERE was_exploration) as exploration_count,
  COUNT(*) FILTER (WHERE NOT was_exploration) as exploitation_count,
  MAX(created_at) as last_activity,
  ARRAY_AGG(reward ORDER BY created_at DESC) FILTER (WHERE reward IS NOT NULL) as recent_rewards
FROM public.feedback
GROUP BY user_id;

-- Create index on the materialized view
CREATE UNIQUE INDEX idx_user_progress_user_id ON public.user_progress(user_id);

-- Function to refresh the view (call periodically or on feedback insert)
CREATE OR REPLACE FUNCTION refresh_user_progress()
RETURNS TRIGGER AS $$
BEGIN
  REFRESH MATERIALIZED VIEW CONCURRENTLY public.user_progress;
  RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Trigger to refresh on new feedback (with debounce in production)
CREATE TRIGGER refresh_progress_on_feedback
  AFTER INSERT ON public.feedback
  FOR EACH STATEMENT
  EXECUTE FUNCTION refresh_user_progress();
```

---

## Phase 2: Supabase Edge Functions

### 2.1 Emotion Analysis Function

```typescript
// supabase/functions/analyze-emotion/index.ts
import { serve } from 'https://deno.land/std@0.168.0/http/server.ts'
import { createClient } from 'https://esm.sh/@supabase/supabase-js@2'

const emotionKeywords = {
  joy: ['happy', 'excited', 'great', 'wonderful', 'amazing', 'love', 'fantastic'],
  sadness: ['sad', 'depressed', 'down', 'unhappy', 'miserable', 'crying'],
  anger: ['angry', 'frustrated', 'annoyed', 'furious', 'irritated'],
  fear: ['scared', 'anxious', 'worried', 'nervous', 'afraid', 'terrified'],
  // ... more emotions
}

function analyzeText(text: string) {
  const lowerText = text.toLowerCase()
  let valence = 0
  let arousal = 0
  let stressLevel = 0.5
  let primaryEmotion = 'neutral'
  let maxScore = 0

  for (const [emotion, keywords] of Object.entries(emotionKeywords)) {
    const score = keywords.filter(kw => lowerText.includes(kw)).length
    if (score > maxScore) {
      maxScore = score
      primaryEmotion = emotion
    }
  }

  // Calculate valence/arousal based on primary emotion
  const emotionMap = {
    joy: { valence: 0.8, arousal: 0.6, stress: 0.1 },
    sadness: { valence: -0.6, arousal: -0.4, stress: 0.6 },
    anger: { valence: -0.7, arousal: 0.8, stress: 0.8 },
    fear: { valence: -0.5, arousal: 0.6, stress: 0.9 },
    neutral: { valence: 0, arousal: 0, stress: 0.3 },
  }

  const mapping = emotionMap[primaryEmotion] || emotionMap.neutral
  return {
    valence: mapping.valence + (Math.random() - 0.5) * 0.2,
    arousal: mapping.arousal + (Math.random() - 0.5) * 0.2,
    stressLevel: mapping.stress + (Math.random() - 0.5) * 0.1,
    primaryEmotion,
    confidence: Math.min(0.95, 0.6 + maxScore * 0.1),
  }
}

serve(async (req) => {
  const { text, userId } = await req.json()

  const supabase = createClient(
    Deno.env.get('SUPABASE_URL')!,
    Deno.env.get('SUPABASE_SERVICE_ROLE_KEY')!
  )

  const analysis = analyzeText(text)

  // Store the analysis
  const { data, error } = await supabase
    .from('emotion_analyses')
    .insert({
      user_id: userId,
      input_text: text,
      ...analysis,
    })
    .select()
    .single()

  if (error) {
    return new Response(JSON.stringify({ error: error.message }), { status: 400 })
  }

  return new Response(JSON.stringify({
    success: true,
    data: { state: analysis, analysisId: data.id }
  }))
})
```

### 2.2 Recommendation Engine Function

```typescript
// supabase/functions/get-recommendations/index.ts
import { serve } from 'https://deno.land/std@0.168.0/http/server.ts'
import { createClient } from 'https://esm.sh/@supabase/supabase-js@2'

function discretizeState(valence: number, arousal: number, stress: number): string {
  const vBin = Math.floor((valence + 1) * 2.5) // 0-5
  const aBin = Math.floor((arousal + 1) * 2.5)
  const sBin = Math.floor(stress * 5)
  return `${vBin}_${aBin}_${sBin}`
}

serve(async (req) => {
  const { userId, currentState, desiredState, limit = 5 } = await req.json()

  const supabase = createClient(
    Deno.env.get('SUPABASE_URL')!,
    Deno.env.get('SUPABASE_SERVICE_ROLE_KEY')!
  )

  const stateKey = discretizeState(
    currentState.valence,
    currentState.arousal,
    currentState.stressLevel
  )

  // Get Q-values for this state
  const { data: qValues } = await supabase
    .from('q_values')
    .select('content_id, q_value, visit_count')
    .eq('user_id', userId)
    .eq('state_key', stateKey)

  // Get all content
  const { data: content } = await supabase
    .from('content')
    .select('*')
    .eq('is_active', true)

  // Epsilon-greedy selection
  const epsilon = 0.15
  const qMap = new Map(qValues?.map(q => [q.content_id, q]) || [])

  const recommendations = content?.map(c => {
    const qEntry = qMap.get(c.id)
    const qValue = qEntry?.q_value || 0.5
    const targets = c.emotional_targets || {}

    // Calculate predicted outcome
    const predictedOutcome = {
      expectedValence: targets.targetValence || 0,
      expectedArousal: targets.targetArousal || 0,
      expectedStress: targets.targetStress || 0.3,
      confidence: qEntry ? Math.min(0.95, 0.5 + qEntry.visit_count * 0.05) : 0.5,
    }

    // Score based on Q-value and alignment with desired state
    const alignmentScore = 1 - Math.abs(predictedOutcome.expectedValence - desiredState.valence) / 2
    const combinedScore = qValue * 0.6 + alignmentScore * 0.4

    return {
      contentId: c.id,
      title: c.title,
      category: c.category,
      duration: c.duration_minutes,
      combinedScore,
      predictedOutcome,
      reasoning: `Based on your preferences and ${qEntry?.visit_count || 0} similar experiences`,
      isExploration: Math.random() < epsilon,
    }
  })
  .sort((a, b) => b.combinedScore - a.combinedScore)
  .slice(0, limit)

  return new Response(JSON.stringify({
    success: true,
    data: { recommendations }
  }))
})
```

### 2.3 Feedback Processing Function

```typescript
// supabase/functions/submit-feedback/index.ts
import { serve } from 'https://deno.land/std@0.168.0/http/server.ts'
import { createClient } from 'https://esm.sh/@supabase/supabase-js@2'

function calculateReward(
  emotionBefore: any,
  emotionAfter: any,
  desiredState: any,
  completed: boolean,
  rating: number
): number {
  // Emotional improvement component
  const valenceDelta = emotionAfter.valence - emotionBefore.valence
  const stressDelta = emotionBefore.stressLevel - emotionAfter.stressLevel

  // Alignment with desired state
  const valenceAlignment = 1 - Math.abs(emotionAfter.valence - desiredState.valence)
  const stressAlignment = 1 - Math.abs(emotionAfter.stressLevel - desiredState.stressLevel)

  // Combine components
  let reward = 0
  reward += valenceDelta * 0.3 // Valence improvement
  reward += stressDelta * 0.2 // Stress reduction
  reward += valenceAlignment * 0.2 // Goal alignment
  reward += stressAlignment * 0.1 // Stress goal alignment
  reward += (rating - 3) * 0.1 // User rating (-0.2 to +0.2)
  reward += completed ? 0.1 : -0.05 // Completion bonus

  return Math.max(-1, Math.min(1, reward))
}

function discretizeState(valence: number, arousal: number, stress: number): string {
  const vBin = Math.floor((valence + 1) * 2.5)
  const aBin = Math.floor((arousal + 1) * 2.5)
  const sBin = Math.floor(stress * 5)
  return `${vBin}_${aBin}_${sBin}`
}

serve(async (req) => {
  const {
    userId,
    contentId,
    emotionBefore,
    emotionAfter,
    desiredState,
    watchDurationMs,
    totalDurationMs,
    completed,
    starRating,
  } = await req.json()

  const supabase = createClient(
    Deno.env.get('SUPABASE_URL')!,
    Deno.env.get('SUPABASE_SERVICE_ROLE_KEY')!
  )

  // Calculate reward
  const reward = calculateReward(emotionBefore, emotionAfter, desiredState, completed, starRating)

  // Get current Q-value
  const stateKey = discretizeState(emotionBefore.valence, emotionBefore.arousal, emotionBefore.stressLevel)

  const { data: existingQ } = await supabase
    .from('q_values')
    .select('q_value, visit_count')
    .eq('user_id', userId)
    .eq('state_key', stateKey)
    .eq('content_id', contentId)
    .single()

  const oldQValue = existingQ?.q_value || 0.5
  const visitCount = (existingQ?.visit_count || 0) + 1

  // Q-learning update: Q(s,a) = Q(s,a) + α * (reward - Q(s,a))
  const learningRate = 0.1
  const newQValue = oldQValue + learningRate * (reward - oldQValue)

  // Update Q-value
  await supabase
    .from('q_values')
    .upsert({
      user_id: userId,
      state_key: stateKey,
      content_id: contentId,
      q_value: newQValue,
      visit_count: visitCount,
      last_updated: new Date().toISOString(),
    })

  // Store feedback
  const { data: feedback, error } = await supabase
    .from('feedback')
    .insert({
      user_id: userId,
      content_id: contentId,
      emotion_before: emotionBefore,
      emotion_after: emotionAfter,
      desired_state: desiredState,
      watch_duration_ms: watchDurationMs,
      total_duration_ms: totalDurationMs,
      completed,
      star_rating: starRating,
      reward,
      q_value_before: oldQValue,
      q_value_after: newQValue,
      was_exploration: false, // Set based on how content was selected
    })
    .select()
    .single()

  if (error) {
    return new Response(JSON.stringify({ error: error.message }), { status: 400 })
  }

  return new Response(JSON.stringify({
    success: true,
    data: {
      feedbackId: feedback.id,
      reward,
      newQValue,
      learningProgress: {
        totalExperiences: visitCount,
        avgReward: newQValue,
      }
    }
  }))
})
```

---

## Phase 3: Frontend Migration

### 3.1 Install Supabase Client

```bash
cd apps/emotistream-web
npm install @supabase/supabase-js @supabase/auth-helpers-nextjs
```

### 3.2 Supabase Client Setup

```typescript
// src/lib/supabase/client.ts
import { createBrowserClient } from '@supabase/ssr'

export function createClient() {
  return createBrowserClient(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!
  )
}
```

```typescript
// src/lib/supabase/server.ts
import { createServerClient, type CookieOptions } from '@supabase/ssr'
import { cookies } from 'next/headers'

export async function createClient() {
  const cookieStore = await cookies()

  return createServerClient(
    process.env.NEXT_PUBLIC_SUPABASE_URL!,
    process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!,
    {
      cookies: {
        getAll() {
          return cookieStore.getAll()
        },
        setAll(cookiesToSet) {
          try {
            cookiesToSet.forEach(({ name, value, options }) =>
              cookieStore.set(name, value, options)
            )
          } catch {
            // Server component, ignore
          }
        },
      },
    }
  )
}
```

### 3.3 Auth Hook Migration

```typescript
// src/lib/hooks/useSupabaseAuth.ts
'use client'

import { useEffect, useState } from 'react'
import { createClient } from '@/lib/supabase/client'
import type { User, Session } from '@supabase/supabase-js'

export function useSupabaseAuth() {
  const [user, setUser] = useState<User | null>(null)
  const [session, setSession] = useState<Session | null>(null)
  const [loading, setLoading] = useState(true)
  const supabase = createClient()

  useEffect(() => {
    // Get initial session
    supabase.auth.getSession().then(({ data: { session } }) => {
      setSession(session)
      setUser(session?.user ?? null)
      setLoading(false)
    })

    // Listen for auth changes
    const { data: { subscription } } = supabase.auth.onAuthStateChange(
      (_event, session) => {
        setSession(session)
        setUser(session?.user ?? null)
      }
    )

    return () => subscription.unsubscribe()
  }, [])

  const signIn = async (email: string, password: string) => {
    const { data, error } = await supabase.auth.signInWithPassword({
      email,
      password,
    })
    if (error) throw error
    return data
  }

  const signUp = async (email: string, password: string, metadata?: { display_name?: string }) => {
    const { data, error } = await supabase.auth.signUp({
      email,
      password,
      options: {
        data: metadata,
      },
    })
    if (error) throw error
    return data
  }

  const signOut = async () => {
    const { error } = await supabase.auth.signOut()
    if (error) throw error
  }

  return {
    user,
    session,
    loading,
    signIn,
    signUp,
    signOut,
    isAuthenticated: !!session,
  }
}
```

### 3.4 API Client Migration

```typescript
// src/lib/api/supabase-api.ts
import { createClient } from '@/lib/supabase/client'

const supabase = createClient()

// Emotion Analysis
export async function analyzeEmotion(text: string) {
  const { data: { user } } = await supabase.auth.getUser()
  if (!user) throw new Error('Not authenticated')

  const { data, error } = await supabase.functions.invoke('analyze-emotion', {
    body: { text, userId: user.id }
  })

  if (error) throw error
  return data
}

// Get Recommendations
export async function getRecommendations(currentState: any, desiredState: any) {
  const { data: { user } } = await supabase.auth.getUser()
  if (!user) throw new Error('Not authenticated')

  const { data, error } = await supabase.functions.invoke('get-recommendations', {
    body: { userId: user.id, currentState, desiredState }
  })

  if (error) throw error
  return data
}

// Submit Feedback
export async function submitFeedback(feedback: {
  contentId: string
  emotionBefore: any
  emotionAfter: any
  desiredState: any
  watchDurationMs: number
  totalDurationMs: number
  completed: boolean
  starRating: number
}) {
  const { data: { user } } = await supabase.auth.getUser()
  if (!user) throw new Error('Not authenticated')

  const { data, error } = await supabase.functions.invoke('submit-feedback', {
    body: { userId: user.id, ...feedback }
  })

  if (error) throw error
  return data
}

// Get Progress (direct database query with RLS)
export async function getProgress() {
  const { data: { user } } = await supabase.auth.getUser()
  if (!user) throw new Error('Not authenticated')

  const { data, error } = await supabase
    .from('user_progress')
    .select('*')
    .eq('user_id', user.id)
    .single()

  if (error && error.code !== 'PGRST116') throw error // PGRST116 = no rows
  return data
}

// Get Feedback History
export async function getFeedbackHistory(limit = 10) {
  const { data: { user } } = await supabase.auth.getUser()
  if (!user) throw new Error('Not authenticated')

  const { data, error } = await supabase
    .from('feedback')
    .select(`
      *,
      content:content_id (title, category)
    `)
    .eq('user_id', user.id)
    .order('created_at', { ascending: false })
    .limit(limit)

  if (error) throw error
  return data
}
```

---

## Phase 4: Migration Steps

### Step 1: Setup Supabase Project
1. Create new Supabase project at https://supabase.com
2. Note down: Project URL, Anon Key, Service Role Key
3. Configure environment variables

### Step 2: Create Database Schema
1. Run SQL migrations in Supabase SQL Editor
2. Seed initial content data
3. Test RLS policies

### Step 3: Deploy Edge Functions
1. Install Supabase CLI: `npm install -g supabase`
2. Initialize: `supabase init`
3. Create functions in `supabase/functions/`
4. Deploy: `supabase functions deploy`

### Step 4: Migrate Frontend
1. Install Supabase packages
2. Replace API client with Supabase client
3. Replace auth store with Supabase Auth
4. Update components to use new hooks

### Step 5: Data Migration
1. Export existing JSON data
2. Transform to match new schema
3. Import using Supabase API

### Step 6: Testing & Deployment
1. Test all flows locally with Supabase
2. Deploy frontend to Vercel/Lovable
3. Configure production environment variables
4. Monitor and optimize

---

## Environment Variables

```env
# .env.local (Frontend)
NEXT_PUBLIC_SUPABASE_URL=https://your-project.supabase.co
NEXT_PUBLIC_SUPABASE_ANON_KEY=your-anon-key

# Edge Functions (set in Supabase Dashboard)
SUPABASE_URL=https://your-project.supabase.co
SUPABASE_SERVICE_ROLE_KEY=your-service-role-key
```

---

## Benefits of Supabase-Native Architecture

1. **Simplified Deployment**: No separate backend to deploy/manage
2. **Built-in Auth**: Supabase Auth handles sessions, tokens, OAuth
3. **Real-time**: Easy real-time subscriptions for live updates
4. **Row Level Security**: Data isolation without backend code
5. **Edge Functions**: Serverless compute close to users
6. **Scalability**: Managed PostgreSQL scales automatically
7. **Cost-Effective**: Free tier generous for MVPs
8. **Lovable Compatible**: Perfect for Lovable.dev deployment

---

## Timeline Estimate

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Phase 1: Database Schema | 1-2 hours | Supabase project |
| Phase 2: Edge Functions | 2-3 hours | Schema complete |
| Phase 3: Frontend Migration | 3-4 hours | Functions deployed |
| Phase 4: Testing & Deploy | 2-3 hours | All phases complete |
| **Total** | **8-12 hours** | |

---

## Next Steps

1. [ ] Create Supabase project
2. [ ] Run database migrations
3. [ ] Deploy edge functions
4. [ ] Update frontend to use Supabase
5. [ ] Test end-to-end flow
6. [ ] Deploy to production
