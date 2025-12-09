# RuVector Engine Reference Setup Guide

This document outlines the optimal approaches for referencing the `vibe-cast/ruvector-engine` content in the `hackathon-tv5` repository.

## 📋 Source Repository

**Repository:** https://github.com/mondweep/vibe-cast  
**Branch:** `claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn`  
**Path:** `/ruvector-engine`

---

## 🎯 Recommended Approaches

### Option 1: Git Submodule ⭐ (Recommended for Active Development)

**Use when:**
- You need to keep the code synchronized with the source
- You might make changes to contribute back
- You want version control for the referenced code

**Setup:**

```bash
# Add the submodule
git submodule add -b claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn \
  https://github.com/mondweep/vibe-cast.git \
  mondweep/vibe-cast

# Initialize and update
git submodule update --init --recursive

# Create a symlink to the specific directory
ln -s mondweep/vibe-cast/ruvector-engine mondweep/ruvector-engine
```

**Update the submodule:**

```bash
# Pull latest changes
cd mondweep/vibe-cast
git pull origin claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn
cd ../..
git add mondweep/vibe-cast
git commit -m "Update vibe-cast submodule"
```

**Pros:**
- ✅ Maintains git history
- ✅ Easy to sync updates
- ✅ Can contribute changes back
- ✅ Version pinning

**Cons:**
- ❌ Requires submodule commands
- ❌ Team members must init submodules

---

### Option 2: Git Subtree (Best for Independent Fork)

**Use when:**
- You want a complete copy that can diverge
- You don't need to sync frequently
- You want simpler workflow for collaborators

**Setup:**

```bash
# Add the remote
git remote add vibe-cast https://github.com/mondweep/vibe-cast.git

# Fetch the branch
git fetch vibe-cast claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn

# Add as subtree
git subtree add --prefix=mondweep/ruvector-engine \
  vibe-cast claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn \
  --squash
```

**Update the subtree:**

```bash
git subtree pull --prefix=mondweep/ruvector-engine \
  vibe-cast claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn \
  --squash
```

**Pros:**
- ✅ No special commands for collaborators
- ✅ Full code in main repo
- ✅ Can modify freely
- ✅ Can push changes back if needed

**Cons:**
- ❌ Larger repository size
- ❌ More complex merge history

---

### Option 3: NPM Package (Best for Production Use)

**Use when:**
- The code is stable and versioned
- You want dependency management
- You need to use it across multiple projects

**Setup:**

```bash
# If ruvector-engine has a package.json
npm install github:mondweep/vibe-cast#claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn

# Or specify a subdirectory if supported
npm install mondweep/vibe-cast#claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn:ruvector-engine
```

**Pros:**
- ✅ Standard dependency management
- ✅ Version locking
- ✅ Easy to update
- ✅ Works with npm/pnpm/yarn

**Cons:**
- ❌ Requires package.json in source
- ❌ Less flexible for development

---

### Option 4: Direct Copy (Simplest, for Reference Only)

**Use when:**
- You just need a snapshot
- No ongoing synchronization needed
- Quick prototyping

**Setup:**

```bash
# Clone to temp location
git clone -b claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn \
  https://github.com/mondweep/vibe-cast.git /tmp/vibe-cast

# Copy the directory
cp -r /tmp/vibe-cast/ruvector-engine mondweep/

# Clean up
rm -rf /tmp/vibe-cast

# Add to git
git add mondweep/ruvector-engine
git commit -m "Add ruvector-engine from vibe-cast"
```

**Pros:**
- ✅ Simple and straightforward
- ✅ No external dependencies
- ✅ Full control

**Cons:**
- ❌ No sync mechanism
- ❌ Manual updates required
- ❌ No version tracking

---

### Option 5: Sparse Checkout (Advanced)

**Use when:**
- You want only specific files from a large repo
- You need git tracking but minimal disk usage

**Setup:**

```bash
# Create a new directory
mkdir -p mondweep/vibe-cast-sparse
cd mondweep/vibe-cast-sparse

# Initialize sparse checkout
git init
git remote add origin https://github.com/mondweep/vibe-cast.git
git config core.sparseCheckout true

# Specify the path to checkout
echo "ruvector-engine/*" >> .git/info/sparse-checkout

# Pull the specific branch
git pull origin claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn

cd ../..
```

**Pros:**
- ✅ Only downloads needed files
- ✅ Git tracking
- ✅ Can sync updates

**Cons:**
- ❌ Complex setup
- ❌ Less common workflow

---

## 🏆 Recommendation Matrix

| Scenario | Recommended Option |
|----------|-------------------|
| Active development with sync | **Option 1: Submodule** |
| Fork and customize | **Option 2: Subtree** |
| Production dependency | **Option 3: NPM Package** |
| Quick reference/prototype | **Option 4: Direct Copy** |
| Large repo, specific files | **Option 5: Sparse Checkout** |

---

## 📝 Additional Considerations

### Documentation Reference

Create a `mondweep/docs/ruvector-integration.md` to document:
- Why you're using ruvector-engine
- How it integrates with your hackathon project
- Any modifications made
- Link to source repository

### .gitignore Updates

If using submodules or external references, ensure your `.gitignore` is configured appropriately.

### Attribution

Add attribution in your README:

```markdown
## Dependencies

This project uses [RuVector Engine](https://github.com/mondweep/vibe-cast/tree/claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn/ruvector-engine) for vector database functionality.
```

---

## 🚀 Next Steps

1. Choose the approach that best fits your use case
2. Follow the setup instructions above
3. Document the integration in `mondweep/docs/`
4. Update your main README with attribution
5. Test the integration with your hackathon project

---

**Created:** 2025-12-05  
**Source:** https://github.com/mondweep/vibe-cast/tree/claude/agentic-hackathon-setup-01MsFnEEndzVH9sYmgJwfLhn/ruvector-engine
