## Dark Rustlings

This project starts in a dark isometric labyrinth and now includes level progression.

### Run

```powershell
cargo run
```

### Structure

- `src/game/` contains gameplay setup modules.
- `src/game/level/` contains level data types, file loading, and spawning logic.
- `resources/levels/` contains `.ron` level files.

### Level Format

Level files live in `resources/levels/` (currently `level_01.ron` and `level_02.ron`).

- `#` = wall tile
- `.` = floor/walkable tile
- `S` = light switch tile
- `E` = exit tile

### Rules

- You have 30 seconds to find the switch (`S`) or reach the exit (`E`).
- Exiting advances to the next level.
- Exiting on the final level shows a win screen.
- If the timer reaches zero before success, a game-over screen is shown.
- The countdown is displayed in the upper-right corner.
