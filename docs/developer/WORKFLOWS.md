# GitHub Workflows

This directory contains GitHub Actions workflows for automated deployment and CI/CD.

## Available Workflows

### 1. Vercel Preview Deployment

**File:** `workflows/vercel-preview.yml`

Automatically deploys to Vercel preview environment when:

- Pushing to `preview` branch
- Opening/updating pull requests to `main` branch
- Making changes in `www/` folder

**Features:**

- Deploys to Vercel preview URL
- Adds preview URL as PR comment
- Creates deployment summary

### 2. Vercel Production Deployment

**File:** `workflows/vercel-production.yml`

Automatically deploys to Vercel production when:

- Pushing to `main` branch
- Making changes in `www/` folder

**Features:**

- Deploys to production URL
- Creates deployment summary with commit info

## Setup Instructions

See [VERCEL_SETUP.md](./VERCEL_SETUP.md) for complete setup instructions.

### Quick Setup

1. **Get Vercel credentials**

   ```bash
   cd www
   vercel link
   cat .vercel/project.json
   ```

2. **Create Vercel token**

   - Go to [Vercel Account Settings → Tokens](https://vercel.com/account/tokens)
   - Create new token with Full Account scope

3. **Add GitHub secrets**

   - `VERCEL_TOKEN` - Your Vercel token
   - `VERCEL_ORG_ID` - From `.vercel/project.json`
   - `VERCEL_PROJECT_ID` - From `.vercel/project.json`

4. **Push to trigger deployment**
   ```bash
   git push origin preview  # Preview deployment
   git push origin main     # Production deployment
   ```

## Branch Strategy

```
main
├── Deploys to → production.vercel.app
└── PR merges trigger production deployment

preview
├── Deploys to → preview.vercel.app
└── Push triggers preview deployment

feature-branches (PR to main)
└── Deploys to → unique-preview-url.vercel.app
```

## Workflow Triggers

| Workflow   | Trigger           | Environment       |
| ---------- | ----------------- | ----------------- |
| Preview    | Push to `preview` | Vercel Preview    |
| Preview    | PR to `main`      | Vercel Preview    |
| Production | Push to `main`    | Vercel Production |

## Environment Variables

All workflows require these GitHub secrets:

- `VERCEL_TOKEN` - Vercel authentication token
- `VERCEL_ORG_ID` - Your Vercel organization/user ID
- `VERCEL_PROJECT_ID` - Your Vercel project ID

## Workflow Features

### ✅ Automatic Deployments

- No manual intervention needed
- Deploy on every push to configured branches

### ✅ PR Preview URLs

- Unique preview URL for each PR
- Automatic comment with deployment URL
- Updates on every commit

### ✅ Deployment Summaries

- Workflow run summaries with deployment info
- Links to deployed URLs
- Commit and trigger information

### ✅ Conditional Execution

- Only runs when `www/` folder changes
- Prevents unnecessary deployments
- Saves GitHub Actions minutes

## Customization

### Change Deployment Triggers

Edit the workflow files to customize when deployments happen:

```yaml
on:
  push:
    branches:
      - your-branch-name
    paths:
      - 'www/**'
```

### Add Environment Variables

Add secrets in GitHub Settings → Secrets and variables → Actions:

```yaml
env:
  YOUR_VAR: ${{ secrets.YOUR_SECRET }}
```

### Modify Build Command

Update the build step in workflow files:

```yaml
- name: Build Project Artifacts
  run: vercel build --token=${{ secrets.VERCEL_TOKEN }}
  env:
    YOUR_ENV_VAR: value
```

## Monitoring

### View Workflow Runs

- GitHub: Repository → Actions tab
- See all workflow runs, logs, and summaries

### View Deployments

- Vercel Dashboard: Project → Deployments
- See all deployments with GitHub commit links

## Troubleshooting

### Common Issues

1. **Workflow doesn't trigger**

   - Check if changes are in `www/` folder
   - Verify branch names match workflow configuration

2. **Authentication fails**

   - Verify all secrets are correctly set
   - Check Vercel token hasn't expired

3. **Build fails**
   - Test build locally: `cd www && npm run build`
   - Check workflow logs for specific errors

See [VERCEL_SETUP.md](./VERCEL_SETUP.md) for detailed troubleshooting.

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Vercel CLI Documentation](https://vercel.com/docs/cli)
- [Vercel + GitHub Actions Guide](https://vercel.com/guides/how-can-i-use-github-actions-with-vercel)
