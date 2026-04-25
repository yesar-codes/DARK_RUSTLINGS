## Dark Rustlings

This project now starts with a single isometric top-down labyrinth level.

### Run

```powershell
cargo run
```

### Structure

- `src/game/` contains gameplay setup modules.
- `src/game/level/` contains level data types, file loading, and spawning logic.
- `resources/levels/` contains `.ron` level files.

### Level Format

The first level file is `resources/levels/level_01.ron`.

- `#` = wall tile
- `.` = floor/walkable tile
