#!/bin/bash

# This script builds ratatoskr and then copies the exec to ~/.config/niri

cargo build --release;
cp ~/Repositories/ratatoskr/target/release/ratatoskr ~/.config/niri/ \
    && echo -e "\n\033[0;32m\033[1mratatoskr built and copied to ~/.config/niri\033[0m" \
    || echo -e "\n\033[0;31m\033[1mratatoskr copying failed\033[0m"