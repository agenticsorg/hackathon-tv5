# GitHub + Vercel Integration Setup

This guide shows how to set up GitHub Actions to automatically deploy to Vercel.

## Prerequisites

- Vercel account with a project created
- GitHub repository with admin access
- Vercel CLI installed locally (for setup)

## Step 1: Get Vercel Credentials

### Install Vercel CLI

```bash
npm install -g vercel
```

### Login to Vercel

```bash
vercel login
```

### Link Project and Get Credentials

```bash
cd www
vercel link
```

This creates a `.vercel` folder with your project configuration.

### Get Project IDs

```bash
# Get Organization ID
cat .vercel/project.json | grep orgId

# Get Project ID
cat .vercel/project.json | grep projectId
```

### Get Vercel Token

1. Go to [Vercel Account Settings → Tokens](https://vercel.com/account/tokens)
2. Click "Create Token"
3. Name: `GitHub Actions`
4. Scope: `Full Account`
5. Expiration: Choose appropriate duration
6. Copy the token (you won't see it again!)

## Step 2: Add GitHub Secrets

Go to your GitHub repository settings:

**Settings → Secrets and variables → Actions → New repository secret**

Add these three secrets:

### 1. VERCEL_TOKEN

```
Value: [Your Vercel token from Step 1]
```

### 2. VERCEL_ORG_ID

```
Value: [Your org ID from .vercel/project.json]
Example: team_xxxxxxxxxxxxx
```

### 3. VERCEL_PROJECT_ID

```
Value: [Your project ID from .vercel/project.json]
Example: prj_xxxxxxxxxxxxx
```

## Step 3: Verify Workflows

The repository includes two workflows:

### Production Deployment

**File:** `.github/workflows/vercel-production.yml`

**Triggers:**

- Push to `main` branch
- Changes in `www/` folder

**Deploys to:** Vercel Production

### Preview Deployment

**File:** `.github/workflows/vercel-preview.yml`

**Triggers:**

- Push to `preview` branch
- Pull requests to `main` branch
- Changes in `www/` folder

**Deploys to:** Vercel Preview

## Step 4: Test Deployment

### Test Preview Deployment

```bash
# Make a change
cd www
echo "# Test" >> README.md

# Commit and push to preview branch
git add .
git commit -m "Test preview deployment"
git push origin preview
```

### Test Production Deployment

```bash
# Switch to main
git checkout main

# Merge changes
git merge preview

# Push to production
git push origin main
```

## Step 5: Verify in GitHub

1. Go to **Actions** tab in your repository
2. You should see workflow runs
3. Click on a run to see deployment details
4. Check the summary for the deployment URL

## GitHub Actions Features

### Automatic Preview URLs on PRs

When you create a pull request, the workflow will:

1. ✅ Deploy to Vercel Preview
2. ✅ Comment on the PR with the preview URL
3. ✅ Update on each commit

### Deployment Summaries

Each workflow run creates a summary with:

- Deployment URL
- Commit SHA
- Trigger information

## Branch Strategy

### Main Branch → Production

```
main
└── Deploys to production.vercel.app
```

### Preview Branch → Preview Environment

```
preview
└── Deploys to preview.vercel.app
```

### Pull Requests → Preview URLs

```
feature-branch (PR to main)
└── Deploys to unique preview URL
```

## Workflow Configuration

Both workflows:

- ✅ Use Node.js 20.x
- ✅ Cache npm dependencies
- ✅ Run only when `www/` changes
- ✅ Use Vercel CLI for deployment
- ✅ Create deployment summaries

## Troubleshooting

### Secrets Not Found

**Error:** `Error: Input required and not supplied: token`

**Fix:** Verify all three secrets are added in GitHub Settings:

- `VERCEL_TOKEN`
- `VERCEL_ORG_ID`
- `VERCEL_PROJECT_ID`

### Wrong Project Deployed

**Error:** Deploying to wrong project

**Fix:**

1. Delete `.vercel` folder
2. Run `vercel link` again
3. Update `VERCEL_PROJECT_ID` secret

### Build Fails

**Error:** Build errors in workflow

**Fix:**

1. Test build locally: `cd www && npm run build`
2. Check workflow logs for specific errors
3. Ensure all dependencies are in `package.json`

### Permission Denied

**Error:** Cannot deploy to Vercel

**Fix:**

1. Verify Vercel token has correct scope
2. Check organization/project access
3. Regenerate token if needed

## Security Best Practices

- ✅ Never commit `.vercel` folder
- ✅ Never commit Vercel token
- ✅ Use GitHub Secrets for all credentials
- ✅ Rotate tokens periodically
- ✅ Use minimal scope tokens when possible

## Disabling Vercel's GitHub Integration

If you're using GitHub Actions for deployment, you may want to disable Vercel's automatic GitHub integration:

1. Go to Vercel Dashboard → Project Settings
2. Click on "Git" tab
3. Disconnect GitHub repository
4. This prevents double deployments

## Alternative: Vercel's Native GitHub Integration

If you prefer Vercel's built-in integration instead:

1. Go to Vercel Dashboard
2. Import your GitHub repository
3. Vercel will automatically deploy on push
4. **No GitHub Actions needed**

The workflows in this repo provide more control and customization.

## Next Steps

- [ ] Add secrets to GitHub
- [ ] Test preview deployment
- [ ] Test production deployment
- [ ] Configure custom domain in Vercel
- [ ] Set up deployment notifications (optional)

## Resources

- [Vercel CLI Documentation](https://vercel.com/docs/cli)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Vercel GitHub Actions Guide](https://vercel.com/guides/how-can-i-use-github-actions-with-vercel)
