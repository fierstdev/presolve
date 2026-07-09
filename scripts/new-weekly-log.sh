#!/usr/bin/env sh
set -eu
week="${1:-$(date +%G-W%V)}"
file="notes/progress/${week}.md"
if [ -e "$file" ]; then
  echo "$file already exists"
  exit 1
fi
cat > "$file" <<EOF
# ${week} Progress Log

## Focus

TBD

## Completed

- TBD

## Learned

- TBD

## Decisions

- TBD

## Blockers

- TBD

## Next focus

- TBD
EOF

echo "created $file"
