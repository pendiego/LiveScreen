#!/usr/bin/env bash
# Teste do cliff.toml: monta um repo temporário com commits conhecidos e confere as notas.
# Uso: GIT_CLIFF=caminho/git-cliff bash .github/scripts/test-changelog.sh
set -euo pipefail

cliff="${GIT_CLIFF:-git-cliff}"
root="$(cd "$(dirname "$0")/../.." && pwd)"
repo="$(mktemp -d)"
trap 'rm -rf "$repo"' EXIT

cd "$repo"
git init -q
git config user.name teste
git config user.email teste@example.com
git config commit.gpgsign false
cp "$root/cliff.toml" .
commit() { git commit -q --allow-empty -F -; }

commit <<<'chore(release): 0.1.0'
git tag v0.1.0
commit <<'EOF'
feat(ui)!: microfone na tela de assistir

Parágrafo explicativo que não é item e não entra nas notas.

- interface: fonte Geist embutida, temas claro/escuro,
  anel de foco e checkboxes padronizados
- rede: bitrate adaptativo
continuação sem recuo
- item de uma linha

BREAKING CHANGE: ui.json com formato novo
Refs: #42
EOF
commit <<<'fix: trava ao fechar'
commit <<<'atualiza coisas'
commit <<'EOF'
chore(release): 0.2.0

BREAKING CHANGE: não deve aparecer
EOF
git tag v0.2.0

out="$("$cliff" --current --strip all 2>/dev/null)"
printf '%s\n' "$out"

fail=0
has() { grep -qF -- "$1" <<<"$out" || { echo "FALTA: $1" >&2; fail=1; }; }
lacks() { ! grep -qF -- "$1" <<<"$out" || { echo "SOBRA: $1" >&2; fail=1; }; }

has '### Breaking Changes'
has '* ui.json com formato novo'
has '* **ui:** microfone na tela de assistir'
has '  - interface: fonte Geist embutida, temas claro/escuro, anel de foco e checkboxes padronizados'
has '  - rede: bitrate adaptativo continuação sem recuo'
has '  - item de uma linha'
has '### Fixed'
has '* trava ao fechar'
has '### Other Changes'
has '* atualiza coisas'
has '/compare/v0.1.0...v0.2.0'
lacks 'Parágrafo explicativo'
lacks 'Refs: #42'
lacks 'não deve aparecer'
lacks '0.2.0 ('
exit "$fail"
