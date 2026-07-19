# Skill Registry — bevy-libro-examples

## Project
- **Name**: bevy-libro-examples
- **Path**: /home/rubentxu/Proyectos/rust/bevy-libro-examples
- **Type**: Rust Cargo workspace (Bevy engine tutorial)

## Stack
- Language: Rust
- Edition: 2024
- Toolchain: 1.96.0 (from rust-toolchain.toml)
- Framework: Bevy 0.19.0 (workspace dependency pinned `=0.19.0`)
- Build tool: Cargo (resolver=3)
- CI: GitHub Actions (fmt/check/test/clippy)

## Detected Skills

### Rust Development
| Skill | Trigger | Location |
|-------|---------|----------|
| `rust-patterns` | Rust API design, trait bounds, error handling, concurrency | `/home/rubentxu/.config/opencode/skills/rust-patterns/SKILL.md` |
| `go-testing` | N/A (not a Go project) | — |

### SDD Phases
| Skill | Trigger | Location |
|-------|---------|----------|
| `sddk-init` | `sddk init` | `/home/rubentxu/.config/opencode/skills/sddk-init/SKILL.md` |
| `sddk-explore` | `sddk-explore`, `sddk-new` | `/home/rubentxu/.config/opencode/skills/sddk-explore/SKILL.md` |
| `sddk-propose` | `sddk-new`, `sddk-propose` | `/home/rubentxu/.config/opencode/skills/sddk-propose/SKILL.md` |
| `sddk-spec` | `sddk-spec` | `/home/rubentxu/.config/opencode/skills/sddk-spec/SKILL.md` |
| `sddk-design` | `sddk-design` | `/home/rubentxu/.config/opencode/skills/sddk-design/SKILL.md` |
| `sddk-tasks` | `sddk-tasks` | `/home/rubentxu/.config/opencode/skills/sddk-tasks/SKILL.md` |
| `sddk-apply` | `sddk-apply` | `/home/rubentxu/.config/opencode/skills/sddk-apply/SKILL.md` |
| `sddk-verify` | `sddk-verify`, `verify change` | `/home/rubentxu/.config/opencode/skills/sddk-verify/SKILL.md` |
| `sddk-archive` | `sddk-archive` | `/home/rubentxu/.config/opencode/skills/sddk-archive/SKILL.md` |
| `sddk-release` | `sddk-release` | `/home/rubentxu/.config/opencode/skills/sddk-release/SKILL.md` |

### Code Quality
| Skill | Trigger | Location |
|-------|---------|----------|
| `entropy-sdd` | SDD phase entropy analysis | `/home/rubentxu/.config/opencode/skills/entropy-sdd/SKILL.md` |
| `chronos-sdd` | Runtime behavior analysis | `/home/rubentxu/.config/opencode/skills/chronos-sdd/SKILL.md` |
| `cognicode-sdd` | Codebase analysis for SDD | `/home/rubentxu/.config/opencode/skills/cognicode-sdd/SKILL.md` |

### Bevy-specific
No Bevy-specific skills detected. Consider adding `bevy-engine` skill if available.

## Conventions

### Workspace Structure
```
bevy-libro-examples/
├── Cargo.toml           # workspace root (resolver=3, edition=2024)
├── rust-toolchain.toml  # 1.96.0 + clippy + rustfmt
├── .github/workflows/ci.yml
├── chapters/
│   └── chapter-04-first-contact/
│       ├── Cargo.toml
│       ├── src/lib.rs    # Position, Velocity, build_app, move_entities
│       └── examples/first_ecs.rs
└── crates/              # shared workspace crates (future)
```

### CI Commands (from ci.yml)
- Format: `cargo fmt --all --check`
- Type check: `cargo check --workspace --all-targets --locked`
- Test: `cargo test --workspace --locked`
- Lint: `cargo clippy --workspace --all-targets --locked -- -D warnings`

## Notes
- First test passes: `test tests::update_moves_the_spawned_entity`
- No coverage tool configured
- No TDD workflow established
- Strict TDD: **false**
