# Icon Generation

The Tauri configuration expects the following icon sizes:
- 32x32.png
- 128x128.png
- 128x128@2x.png (256x256)
- icon.icns (macOS)
- icon.ico (Windows)

## Current Status

✅ icon.png (source: 512x512 or larger)
✅ icon.icns (macOS bundle icon)

## Generate Missing Sizes

### Using ImageMagick (Linux/macOS/Windows)
```bash
cd src-tauri/icons

# Generate required PNG sizes
convert icon.png -resize 32x32 32x32.png
convert icon.png -resize 128x128 128x128.png
convert icon.png -resize 256x256 128x128@2x.png

# Generate Windows ICO (contains multiple sizes)
convert icon.png -define icon:auto-resize=256,128,96,64,48,32,16 icon.ico
```

### Using macOS sips
```bash
cd src-tauri/icons

sips -z 32 32 icon.png --out 32x32.png
sips -z 128 128 icon.png --out 128x128.png
sips -z 256 256 icon.png --out 128x128@2x.png
```

### Manual Alternative

If no tools are available, the application will use icon.png as fallback.
The icons can be generated later for production builds.

## Notes

- icon.png should be at least 512x512 for best quality
- icon.icns is already present for macOS
- Windows .ico should contain multiple sizes for different contexts
