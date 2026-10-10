#!/bin/sh
# Scaffolder aplikacji na libLunie (środowisko Luna).
#
# Użycie:
#   ./tools/libluna-new.sh <katalog-aplikacji> [nazwa-pakietu]
#
# Tworzy katalog z Cargo.toml (zależność `libluna` wskazująca na to drzewo)
# oraz src/main.rs z szablonu templates/minimal-app.

set -eu

if [ "$#" -lt 1 ]; then
    echo "użycie: libluna-new.sh <katalog-aplikacji> [nazwa-pakietu]" >&2
    exit 1
fi

target="$1"
name="${2:-$(basename "$target")}"

here="$(cd "$(dirname "$0")" && pwd)"
root="$(cd "$here/.." && pwd)"
libluna_path="$root/crates/libluna"
template="$root/templates/minimal-app"

if [ ! -d "$template" ]; then
    echo "nie znaleziono szablonu: $template" >&2
    exit 1
fi

mkdir -p "$target/src"

sed "s#__APP_NAME__#$name#g; s#__LIBLUNA_PATH__#$libluna_path#g" \
    "$template/Cargo.toml" > "$target/Cargo.toml"
sed "s#__APP_NAME__#$name#g" \
    "$template/src/main.rs" > "$target/src/main.rs"

echo "utworzono aplikację: $target"
echo "  pakiet: $name"
echo "  libluna: $libluna_path"
echo "uruchom: (cd $target && cargo run)"
