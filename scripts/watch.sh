#!/usr/bin/env sh
# watch files while working

cargo watch -i '{scripts,.github}' -x check -x test -x run
