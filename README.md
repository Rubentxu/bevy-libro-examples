# Bevy 2D/ECS — Repo de Ejemplos

Código de ejemplo del libro **"Patrones 2D y ECS con Bevy 0.19"**.

Todos los ejemplos están verificados contra la API real de Bevy 0.19 mediante tests automatizados.

## Estructura

```
chapters/
├── chapter-04-first-contact/       # Entities, Components, Systems (10 tests)
├── chapter-05-queries-resources/   # Query, ParamSet, Resources (14 tests)
├── chapter-06-events/              # Message API, combat (7 tests)
├── chapter-07-schedules/           # System sets, run conditions (7 tests)
├── chapter-08-required-components/ # #[require(...)] (5 tests)
├── chapter-09-hooks/               # Component hooks, HookContext (4 tests)
├── chapter-10-observers/           # On<E> API, triggers (4 tests)
├── chapter-11-relationships/       # ChildOf, custom relations (7 tests)
├── chapter-12-scenes/              # Scene building, hierarchies (6 tests)
├── chapter-13-design-rules/        # Hooks vs observers vs systems (6 tests)
├── chapter-14-rendering/           # Rendering (stub — requires GPU)
├── ...
├── chapter-21-ai/                  # FSM, utility AI, behavior trees (11 tests)
├── chapter-21c-procedural-gen/     # RNG, cave/dungeon generation (12 tests)
├── chapter-22-pathfinding/         # A* algorithm (12 tests)
├── chapter-24-persistence/         # Serialization, checkpoints (7 tests)
├── ...
└── chapter-28b-deployment/         # Build config, deployment (stub)
```

## Uso

```bash
# Ejecutar todos los tests
cargo test

# Ejecutar tests de un capítulo específico
cargo test -p bevy-book-chapter-10

# Ejecutar un test específico
cargo test -p bevy-book-chapter-22 -- a_star_around_wall
```

## Resultado

- **38 crates** cubriendo todos los capítulos (04–28b)
- **197 tests** pasando
- **Bevy 0.19.0** (`default-features = false` para tests headless)

## Capítulos con tests sustantivos

Los capítulos marcados con tests usan lógica pura (sin GPU):

| Capítulo | Tests | Conceptos clave |
|----------|-------|-----------------|
| 04 | 10 | ECS básico, spawn, query |
| 05 | 14 | Query, ParamSet, Resources |
| 06 | 7 | Message API, combat |
| 07 | 7 | System sets, run conditions |
| 08 | 5 | `#[require()]` |
| 09 | 4 | Component hooks, HookContext |
| 10 | 4 | `On<E>` API (Bevy 0.19) |
| 11 | 7 | ChildOf, custom Likes/LikedBy |
| 12 | 6 | Scene building, hierarchies |
| 13 | 6 | Hooks vs observers vs systems |
| 17 | 13 | AABB/circle collision, kinematic bodies, coyote time |
| 19 | 10 | Animation clips, frame cycling, blend trees |
| 20 | 12 | Screen shake, hit pause, knockback, cooldowns |
| 21 | 11 | FSM, utility AI, behavior trees |
| 21c | 12 | RNG, cave/dungeon generation |
| 22 | 12 | A* algorithm, grid maps |
| 24 | 7 | Serialization, checkpoints |
| 25b | 11 | Rollback: state snapshots, input buffers, re-simulation |
| 26 | 12 | Frame metrics, budgets, percentiles, profiling |
| 27b | 10 | SemVer parsing, change types, compatibility |
| 28b | 13 | Platforms, build profiles, release checklist |

Los capítulos de rendering/GPU puro (14–16, 18, 23, 25, 27, 28) son stubs
que compilan pero requieren `DefaultPlugins` y GPU para tests funcionales.

## Errata

Las correcciones aplicadas al libro HTML están documentadas en
`static/libros/bevy-2d-ecs/errata.md` del blog.
