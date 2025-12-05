# Extension Icons

This directory should contain the following PNG icon files for the ARW Chrome Extension:

## Required Icons

- **icon16.png** - 16x16 pixels (toolbar icon)
- **icon32.png** - 32x32 pixels (extension management)
- **icon48.png** - 48x48 pixels (extension management)
- **icon128.png** - 128x128 pixels (Chrome Web Store, installation)

## Creating Icons

### Option 1: Use Online Generator

1. Go to [favicon.io](https://favicon.io/favicon-generator/)
2. Configure:
   - Text: "ARW"
   - Background: #3b82f6 (blue)
   - Font: Inter or similar
   - Size: 128x128
3. Download and extract
4. Copy files to this directory

### Option 2: Design Custom Icons

Use any image editor (Figma, Photoshop, GIMP) to create:
- Simple "ARW" text on blue background
- Magnifying glass icon (inspector theme)
- Checkmark icon (compliance theme)
- Web icon with "A" overlay

**Design Guidelines:**
- Clear and recognizable at 16x16
- Simple, not too detailed
- Blue color scheme (#3b82f6)
- Consistent across sizes

### Option 3: Use Existing Icons

Copy appropriate icons from:
- arw-inspector favicons
- Other ARW project assets
- Free icon libraries

## Temporary Workaround

For development, you can use placeholder files:
```bash
# Create empty placeholder files (extension will load but no icon shown)
touch icon16.png icon32.png icon48.png icon128.png
```

The extension will function without icons, but:
- No toolbar icon displayed
- Chrome shows default icon
- Extension still works completely

## SVG to PNG Conversion

If you have an SVG logo:
```bash
# Using ImageMagick
convert -background none -size 16x16 logo.svg icon16.png
convert -background none -size 32x32 logo.svg icon32.png
convert -background none -size 48x48 logo.svg icon48.png
convert -background none -size 128x128 logo.svg icon128.png
```

## Current Status

⚠️ **Icons not yet created** - Add PNG files to this directory before publishing.

For development: Extension works without icons, just shows default Chrome icon.
