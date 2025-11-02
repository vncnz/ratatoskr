#!/bin/bash

# This script builds ratatoskr and then copies the exec to ~/.config/niri

cargo build --release --bin legacy-ratatoskr;
cp ~/Repositories/ratatoskr/target/release/legacy-ratatoskr ~/.config/niri/ \
    && echo -e "\n\033[0;32m\033[1mlegacy-ratatoskr built and copied to ~/.config/niri\033[0m" \
    || echo -e "\n\033[0;31m\033[1mlegacy-ratatoskr copying failed\033[0m";

cargo build --release --bin sock-ratatoskr;
cp ~/Repositories/ratatoskr/target/release/sock-ratatoskr ~/.config/niri/ \
    && echo -e "\n\033[0;32m\033[1msock-ratatoskr built and copied to ~/.config/niri\033[0m" \
    || echo -e "\n\033[0;31m\033[1msock-ratatoskr copying failed\033[0m";