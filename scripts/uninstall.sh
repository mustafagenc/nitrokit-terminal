#!/bin/bash

# Nitroterm macOS/Linux Uninstaller

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
FORCE="${FORCE:-false}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${RED}🗑️  Nitroterm Uninstaller${NC}"
echo -e "${YELLOW}Removing Nitroterm from: $INSTALL_DIR${NC}"

if [[ "$FORCE" != "true" ]]; then
    read -p "Are you sure you want to uninstall Nitroterm? [y/N]: " -r
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${GREEN}Uninstallation cancelled.${NC}"
        exit 0
    fi
fi

# Remove binary
if [[ -f "$INSTALL_DIR/nitroterm" ]]; then
    echo -e "${BLUE}📁 Removing Nitroterm binary...${NC}"
    rm -f "$INSTALL_DIR/nitroterm"
    echo -e "${GREEN}✅ Nitroterm binary removed!${NC}"
fi

echo -e "${YELLOW}⚠️  Note: PATH entries and aliases in shell configuration files were not automatically removed.${NC}"
echo -e "${YELLOW}You may want to manually remove them from ~/.bashrc, ~/.zshrc, etc.${NC}"

echo -e "${GREEN}🎉 Nitroterm has been successfully uninstalled!${NC}"