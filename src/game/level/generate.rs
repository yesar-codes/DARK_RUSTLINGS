use std::collections::VecDeque;

use rand::prelude::IndexedRandom;
use rand::rng;

use crate::game::level::data::LevelData;

pub fn generate_maze(width: usize, height: usize) -> Vec<Vec<char>> {
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

            // carve path
            grid[ny][nx] = '.';
            grid[(y + ny) / 2][(x + nx) / 2] = '.';

            stack.push((nx, ny));
        }
    }

    grid
}

pub fn place_specials(rows: &mut Vec<Vec<char>>) -> (usize, usize) {
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

    let &(sx, sy) = floor.choose(&mut rng).unwrap();
    rows[sy][sx] = 'S';

    let (ex, ey) = find_farthest_point((px, py), rows);
    rows[ey][ex] = 'E';

    (px, py)
}

fn find_farthest_point(
    start: (usize, usize),
    grid: &Vec<Vec<char>>,
) -> (usize, usize) {
    let h = grid.len();
    let w = grid[0].len();

    let mut visited = vec![vec![false; w]; h];
    let mut queue = VecDeque::new();

    queue.push_back((start, 0usize));
    visited[start.1][start.0] = true;

    let mut farthest = start;
    let mut max_dist = 0;

    while let Some(((x, y), dist)) = queue.pop_front() {
        if dist > max_dist {
            max_dist = dist;
            farthest = (x, y);
        }

        let dirs = [(0,1),(1,0),(0,-1),(-1,0)];

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

            if !visited[ny][nx] && grid[ny][nx] != '#' {
                visited[ny][nx] = true;
                queue.push_back(((nx, ny), dist + 1));
            }
        }
    }

    farthest
}

pub fn ascii_to_strings(rows: Vec<Vec<char>>) -> Vec<String> {
    rows.into_iter()
        .map(|r| r.into_iter().collect())
        .collect()
}

pub fn generate_level(name: &str, tile_width: f32, tile_height: f32) -> LevelData {
    let mut rows = generate_maze(15, 13);

    place_specials(&mut rows);

    LevelData {
        name: name.to_string(),
        tile_width,
        tile_height,
        rows: ascii_to_strings(rows),
    }
}