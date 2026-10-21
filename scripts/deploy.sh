#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

APP_NAME="Handsdown.app"
BUILD_PATH="src-tauri/target/release/bundle/macos/$APP_NAME"
INSTALL_PATH="/Applications/$APP_NAME"

echo -e "${BLUE}🔨 Building Handy...${NC}"
bun run tauri build

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Build complete!${NC}"

# Check if the built app exists
if [ ! -d "$BUILD_PATH" ]; then
    echo -e "${RED}❌ Built app not found at $BUILD_PATH${NC}"
    exit 1
fi

echo -e "${BLUE}📦 Installing to /Applications...${NC}"

# Check if old version exists and remove it
if [ -d "$INSTALL_PATH" ]; then
    echo -e "${YELLOW}⚠️  Removing old version...${NC}"
    rm -rf "$INSTALL_PATH"

    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Failed to remove old version (may need sudo)${NC}"
        echo -e "${YELLOW}Try running: sudo rm -rf $INSTALL_PATH${NC}"
        exit 1
    fi
fi

# Copy new version
cp -R "$BUILD_PATH" "$INSTALL_PATH"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Handy installed to /Applications/${NC}"
    echo -e "${GREEN}🚀 You can now launch Handy from Applications folder${NC}"
    echo -e "${YELLOW}⚠️ Remember to access the Settings page for system permissions!${NC}"

    # Optional: Open the Applications folder
    echo -e "${BLUE}📂 Opening Applications folder...${NC}"
    open /Applications
else
    echo -e "${RED}❌ Failed to copy to /Applications${NC}"
    echo -e "${YELLOW}You may need to run with sudo:${NC}"
    echo -e "${YELLOW}  sudo cp -R $BUILD_PATH $INSTALL_PATH${NC}"
    exit 1
fi
