#!/bin/sh
# Build the GitHub Pages copy of the site into $1 (default: dist/).
#
# The site is static, so this only drops development files and rebases the
# 404 page, whose links are root-absolute (it is served at any missing URL),
# onto the project path, e.g. https://flotrpy.github.io/mcp/.
set -eu
here=$(cd "$(dirname "$0")/.." && pwd)
out=${1:-dist}
base=${PAGES_BASE:-/mcp/}

rm -rf "$out"
mkdir -p "$out"
cp -R "$here"/. "$out"/
rm -rf "$out/tools" "$out/tests" "$out/package.json" "$out/vercel.json" "$out/README.md"
# Pages has no clean URLs, so /dashboard becomes /mcp/dashboard.html.
sed -i.bak -e "s#\(href\|src\)=\"/#\1=\"${base}#g" -e "s#href=\"${base}dashboard\"#href=\"${base}dashboard.html\"#g" "$out/404.html"
rm -f "$out/404.html.bak"
touch "$out/.nojekyll"
echo "built $out for $base"
