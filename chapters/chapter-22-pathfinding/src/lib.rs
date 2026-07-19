// Capítulo 22. Pathfinding — A* algorithm
// Pure algorithm, no GPU dependency, fully testable.
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Manhattan distance (good for 4-directional grid movement)
    pub fn manhattan_distance(&self, other: &GridPos) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// Euclidean distance (good for 8-directional or free movement)
    pub fn euclidean_distance(&self, other: &GridPos) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dy = (self.y - other.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn neighbors_4(&self) -> Vec<GridPos> {
        vec![
            GridPos::new(self.x + 1, self.y),
            GridPos::new(self.x - 1, self.y),
            GridPos::new(self.x, self.y + 1),
            GridPos::new(self.x, self.y - 1),
        ]
    }

    pub fn neighbors_8(&self) -> Vec<GridPos> {
        let mut result = Vec::with_capacity(8);
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                result.push(GridPos::new(self.x + dx, self.y + dy));
            }
        }
        result
    }
}

/// A simple grid-based map where some cells are walls
#[derive(Clone, Debug)]
pub struct GridMap {
    pub width: i32,
    pub height: i32,
    pub walls: HashSet<GridPos>,
}

impl GridMap {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            walls: HashSet::new(),
        }
    }

    pub fn add_wall(&mut self, pos: GridPos) {
        self.walls.insert(pos);
    }

    pub fn is_walkable(&self, pos: &GridPos) -> bool {
        pos.x >= 0
            && pos.y >= 0
            && pos.x < self.width
            && pos.y < self.height
            && !self.walls.contains(pos)
    }

    pub fn cost(&self, _from: &GridPos, to: &GridPos) -> i32 {
        // Uniform cost = 1 per step. Override for terrain costs.
        if self.is_walkable(to) {
            1
        } else {
            i32::MAX
        }
    }
}

/// Node in the A* open set, ordered by f-score (lowest first)
#[derive(Clone, Debug, Eq, PartialEq)]
struct AStarNode {
    f_score: i32,
    pos: GridPos,
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for BinaryHeap (min-heap behavior)
        other.f_score.cmp(&self.f_score)
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A* pathfinding algorithm.
/// Returns the path from start to goal (inclusive), or None if no path exists.
pub fn a_star(map: &GridMap, start: GridPos, goal: GridPos) -> Option<Vec<GridPos>> {
    if !map.is_walkable(&start) || !map.is_walkable(&goal) {
        return None;
    }
    if start == goal {
        return Some(vec![start]);
    }

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<GridPos, GridPos> = HashMap::new();
    let mut g_score: HashMap<GridPos, i32> = HashMap::new();
    let mut closed_set: HashSet<GridPos> = HashSet::new();

    g_score.insert(start, 0);
    open_set.push(AStarNode {
        f_score: start.manhattan_distance(&goal),
        pos: start,
    });

    while let Some(current) = open_set.pop() {
        if closed_set.contains(&current.pos) {
            continue;
        }
        closed_set.insert(current.pos);

        if current.pos == goal {
            // Reconstruct path
            let mut path = vec![goal];
            let mut node = goal;
            while let Some(&prev) = came_from.get(&node) {
                path.push(prev);
                node = prev;
            }
            path.reverse();
            return Some(path);
        }

        for neighbor in current.pos.neighbors_4() {
            if !map.is_walkable(&neighbor) || closed_set.contains(&neighbor) {
                continue;
            }

            let tentative_g = g_score[&current.pos] + map.cost(&current.pos, &neighbor);

            let known_g = g_score.get(&neighbor).copied().unwrap_or(i32::MAX);
            if tentative_g < known_g {
                came_from.insert(neighbor, current.pos);
                g_score.insert(neighbor, tentative_g);
                let f = tentative_g + neighbor.manhattan_distance(&goal);
                open_set.push(AStarNode {
                    f_score: f,
                    pos: neighbor,
                });
            }
        }
    }

    None // No path found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manhattan_distance_basic() {
        let a = GridPos::new(0, 0);
        let b = GridPos::new(3, 4);
        assert_eq!(a.manhattan_distance(&b), 7);
    }

    #[test]
    fn manhattan_distance_zero() {
        let a = GridPos::new(5, 5);
        assert_eq!(a.manhattan_distance(&a), 0);
    }

    #[test]
    fn neighbors_4_count() {
        let pos = GridPos::new(5, 5);
        assert_eq!(pos.neighbors_4().len(), 4);
    }

    #[test]
    fn neighbors_8_count() {
        let pos = GridPos::new(5, 5);
        assert_eq!(pos.neighbors_8().len(), 8);
    }

    #[test]
    fn a_star_straight_line() {
        let map = GridMap::new(10, 10);
        let start = GridPos::new(0, 0);
        let goal = GridPos::new(5, 0);

        let path = a_star(&map, start, goal).expect("Path should exist");

        assert_eq!(path.first(), Some(&start));
        assert_eq!(path.last(), Some(&goal));
        assert_eq!(path.len(), 6); // 0,0 -> 1,0 -> ... -> 5,0
    }

    #[test]
    fn a_star_around_wall() {
        let mut map = GridMap::new(10, 10);
        // Wall blocking direct path
        for y in 0..9 {
            map.add_wall(GridPos::new(3, y));
        }

        let start = GridPos::new(0, 0);
        let goal = GridPos::new(7, 0);

        let path = a_star(&map, start, goal).expect("Path should exist around wall");

        assert_eq!(path.first(), Some(&start));
        assert_eq!(path.last(), Some(&goal));

        // Path should not go through walls
        for pos in &path {
            assert!(!map.walls.contains(pos), "Path goes through wall at {:?}", pos);
        }
    }

    #[test]
    fn a_star_no_path() {
        let mut map = GridMap::new(5, 5);
        // Complete wall blocking
        for y in 0..5 {
            map.add_wall(GridPos::new(2, y));
        }

        let start = GridPos::new(0, 0);
        let goal = GridPos::new(4, 0);

        let path = a_star(&map, start, goal);
        assert!(path.is_none(), "Should not find a path through complete wall");
    }

    #[test]
    fn a_star_start_equals_goal() {
        let map = GridMap::new(5, 5);
        let start = GridPos::new(2, 2);

        let path = a_star(&map, start, start).expect("Path should exist");
        assert_eq!(path, vec![start]);
    }

    #[test]
    fn a_star_optimal_path_length() {
        let map = GridMap::new(10, 10);
        let start = GridPos::new(0, 0);
        let goal = GridPos::new(3, 4);

        let path = a_star(&map, start, goal).expect("Path should exist");

        // Manhattan distance = 7, path cost should be 7 steps = 8 nodes
        assert_eq!(path.len(), 8);
    }

    #[test]
    fn grid_map_walkable() {
        let mut map = GridMap::new(5, 5);
        map.add_wall(GridPos::new(2, 2));

        assert!(map.is_walkable(&GridPos::new(0, 0)));
        assert!(map.is_walkable(&GridPos::new(4, 4)));
        assert!(!map.is_walkable(&GridPos::new(2, 2)));
        assert!(!map.is_walkable(&GridPos::new(-1, 0)), "Out of bounds");
        assert!(!map.is_walkable(&GridPos::new(5, 0)), "Out of bounds");
    }

    #[test]
    fn euclidean_distance() {
        let a = GridPos::new(0, 0);
        let b = GridPos::new(3, 4);
        let dist = a.euclidean_distance(&b);
        assert!((dist - 5.0).abs() < 0.001, "3-4-5 triangle distance should be 5.0");
    }
}
