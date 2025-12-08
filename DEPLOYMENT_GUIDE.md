# Quick Deployment Guide - Get Your Live Link in 10 Minutes

**Goal:** Deploy EntertainAI to get a shareable live URL

**Stack:** Vercel (Next.js) + Railway (Python FastAPI)

---

## 🚀 Option 1: Vercel + Railway (Fastest - 10 minutes)

### Step 1: Deploy Python Backend to Railway (5 min)

1. **Go to Railway:** https://railway.app/
2. **Sign up/Login** (GitHub auth recommended)
3. **New Project** → **Deploy from GitHub repo**
4. **Select:** `dalmaraz007/agentic-pancakes`
5. **Root Directory:** Set to `/api`
6. **Build Command:** `pip install -r requirements.txt`
7. **Start Command:** `python main.py`
8. **Environment Variables:** (None needed for basic demo)
9. **Deploy** → Wait ~3 minutes
10. **Copy the URL:** e.g., `https://entertainai-api-production.up.railway.app`

### Step 2: Deploy Next.js UI to Vercel (5 min)

1. **Go to Vercel:** https://vercel.com/
2. **Sign up/Login** (GitHub auth recommended)
3. **New Project** → **Import Git Repository**
4. **Select:** `dalmaraz007/agentic-pancakes`
5. **Framework Preset:** Next.js (auto-detected)
6. **Root Directory:** `web-ui`
7. **Environment Variables:**
   ```
   PYTHON_API_URL=https://your-railway-url.up.railway.app
   NEXT_PUBLIC_APP_NAME=EntertainAI
   NEXT_PUBLIC_APP_VERSION=1.0.0
   ```
8. **Deploy** → Wait ~2 minutes
9. **Get your URL:** e.g., `https://entertainai.vercel.app`

### ✅ Done! Share your link:
```
https://entertainai.vercel.app
```

---

## 🚀 Option 2: Render (All-in-One - 15 minutes)

### Deploy Both Services on Render

1. **Go to Render:** https://render.com/
2. **Sign up/Login**

#### Backend (Python):
1. **New** → **Web Service**
2. **Connect GitHub:** `dalmaraz007/agentic-pancakes`
3. **Root Directory:** `api`
4. **Build Command:** `pip install -r requirements.txt`
5. **Start Command:** `uvicorn main:app --host 0.0.0.0 --port $PORT`
6. **Instance Type:** Free
7. **Deploy** → Copy URL

#### Frontend (Next.js):
1. **New** → **Static Site**
2. **Connect GitHub:** `dalmaraz007/agentic-pancakes`
3. **Root Directory:** `web-ui`
4. **Build Command:** `npm run build`
5. **Publish Directory:** `.next`
6. **Environment Variables:**
   ```
   PYTHON_API_URL=https://your-backend.onrender.com
   ```
7. **Deploy** → Get URL

---

## 🚀 Option 3: Google Cloud (Production-Ready - 30 minutes)

### Deploy to Cloud Run

```bash
# 1. Install Google Cloud CLI
# https://cloud.google.com/sdk/docs/install

# 2. Authenticate
gcloud auth login
gcloud config set project YOUR_PROJECT_ID

# 3. Deploy Backend
cd api
gcloud run deploy entertainai-api \
  --source . \
  --region us-central1 \
  --allow-unauthenticated

# 4. Get backend URL
# e.g., https://entertainai-api-xxxxx-uc.a.run.app

# 5. Deploy Frontend to Vercel (same as Option 1)
# Set PYTHON_API_URL to the Cloud Run URL
```

---

## 📋 Pre-Deployment Checklist

### Required Files (Already Created ✅)

- ✅ `api/main.py` - FastAPI backend
- ✅ `api/requirements.txt` - Python dependencies
- ✅ `web-ui/package.json` - Next.js dependencies
- ✅ `web-ui/.env.local.example` - Environment template

### Files We Need to Create:

1. **Procfile** (for Railway/Render)
2. **railway.json** (Railway config)
3. **render.yaml** (Render config)
4. **Dockerfile** (optional, for Cloud Run)

---

## 🔧 Quick Deploy Commands

### If you have accounts set up:

```bash
# Deploy to Vercel (from web-ui directory)
cd web-ui
npx vercel --prod

# Deploy to Railway (from api directory)
cd api
railway up

# Deploy to Render
# Use dashboard - no CLI needed
```

---

## 🆘 Common Issues

### Issue 1: CORS Errors
**Fix:** Make sure `api/main.py` has your Vercel URL in allowed origins:

```python
app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:3000",
        "https://your-vercel-app.vercel.app",  # Add this
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)
```

### Issue 2: Backend Not Responding
**Check:**
1. Railway/Render logs show "Uvicorn running on..."
2. Health endpoint works: `https://your-backend.up.railway.app/health`
3. Environment variable `PYTHON_API_URL` is correct in Vercel

### Issue 3: Next.js Build Fails
**Fix:** Ensure `web-ui/package.json` has all dependencies

---

## 📊 Deployment Status

After deployment, verify:

1. **Backend Health Check:**
   ```bash
   curl https://your-backend-url/health
   # Should return: {"status":"healthy","service":"EntertainAI API"}
   ```

2. **Frontend Loads:**
   ```bash
   curl https://your-frontend-url
   # Should return HTML
   ```

3. **API Integration:**
   ```bash
   curl -X POST https://your-frontend-url/api/recommendations \
     -H "Content-Type: application/json" \
     -d '{"query":"action thriller"}'
   # Should return recommendations
   ```

---

## 🎯 Recommended Approach for Hackathon Demo

**Use: Vercel + Railway**

**Why:**
- ✅ Fastest deployment (10 minutes)
- ✅ Free tier (no credit card needed)
- ✅ Automatic HTTPS
- ✅ Great performance
- ✅ Easy to share URL

**Steps:**
1. Deploy backend to Railway (5 min)
2. Deploy frontend to Vercel (5 min)
3. Share Vercel URL with judges

---

## 💡 Pro Tips

1. **Custom Domain (Optional):**
   - Vercel allows custom domains on free tier
   - Railway allows custom domains on free tier
   - Example: `entertainai.yourdomain.com`

2. **Analytics:**
   - Vercel has built-in analytics
   - Add to see demo traffic

3. **Preview Deployments:**
   - Every git push creates a preview URL
   - Test before promoting to production

4. **Environment Variables:**
   - Store in Vercel/Railway dashboard
   - Never commit secrets to git

---

## 🔗 Useful Links

- **Vercel Docs:** https://vercel.com/docs
- **Railway Docs:** https://docs.railway.app/
- **Render Docs:** https://render.com/docs
- **Cloud Run Docs:** https://cloud.google.com/run/docs

---

## ✅ Quick Deploy Checklist

- [ ] Railway account created
- [ ] Vercel account created
- [ ] Python backend deployed to Railway
- [ ] Railway URL copied
- [ ] Next.js deployed to Vercel
- [ ] PYTHON_API_URL environment variable set in Vercel
- [ ] Backend health check passes
- [ ] Frontend loads correctly
- [ ] API integration works
- [ ] Live URL shared with judges

**Estimated Time:** 10-15 minutes total

---

**Ready to deploy? Let me know which option you want and I'll guide you through it step-by-step!**

---

**Document Version:** 1.0
**Last Updated:** 2024-12-06
**Status:** Ready for Deployment
