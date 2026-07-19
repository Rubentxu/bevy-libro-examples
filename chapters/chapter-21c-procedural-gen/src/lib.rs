// Capítulo 21C. Procedural Generation — RNG, noise, dungeon generation
// Pure algorithms, fully testable.
use std::collections::HashSet;

/// Simple seeded PRNG (Linear Congruential Generator)
/// Same seed = same output. Deterministic for reproducible worlds.
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u32(&mut self) -> u32 {
        // LCG parameters (same as glibc)
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    pub fn next_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }

    pub fn range(&mut self, min: i32, max: i32) -> i32 {
        min + (self.next_u32() as i32).rem_euclid(max - min)
    }

    pub fn chance(&mut self, probability: f32) -> bool {
        self.next_f32() < probability
    }
}

/// Cellular automata for cave generation
pub fn generate_cave(width: usize, height: usize, rng: &mut Rng, wall_prob: f32) -> Vec<Vec<bool>> {
    let mut grid = vec![vec![false; width]; height];

    // Random initial fill
    for y in 0..height {
        for x in 0..width {
            if x == 0 || y == 0 || x == width - 1 || y == height - 1 {
                grid[y][x] = true; // Border walls
            } else {
                grid[y][x] = rng.chance(wall_prob);
            }
        }
    }

    // Smooth with cellular automata rules
    for _ in 0..5 {
        grid = smooth_cave(&grid);
    }

    grid
}

fn smooth_cave(grid: &[Vec<bool>]) -> Vec<Vec<bool>> {
    let height = grid.len();
    let width = grid[0].len();
    let mut new_grid = vec![vec![false; width]; height];

    for y in 0..height {
        for x in 0..width {
            let neighbors = count_wall_neighbors(grid, x, y);
            new_grid[y][x] = neighbors >= 5;
        }
    }

    new_grid
}

fn count_wall_neighbors(grid: &[Vec<bool>], x: usize, y: usize) -> usize {
    let mut count = 0;
    let height = grid.len();
    let width = grid[0].len();

    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                count += 1; // Out of bounds = wall
            } else if grid[ny as usize][nx as usize] {
                count += 1;
            }
        }
    }

    count
}

/// Simple room placement for dungeon generation
#[derive(Clone, Debug, PartialEq)]
pub struct Room {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Room {
    pub fn center(&self) -> (usize, usize) {
        (self.x + self.width / 2, self.y + self.height / 2)
    }

    pub fn intersects(&self, other: &Room) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}

pub fn generate_dungeon(
    map_width: usize,
    map_height: usize,
    max_rooms: usize,
    room_min_size: usize,
    room_max_size: usize,
    rng: &mut Rng,
) -> (Vec<Room>, Vec<Vec<bool>>) {
    let mut rooms = Vec::new();
    let mut grid = vec![vec![true; map_width]; map_height]; // All walls

    for _ in 0..max_rooms {
        let w = rng.range(room_min_size as i32, room_max_size as i32) as usize;
        let h = rng.range(room_min_size as i32, room_max_size as i32) as usize;
        let x = rng.range(1, (map_width - w - 1) as i32) as usize;
        let y = rng.range(1, (map_height - h - 1) as i32) as usize;

        let new_room = Room { x, y, width: w, height: h };

        let intersects = rooms.iter().any(|r: &Room| r.intersects(&new_room));
        if !intersects {
            // Carve room
            for ry in y..y + h {
                for rx in x..x + w {
                    grid[ry][rx] = false;
                }
            }

            // Connect to previous room with corridor
            if !rooms.is_empty() {
                let (prev_cx, prev_cy) = rooms.last().unwrap().center();
                let (new_cx, new_cy) = new_room.center();

                // Horizontal corridor
                for cx in prev_cx.min(new_cx)..=prev_cx.max(new_cx) {
                    grid[prev_cy][cx] = false;
                }
                // Vertical corridor
                for cy in prev_cy.min(new_cy)..=prev_cy.max(new_cy) {
                    grid[cy][new_cx] = false;
                }
            }

            rooms.push(new_room);
        }
    }

    (rooms, grid)
}

/// Hash-based procedural naming
pub fn hash_name(seed: u64, syllables: &[&str]) -> String {
    let mut rng = Rng::new(seed);
    let count = rng.range(2, 4);
    (0..count)
        .map(|_| syllables[rng.range(0, syllables.len() as i32) as usize])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_deterministic_same_seed() {
        let mut r1 = Rng::new(42);
        let mut r2 = Rng::new(42);

        for _ in 0..100 {
            assert_eq!(r1.next_u32(), r2.next_u32());
        }
    }

    #[test]
    fn rng_different_seeds_diverge() {
        let mut r1 = Rng::new(42);
        let mut r2 = Rng::new(43);
        assert_ne!(r1.next_u32(), r2.next_u32());
    }

    #[test]
    fn rng_range_within_bounds() {
        let mut rng = Rng::new(123);
        for _ in 0..1000 {
            let val = rng.range(5, 10);
            assert!(val >= 5 && val < 10, "Value {} out of range [5, 10)", val);
        }
    }

    #[test]
    fn rng_chance_probability() {
        let mut rng = Rng::new(999);
        let mut hits = 0;
        let trials = 10000;
        for _ in 0..trials {
            if rng.chance(0.3) {
                hits += 1;
            }
        }
        let ratio = hits as f32 / trials as f32;
        assert!((ratio - 0.3).abs() < 0.05, "Ratio {} should be ~0.3", ratio);
    }

    #[test]
    fn cave_generation_has_walls_and_open() {
        let mut rng = Rng::new(42);
        let cave = generate_cave(20, 20, &mut rng, 0.45);

        let walls: usize = cave.iter().map(|row| row.iter().filter(|&&c| c).count()).sum();
        let open = 400 - walls;

        assert!(walls > 0, "Should have walls");
        assert!(open > 0, "Should have open space");
    }

    #[test]
    fn cave_borders_are_walls() {
        let mut rng = Rng::new(42);
        let cave = generate_cave(15, 15, &mut rng, 0.4);

        // All border cells should be walls
        for x in 0..15 {
            assert!(cave[0][x], "Top border should be wall");
            assert!(cave[14][x], "Bottom border should be wall");
        }
        for y in 0..15 {
            assert!(cave[y][0], "Left border should be wall");
            assert!(cave[y][14], "Right border should be wall");
        }
    }

    #[test]
    fn room_intersects() {
        let r1 = Room { x: 0, y: 0, width: 5, height: 5 };
        let r2 = Room { x: 3, y: 3, width: 5, height: 5 };
        let r3 = Room { x: 10, y: 10, width: 3, height: 3 };

        assert!(r1.intersects(&r2));
        assert!(!r1.intersects(&r3));
    }

    #[test]
    fn room_center() {
        let room = Room { x: 10, y: 20, width: 10, height: 10 };
        let (cx, cy) = room.center();
        assert_eq!(cx, 15);
        assert_eq!(cy, 25);
    }

    #[test]
    fn dungeon_has_rooms() {
        let mut rng = Rng::new(42);
        let (rooms, grid) = generate_dungeon(50, 50, 10, 4, 10, &mut rng);

        assert!(!rooms.is_empty(), "Should have at least one room");
        assert!(rooms.len() <= 10);

        // Check grid has open cells where rooms are
        for room in &rooms {
            assert!(!grid[room.y + 1][room.x + 1], "Room interior should be open");
        }
    }

    #[test]
    fn dungeon_rooms_dont_overlap() {
        let mut rng = Rng::new(42);
        let (rooms, _) = generate_dungeon(60, 60, 15, 4, 8, &mut rng);

        for i in 0..rooms.len() {
            for j in (i + 1)..rooms.len() {
                assert!(!rooms[i].intersects(&rooms[j]), "Rooms overlap");
            }
        }
    }

    #[test]
    fn hash_name_deterministic() {
        let syllables = vec!["ka", "ru", "shi", "ma", "to"];
        let name1 = hash_name(42, &syllables);
        let name2 = hash_name(42, &syllables);
        assert_eq!(name1, name2);
    }

    #[test]
    fn hash_name_different_seeds() {
        let syllables = vec!["ka", "ru", "shi"];
        let name1 = hash_name(42, &syllables);
        let name2 = hash_name(100, &syllables);
        assert_ne!(name1, name2);
    }
}
