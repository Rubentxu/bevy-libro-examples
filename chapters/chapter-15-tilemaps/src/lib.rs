// Capítulo 15. Tilemaps — Coordinate math, chunk management
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TileCoord {
    pub x: i32,
    pub y: i32,
}

impl TileCoord {
    pub fn new(x: i32, y: i32) -> Self { Self { x, y } }

    pub fn to_world(&self, tile_size: f32) -> (f32, f32) {
        (self.x as f32 * tile_size, self.y as f32 * tile_size)
    }

    pub fn from_world(wx: f32, wy: f32, tile_size: f32) -> Self {
        Self {
            x: (wx / tile_size).floor() as i32,
            y: (wy / tile_size).floor() as i32,
        }
    }

    pub fn neighbors_4(&self) -> Vec<TileCoord> {
        vec![
            TileCoord::new(self.x + 1, self.y),
            TileCoord::new(self.x - 1, self.y),
            TileCoord::new(self.x, self.y + 1),
            TileCoord::new(self.x, self.y - 1),
        ]
    }

    pub fn neighbors_8(&self) -> Vec<TileCoord> {
        let mut result = Vec::with_capacity(8);
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx != 0 || dy != 0 {
                    result.push(TileCoord::new(self.x + dx, self.y + dy));
                }
            }
        }
        result
    }

    pub fn chebyshev_distance(&self, other: &TileCoord) -> i32 {
        (self.x - other.x).abs().max((self.y - other.y).abs())
    }
}

/// Chunked tilemap for efficient storage of large maps
pub struct ChunkedTilemap {
    pub chunk_size: i32,
    pub tiles: std::collections::HashMap<(i32, i32), Vec<u32>>,
}

impl ChunkedTilemap {
    pub fn new(chunk_size: i32) -> Self {
        Self {
            chunk_size,
            tiles: std::collections::HashMap::new(),
        }
    }

    fn chunk_key(&self, coord: &TileCoord) -> (i32, i32) {
        let cx = coord.x.div_euclid(self.chunk_size);
        let cy = coord.y.div_euclid(self.chunk_size);
        (cx, cy)
    }

    fn local_index(&self, coord: &TileCoord) -> usize {
        let lx = coord.x.rem_euclid(self.chunk_size) as usize;
        let ly = coord.y.rem_euclid(self.chunk_size) as usize;
        ly * self.chunk_size as usize + lx
    }

    pub fn set_tile(&mut self, coord: TileCoord, tile_id: u32) {
        let key = self.chunk_key(&coord);
        let idx = self.local_index(&coord);
        let chunk_size_sq = (self.chunk_size * self.chunk_size) as usize;

        let chunk = self.tiles.entry(key).or_insert_with(|| vec![0; chunk_size_sq]);
        chunk[idx] = tile_id;
    }

    pub fn get_tile(&self, coord: &TileCoord) -> u32 {
        let key = self.chunk_key(coord);
        let idx = self.local_index(coord);

        match self.tiles.get(&key) {
            Some(chunk) => chunk[idx],
            None => 0,
        }
    }

    pub fn chunk_count(&self) -> usize {
        self.tiles.len()
    }
}

/// Isometric coordinate conversion
pub fn iso_to_screen(iso_x: f32, iso_y: f32, tile_width: f32, tile_height: f32) -> (f32, f32) {
    let sx = (iso_x - iso_y) * tile_width * 0.5;
    let sy = (iso_x + iso_y) * tile_height * 0.5;
    (sx, sy)
}

pub fn screen_to_iso(sx: f32, sy: f32, tile_width: f32, tile_height: f32) -> (f32, f32) {
    let iso_x = (sx / (tile_width * 0.5) + sy / (tile_height * 0.5)) * 0.5;
    let iso_y = (sy / (tile_height * 0.5) - sx / (tile_width * 0.5)) * 0.5;
    (iso_x, iso_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_to_world() {
        let tile = TileCoord::new(3, 5);
        let (wx, wy) = tile.to_world(32.0);
        assert!((wx - 96.0).abs() < 0.001);
        assert!((wy - 160.0).abs() < 0.001);
    }

    #[test]
    fn world_to_tile() {
        let tile = TileCoord::from_world(96.0, 160.0, 32.0);
        assert_eq!(tile, TileCoord::new(3, 5));
    }

    #[test]
    fn tile_negative_world() {
        let tile = TileCoord::new(-2, -3);
        let (wx, wy) = tile.to_world(32.0);
        assert!((wx - (-64.0)).abs() < 0.001);
        assert!((wy - (-96.0)).abs() < 0.001);
    }

    #[test]
    fn neighbors_4_count() {
        let tile = TileCoord::new(5, 5);
        assert_eq!(tile.neighbors_4().len(), 4);
    }

    #[test]
    fn neighbors_8_count() {
        let tile = TileCoord::new(5, 5);
        assert_eq!(tile.neighbors_8().len(), 8);
    }

    #[test]
    fn chebyshev_distance() {
        let a = TileCoord::new(0, 0);
        let b = TileCoord::new(3, 5);
        assert_eq!(a.chebyshev_distance(&b), 5); // max(3, 5)
    }

    #[test]
    fn chebyshev_distance_adjacent() {
        let a = TileCoord::new(5, 5);
        let b = TileCoord::new(6, 5);
        assert_eq!(a.chebyshev_distance(&b), 1);
    }

    #[test]
    fn chunked_tilemap_set_get() {
        let mut map = ChunkedTilemap::new(16);
        map.set_tile(TileCoord::new(5, 3), 42);
        assert_eq!(map.get_tile(&TileCoord::new(5, 3)), 42);
    }

    #[test]
    fn chunked_tilemap_crosses_boundary() {
        let mut map = ChunkedTilemap::new(8);
        map.set_tile(TileCoord::new(0, 0), 1);
        map.set_tile(TileCoord::new(8, 0), 2); // Different chunk

        assert_eq!(map.chunk_count(), 2, "Should span 2 chunks");
    }

    #[test]
    fn chunked_tilemap_empty_returns_zero() {
        let map = ChunkedTilemap::new(16);
        assert_eq!(map.get_tile(&TileCoord::new(100, 100)), 0);
    }

    #[test]
    fn iso_roundtrip() {
        let (ix, iy) = (3.0, 5.0);
        let (sx, sy) = iso_to_screen(ix, iy, 64.0, 32.0);
        let (rix, riy) = screen_to_iso(sx, sy, 64.0, 32.0);
        assert!((rix - ix).abs() < 0.01);
        assert!((riy - iy).abs() < 0.01);
    }
}
