# AI Coding Agent Skill/Plugin Storage Paths

Status: Current as of March 2026

A comprehensive reference of where each major AI coding agent stores its skills, custom instructions, and rules files.

---

## 1. Claude Code (Anthropic)

**Skills directory concept**: Yes, first-class support.

| Level | Path | Format |
|-------|------|--------|
| Project skills | `<project>/.claude/skills/<skill-name>/SKILL.md` | Markdown with YAML frontmatter |
| Personal skills | `~/.claude/skills/<skill-name>/SKILL.md` | Markdown with YAML frontmatter |
| Enterprise skills | Managed via admin | Markdown with YAML frontmatter |
| Project instructions | `<project>/CLAUDE.md` | Markdown |
| Personal instructions | `~/.claude/CLAUDE.md` | Markdown |
| Project-scoped personal | `~/.claude/projects/<project-path>/CLAUDE.md` | Markdown |
| Slash commands | `<project>/.claude/commands/<name>.md` | Markdown |

**Notes**: Skills take precedence over commands if names collide. Nested `CLAUDE.md` files in subdirectories are auto-discovered. Priority order: enterprise > personal > project.

---

## 2. Amp (Sourcegraph)

**Skills directory concept**: No dedicated skills directory. Uses `AGENTS.md` files and custom toolboxes.

| Level | Path | Format |
|-------|------|--------|
| Project instructions | `<project>/AGENTS.md` | Markdown |
| Subdirectory instructions | `<subdir>/AGENTS.md` | Markdown |
| Custom tools (toolbox) | `~/.amp-tools/` (set via `AMP_TOOLBOX` env var) | Script files |
| Custom slash commands | `<project>/.agents/commands/<name>.md` | Markdown |
| Global settings | `~/.config/amp/settings.json` | JSON |
| Workspace settings | `<project>/.amp/settings.json` | JSON |

**Notes**: Amp reads `AGENTS.md` at project root for build/test steps and conventions. Multiple `AGENTS.md` files can exist in subdirectories for large codebases. MCP server configuration goes in the settings JSON files.

---

## 3. Goose (Block)

**Skills directory concept**: No file-based skills directory. Has built-in toggleable agent skills and MCP extensions.

| Level | Path | Format |
|-------|------|--------|
| Project hints | `<project>/.goosehints` | Plain text / Markdown |
| User hints | `~/.goosehints` | Plain text / Markdown |
| Subdirectory hints | `<subdir>/.goosehints` | Plain text / Markdown |
| Configuration | `~/.config/goose/config.yaml` | YAML |

**Notes**: `.goosehints` provides persistent instructions loaded every session. Built-in agent skills (e.g., "Save Files") are toggled via the UI, not files. Extensibility is primarily through MCP servers configured in `config.yaml`.

---

## 4. Aider

**Skills directory concept**: No. Uses conventions files and config.

| Level | Path | Format |
|-------|------|--------|
| Conventions (default) | `<project>/CONVENTIONS.md` | Markdown |
| Conventions (custom) | Any file via `--read <file>` or `--conventions-file <file>` | Markdown |
| Configuration | `~/.aider.conf.yml` | YAML |
| Project configuration | `<project>/.aider.conf.yml` | YAML |

**Notes**: Conventions files are loaded via `--read CONVENTIONS.md` on the command line, or configured in `.aider.conf.yml` with `read: CONVENTIONS.md`. Multiple conventions files can be specified as an array. AiderDesk (a related GUI tool) adds `<project>/.aider-desk/skills/` and `~/.aider-desk/skills/` but this is not part of core Aider.

---

## 5. Cline (VS Code Extension)

**Skills directory concept**: Yes, via `.clinerules/` directory.

| Level | Path | Format |
|-------|------|--------|
| Rules file (single) | `<project>/.clinerules` | Plain text / Markdown |
| Rules directory | `<project>/.clinerules/*.md` | Markdown files |
| Global instructions | VS Code settings UI | Plain text |

**Notes**: Supports either a single `.clinerules` file OR a `.clinerules/` directory with multiple `.md` files. Numeric prefixes on filenames control ordering (e.g., `01-style.md`, `02-testing.md`). Files in the directory are combined into a unified rule set. Version-controllable via git.

---

## 6. Continue (VS Code / JetBrains Extension)

**Skills directory concept**: Yes, via `.continue/rules/` directory.

| Level | Path | Format |
|-------|------|--------|
| Project rules directory | `<project>/.continue/rules/*.md` or `*.yaml` | Markdown or YAML |
| Project rules file | `<project>/.continuerules` | Markdown |
| User config | `~/.continue/config.yaml` | YAML |
| Global data directory | `~/.continue/` | Various |

**Notes**: Rules support `globs` and `regex` properties in YAML frontmatter to scope when they apply. Markdown format is recommended over YAML. In multi-folder VS Code workspaces, each folder can have its own `.continuerules` file. Rules defined in `config.yaml` use the `rules:` array property (replaces the old `systemMessage`).

---

## 7. Cursor

**Skills directory concept**: Yes, via `.cursor/rules/` directory.

| Level | Path | Format |
|-------|------|--------|
| Project rules (current) | `<project>/.cursor/rules/*.mdc` | MDC (Markdown with YAML frontmatter) |
| Legacy project rules | `<project>/.cursorrules` | Plain text |
| Subdirectory rules | `<subdir>/.cursor/rules/*.mdc` | MDC |
| Global rules | Cursor Settings > General > Rules for AI | Plain text (UI) |

**Notes**: `.mdc` files have YAML frontmatter with `description`, `globs`, and `alwaysApply` properties. The `.cursorrules` file is legacy and will eventually be removed; migrate to `.cursor/rules/` directory. Filenames use kebab-case. Subdirectories can have their own `.cursor/rules/` scoped to that folder.

---

## 8. Windsurf (Codeium)

**Skills directory concept**: Yes, via `.windsurf/rules/` directory.

| Level | Path | Format |
|-------|------|--------|
| Global rules | `~/.codeium/windsurf/memories/global_rules.md` | Markdown |
| Workspace rules (current) | `<project>/.windsurf/rules/*.md` | Markdown |
| Legacy workspace rules | `<project>/.windsurfrules` | Plain text |

**Notes**: Global rules are limited to 6,000 characters. Individual workspace rule files are also limited (6,000 chars each, 12,000 total combined). Windsurf auto-discovers `.windsurf/rules/` in the workspace and parent directories up to the git root. Rules are re-read on workspace open, not on hot-reload.

---

## 9. GitHub Copilot

**Skills directory concept**: Yes, via `.github/instructions/` directory.

| Level | Path | Format |
|-------|------|--------|
| Repository-wide instructions | `<project>/.github/copilot-instructions.md` | Markdown |
| Path-specific instructions | `<project>/.github/instructions/<name>.instructions.md` | Markdown with YAML frontmatter |
| User-level instructions | VS Code settings / GitHub.com settings | Plain text |

**Notes**: Path-specific `.instructions.md` files use YAML frontmatter with `applyTo` glob patterns to scope instructions to matching files. Both repository-wide and path-specific instructions are combined when both match. Subdirectories within `.github/instructions/` are supported for organization.

---

## 10. Roo Code

**Skills directory concept**: Yes, via `.roo/rules/` directory with mode-specific subdirectories.

| Level | Path | Format |
|-------|------|--------|
| Global rules | `~/.roo/rules/*.md` | Markdown / text |
| Project rules | `<project>/.roo/rules/*.md` | Markdown / text |
| Mode-specific rules | `<project>/.roo/rules-<mode-slug>/*.md` | Markdown / text |
| Legacy rules file | `<project>/.roorules` | Plain text |
| Legacy mode-specific | `<project>/.roorules-<mode-slug>` | Plain text |

**Notes**: Files are read recursively (including subdirectories) and appended to the system prompt in alphabetical order by filename. Workspace rules take precedence over global rules when they conflict. Modes include built-in modes and custom modes (e.g., `.roo/rules-docs-writer/`). Falls back to `.roorules` file if `.roo/rules/` directory is empty or missing.

---

## Cross-Agent Compatibility Summary

| Feature | Claude Code | Amp | Goose | Aider | Cline | Continue | Cursor | Windsurf | Copilot | Roo Code |
|---------|------------|-----|-------|-------|-------|----------|--------|----------|---------|----------|
| Skills directory | `.claude/skills/` | -- | -- | -- | `.clinerules/` | `.continue/rules/` | `.cursor/rules/` | `.windsurf/rules/` | `.github/instructions/` | `.roo/rules/` |
| Instruction file | `CLAUDE.md` | `AGENTS.md` | `.goosehints` | `CONVENTIONS.md` | `.clinerules` | `.continuerules` | `.cursorrules` | `.windsurfrules` | `copilot-instructions.md` | `.roorules` |
| Global config dir | `~/.claude/` | `~/.config/amp/` | `~/.config/goose/` | `~/.aider.conf.yml` | VS Code settings | `~/.continue/` | Cursor settings | `~/.codeium/windsurf/` | VS Code/GH settings | `~/.roo/` |
| Scoped/glob rules | -- | -- | -- | -- | -- | Yes (globs) | Yes (globs) | -- | Yes (applyTo) | Yes (per-mode) |
| File format | Markdown+YAML FM | Markdown | Text/MD | Markdown | Markdown | MD or YAML | MDC (MD+YAML FM) | Markdown | MD+YAML FM | Markdown |

---

## Emerging Standard: AGENTS.md

There is an emerging cross-agent convention around `AGENTS.md` as a universal agent instructions file (similar to how `README.md` became standard). Amp uses it natively, and there are community proposals for Aider and others to adopt it. This is worth watching as a potential universal standard.
