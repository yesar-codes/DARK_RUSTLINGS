use std::collections::VecDeque;
use rand::prelude::IndexedRandom;
use rand::rng;

use crate::game::level::data::LevelData;

fn generate_maze(width: usize, height: usize) -> Vec<Vec<char>> {
    let mut grid = vec![vec!['#'; width]; height];

    let mut rng = rng();

    let mut stack = vec![(1usize, 1usize)];
    grid[1][1] = '.';

    while let Some((x, y)) = stack.pop() {
        let mut neighbors = Vec::new();

        if y > 2 && grid[y - 2][x] == '#' {
            neighbors.push((x, y - 2));
        }
        if x < width - 3 && grid[y][x + 2] == '#' {
            neighbors.push((x + 2, y));
        }
        if y < height - 3 && grid[y + 2][x] == '#' {
            neighbors.push((x, y + 2));
        }
        if x > 2 && grid[y][x - 2] == '#' {
            neighbors.push((x - 2, y));
        }

        if !neighbors.is_empty() {
            stack.push((x, y));

            let &(nx, ny) = neighbors.choose(&mut rng).unwrap();

            grid[ny][nx] = '.';
            grid[(y + ny) / 2][(x + nx) / 2] = '.';

            stack.push((nx, ny));
        }
    }

    grid
}

fn place_specials(rows: &mut Vec<Vec<char>>) -> (usize, usize) {
    let mut rng = rng();
    let mut floor = Vec::new();

    for y in 0..rows.len() {
        for x in 0..rows[y].len() {
            if rows[y][x] == '.' {
                floor.push((x, y));
            }
        }
    }

    let &(px, py) = floor.choose(&mut rng).unwrap();
    rows[py][px] = 'P';

    let switch_candidates: Vec<(usize, usize)> = floor
        .iter()
        .copied()
        .filter(|&(x, y)| x != px || y != py)
        .collect();
    if let Some(&(sx, sy)) = switch_candidates.choose(&mut rng) {
        rows[sy][sx] = 'S';
    }

    let (ex, ey) = select_border_exit((px, py), rows);
    rows[ey][ex] = 'E';

    let mut powerup_candidates = Vec::new();
    for y in 0..rows.len() {
        for x in 0..rows[y].len() {
            if rows[y][x] == '.' {
                powerup_candidates.push((x, y));
            }
        }
    }

    if let Some(&(gx, gy)) = powerup_candidates.choose(&mut rng) {
        rows[gy][gx] = 'G';
        powerup_candidates.retain(|&(x, y)| x != gx || y != gy);
    }

    if let Some(&(rx, ry)) = powerup_candidates.choose(&mut rng) {
        rows[ry][rx] = 'R';
    }

    (px, py)
}

fn ascii_to_strings(rows: Vec<Vec<char>>) -> Vec<String> {
    rows.into_iter()
        .map(|r| r.into_iter().collect())
        .collect()
}

pub(super) fn generate_level(name: &str, tile_width: f32, tile_height: f32, level_index: usize, premade_count: usize) -> LevelData {
    const BASE_WIDTH: usize = 13;
    const BASE_HEIGHT: usize = 11;

    let mut width_difficulty = level_index - premade_count;
    let mut height_difficulty = level_index - premade_count;
    if (level_index + BASE_WIDTH) % 2 == 0 {
        width_difficulty -= 1;
        height_difficulty -= 1;
    }
    if level_index % 4 == 1 {
        height_difficulty += 2;
    } else if level_index % 4 == 3 {
        width_difficulty += 2;
    }
    let mut rows = generate_maze(BASE_WIDTH + width_difficulty, BASE_HEIGHT + height_difficulty);

    place_specials(&mut rows);

    LevelData {
        name: name.to_string(),
        tile_width,
        tile_height,
        rows: ascii_to_strings(rows),
    }
}

fn select_border_exit(start: (usize, usize), rows: &mut Vec<Vec<char>>) -> (usize, usize) {
    let h = rows.len();
    let w = rows[0].len();

    let mut visited = vec![vec![false; w]; h];
    let mut queue = VecDeque::new();
    let mut distance = vec![vec![usize::MAX; w]; h];

    queue.push_back((start, 0usize));
    visited[start.1][start.0] = true;
    distance[start.1][start.0] = 0;

    while let Some(((x, y), dist)) = queue.pop_front() {
        let dirs = [(0, 1), (1, 0), (0, -1), (-1, 0)];

        for (dx, dy) in dirs {
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if nx < 0 || ny < 0 {
                continue;
            }

            let nx = nx as usize;
            let ny = ny as usize;
            if nx >= w || ny >= h {
                continue;
            }

            if !visited[ny][nx] && rows[ny][nx] != '#' {
                visited[ny][nx] = true;
                distance[ny][nx] = dist + 1;
                queue.push_back(((nx, ny), dist + 1));
            }
        }
    }

    let mut best: Option<(usize, usize, usize, usize, usize)> = None;

    for y in 1..(h - 1) {
        if rows[y][1] != '#' && distance[y][1] != usize::MAX {
            let d = distance[y][1];
            if best.map_or(true, |candidate| d > candidate.4) {
                best = Some((0, y, 1, y, d));
            }
        }
        if rows[y][w - 2] != '#' && distance[y][w - 2] != usize::MAX {
            let d = distance[y][w - 2];
            if best.map_or(true, |candidate| d > candidate.4) {
                best = Some((w - 1, y, w - 2, y, d));
            }
        }
    }

    for x in 1..(w - 1) {
        if rows[1][x] != '#' && distance[1][x] != usize::MAX {
            let d = distance[1][x];
            if best.map_or(true, |candidate| d > candidate.4) {
                best = Some((x, 0, x, 1, d));
            }
        }
        if rows[h - 2][x] != '#' && distance[h - 2][x] != usize::MAX {
            let d = distance[h - 2][x];
            if best.map_or(true, |candidate| d > candidate.4) {
                best = Some((x, h - 1, x, h - 2, d));
            }
        }
    }

    if let Some((exit_x, exit_y, inner_x, inner_y, _)) = best {
        rows[inner_y][inner_x] = '.';
        return (exit_x, exit_y);
    }

    let fallback_inner_x = start.0.clamp(1, w - 2);
    rows[1][fallback_inner_x] = '.';
    (fallback_inner_x, 0)
}
