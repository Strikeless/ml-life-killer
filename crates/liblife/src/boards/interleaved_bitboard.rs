pub struct InterleavedBitboardLifeBoard {
    width: u32,
    height: u32,
}

/*
TODO: Port this somewhat optimized board and engine into the new API

#![feature(stmt_expr_attributes)]
#![cfg_attr(test, feature(test, random))]

mod rule;

#[cfg(test)]
mod test;

pub use rule::{LifeRules, LifeRule, LifeRuleCondition};

use std::{fmt::Debug, ops::{Deref, DerefMut}, sync::Arc};
use itertools::Itertools;

/// The backing type of a chunk bitboard, representing the tile states of a chunk.
/// Must have atleast [`CHUNK_WIDTH`]*[`CHUNK_HEIGHT`] bits.
type ChunkBitBoard = u16;

/// The width of a single chunk. Must exceed [`CHUNK_OVERLAP`].
const CHUNK_WIDTH: usize = 4;
/// The height of a single chunk. Must exceed [`CHUNK_OVERLAP`].
const CHUNK_HEIGHT: usize = 4;

/// The number of unique chunk bitboard values of the size [`CHUNK_WIDTH`] x [`CHUNK_HEIGHT`].
const CHUNK_UNIQUE_STATE_COUNT: usize = 2usize.pow(CHUNK_WIDTH as u32 * CHUNK_HEIGHT as u32);

/// How many tiles of overlap there is between chunks.
/// 2 is THE correct value for the algorithm, this is a constant just for clarity.
const CHUNK_OVERLAP: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Life {
    pub chunk_store: ChunkStore,
    pub chunk_tick_cache_manager: Arc<ChunkTickCacheManager>,
}

impl Life {
    pub fn new(chunk_tick_cache_manager: Arc<ChunkTickCacheManager>, width: usize, height: usize) -> Self {
        if !Self::are_valid_dimensions(width, height) {
            panic!("Invalid dimensions, must be divisible by chunk size {CHUNK_WIDTH}x{CHUNK_HEIGHT}");
        }

        /*
            Chunks overlap visualization, character indicates the chunk, uppercase = managed tile, lowercase = unmanaged tile
            a--   a--   ab-   ab-   -bc   -bc   --c   --c
            ---   ---   ---   ---   ---   ---   ---   ---

            a--   A--   Ab-   aB-   -Bc   -bC   --C   --c
            ---   ---   ---   ---   ---   ---   ---   ---

            a--   A--   Ab-   aB-   -Bc   -bC   --C   --c
            d--   d--   de-   de-   -ef   -ef   --f   --f

            a--   a--   ab-   ab-   -bc   -bc   --c   --c
            d--   D--   De-   dE-   -Ef   -eF   --F   --f

            ---   ---   ---   ---   ---   ---   ---   ---
            d--   D--   De-   dE-   -Ef   -eF   --F   --f

            ---   ---   ---   ---   ---   ---   ---   ---
            d--   d--   de-   de-   -ef   -ef   --f   --f
        */
        let chunks_wide = width / (CHUNK_WIDTH - CHUNK_OVERLAP);
        let chunks_tall = height / (CHUNK_HEIGHT - CHUNK_OVERLAP);
        let chunk_store = ChunkStore::new(chunks_wide, chunks_tall);

        Self {
            chunk_store,
            chunk_tick_cache_manager,
        }
    }

    pub fn are_valid_dimensions(width: usize, height: usize) -> bool {
        width * height > 0 && width % CHUNK_WIDTH == 0 && height % CHUNK_HEIGHT == 0
    }

    pub fn width(&self) -> usize {
        self.chunk_store.width() * (CHUNK_WIDTH - CHUNK_OVERLAP)
    }

    pub fn height(&self) -> usize {
        self.chunk_store.height() * (CHUNK_HEIGHT - CHUNK_OVERLAP)
    }

    pub fn tile(&self, tile_pos: [usize; 2]) -> bool {
        let (chunk_pos, in_chunk_pos) = self.tile_to_chunk_pos(tile_pos);

        let chunk = self.chunk_store.chunk(chunk_pos);
        chunk.tile(in_chunk_pos)
    }

    pub fn tile_mut(&mut self, tile_pos: [usize; 2]) -> MutableTileRef<'_> {
        let (chunk_pos, in_chunk_pos) = self.tile_to_chunk_pos(tile_pos);

        let chunk = self.chunk_store.chunk_mut(chunk_pos);
        chunk.tile_mut(in_chunk_pos)
    }

    pub fn iter_tiles(&self) -> impl Iterator<Item = bool> {
        (0..self.height())
            .cartesian_product(0..self.width())
            .map(|(y, x)| self.tile([x, y]))
    }

    pub fn iter_index_to_pos(&self, iter_index: usize) -> [usize; 2] {
        index_to_pos(self.width(), iter_index)
    }

    fn tile_to_chunk_pos(&self, [tile_x, tile_y]: [usize; 2]) -> ([usize; 2], [usize; 2]) {
        debug_assert!(tile_x < self.width(), "tile_to_chunk_pos tile_x out of bounds {} >= {}", tile_x, self.width());
        debug_assert!(tile_y < self.height(), "tile_to_chunk_pos tile_y out of bounds {} >= {}", tile_y, self.height());

        const CHUNK_OVERLAP_HALF: usize = CHUNK_OVERLAP / 2;
        /*
         * Pretty sure all this math maths out. If not, have fun figuring this shit out,
         * because I'm not explaining it. Cheers.
         */
        // NOTE: Must minimize to last chunk since they're edge cases where their last tiles have no managing chunk and so we would go out of bounds.
        let chunk_pos = [
            (tile_x.saturating_sub(CHUNK_OVERLAP_HALF) / (CHUNK_WIDTH - CHUNK_OVERLAP)).min(self.chunk_store.width()),
            (tile_y.saturating_sub(CHUNK_OVERLAP_HALF) / (CHUNK_HEIGHT - CHUNK_OVERLAP)).min(self.chunk_store.height()),
        ];

        // This is a clusterfuck.
        // The following would otherwise do but does chunk 0 incorrectly (because 0-1 is saturated to 0 and later on added to 1 again):
        // ((tile_y.saturating_sub(CHUNK_OVERLAP_HALF)) % (CHUNK_HEIGHT - CHUNK_OVERLAP)) + CHUNK_OVERLAP_HALF
        let in_chunk_pos = [
            (((tile_x as isize - CHUNK_OVERLAP_HALF as isize) % (CHUNK_WIDTH - CHUNK_OVERLAP) as isize) + CHUNK_OVERLAP_HALF as isize) as usize,
            (((tile_y as isize - CHUNK_OVERLAP_HALF as isize) % (CHUNK_HEIGHT - CHUNK_OVERLAP) as isize) + CHUNK_OVERLAP_HALF as isize) as usize,
        ];

        (chunk_pos, in_chunk_pos)
    }

    fn chunk_to_origin_tile_pos([chunk_x, chunk_y]: [usize; 2]) -> [usize; 2] {
        [
            chunk_x * (CHUNK_WIDTH - CHUNK_OVERLAP),
            chunk_y * (CHUNK_WIDTH - CHUNK_OVERLAP),
        ]
    }

    pub fn tick(&mut self) {
        let chunks_wide = self.chunk_store.width();
        let chunks_tall = self.chunk_store.height();

        /*
         * Update unmanaged tiles of chunks from the neighbor chunks that do manage those tiles.
         */
        // FIXME: HACK: There's some weird off-by-one error, something weird about the last chunks on both axes
        //              that's causing right and bottom overlap directions to fuck up with out-of-bounds tiles.
        //              Band-aid fix by ignoring the last chunks on both axes. I'm pretty sure this makes no sense
        //              (and if it does, I don't understand it), and there's something wrong elsewhere.
        //              We may be wasting time and memory on useless broken chunks and are currently doing just enough for everything to not break.
        for chunk_x in 0..chunks_wide - 1 {
            for chunk_y in 0..chunks_tall - 1 {
                for overlap_direction in ChunkOverlapDirection::values() {
                    let overlap_src_chunk_pos = match overlap_direction {
                        // Borders, these have no neighbor at this overlap direction so just skip processing them.
                        ChunkOverlapDirection::Left if chunk_x == 0 => continue,
                        ChunkOverlapDirection::Top if chunk_y == 0 => continue,
                        ChunkOverlapDirection::Right if chunk_x == chunks_wide - 1 => continue,
                        ChunkOverlapDirection::Bottom if chunk_y == chunks_tall - 1 => continue,
                        // Neighbored chunks
                        ChunkOverlapDirection::Left => [chunk_x - 1, chunk_y],
                        ChunkOverlapDirection::Top => [chunk_x, chunk_y - 1],
                        ChunkOverlapDirection::Right => [chunk_x + 1, chunk_y],
                        ChunkOverlapDirection::Bottom => [chunk_x, chunk_y + 1],
                    };
                    let [overlap_src_origin_tile_x, overlap_src_origin_tile_y] = Self::chunk_to_origin_tile_pos(overlap_src_chunk_pos);

                    for chunk_overlap in overlap_direction.overlaps() {
                        let ChunkOverlap {
                            // chunk tile position = position of a tile inside of a chunk / position of a tile relative to the origin tile position of a chunk.
                            src_pos: [overlap_src_chunk_tile_x, overlap_src_chunk_tile_y],
                            dest_pos: overlap_dest_chunk_tile_pos,
                        } = chunk_overlap;

                        let overlap_src_tile_pos = [
                            overlap_src_origin_tile_x + overlap_src_chunk_tile_x,
                            overlap_src_origin_tile_y + overlap_src_chunk_tile_y,
                        ];

                        // NOTE: The tile function returns the tile from a chunk where it's managed (doesn't come from an overlap),
                        //       thus we can just get the tile with that and it should just work.
                        let overlap_src_tile = self.tile(overlap_src_tile_pos);

                        // NOTE: Unlike the src tile, we cannot access the dest tile (unmanaged tile in currently iterated chunk) with tile_mut,
                        //       because just as the tile function, tile_mut accesses the tile in the chunk where it's managed and already up-to-date
                        //       (so the exact same chunk and tile as src_tile, on a correct implementation).
                        let chunk = self.chunk_store.chunk_mut([chunk_x, chunk_y]);
                        let mut overlap_dest_tile = chunk.tile_mut(*overlap_dest_chunk_tile_pos);

                        *overlap_dest_tile = overlap_src_tile;
                    }
                }
            }
        }

        /*
         * Update chunks to next state from bitboard cache.
         * This is the actual ticking part, where visible board state gets updated to the next generation.
         */
        for chunk in self.chunk_store.iter_chunks_mut() {
            let chunk_tick_cache = self.chunk_tick_cache_manager.get_cache(&chunk);
            chunk.bitboard = chunk_tick_cache.next_bitboard;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkStore {
    chunks: Vec<Chunk>,
    width: usize,
}

impl ChunkStore {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            chunks: vec![Chunk::empty(); width * height],
            width,
        }
    }

    pub fn from_chunks(chunks: Vec<Chunk>, width: usize, height: usize) -> Self {
        if chunks.len() != width * height {
            panic!("Wrong chunks len {} for width {} and height {} (expected {} chunks)", chunks.len(), width, height, width * height);
        }

        Self {
            chunks,
            width,
        }
    }

    pub fn chunk(&self, pos: [usize; 2]) -> &Chunk {
        let chunk_index = pos_to_index(self.width, pos);
        self.chunks.get(chunk_index).unwrap()
    }

    pub fn chunk_mut(&mut self, pos: [usize; 2]) -> &mut Chunk {
        let chunk_index = pos_to_index(self.width, pos);
        self.chunks.get_mut(chunk_index).unwrap()
    }

    pub fn iter_chunks_mut(&mut self) -> impl Iterator<Item = &mut Chunk> {
        self.chunks.iter_mut()
    }

    pub fn enumerate_chunks_mut(&mut self) -> impl Iterator<Item = ([usize; 2], &mut Chunk)> {
        let width = self.width;

        self.iter_chunks_mut()
            .enumerate()
            .map(move |(chunk_index, chunk)| (index_to_pos(width, chunk_index), chunk))
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.chunks.len() / self.width
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chunk {
    pub bitboard: ChunkBitBoard,
}

impl Chunk {
    pub fn from_bitboard(bitboard: ChunkBitBoard) -> Self {
        Self { bitboard }
    }

    pub fn empty() -> Self {
        Self { bitboard: 0 }
    }

    pub fn tile(&self, pos: [usize; 2]) -> bool {
        debug_assert!(pos[0] < CHUNK_WIDTH, "Chunk.tile X-coordinate over chunk width ({} >= {})", pos[0], CHUNK_WIDTH);
        debug_assert!(pos[1] < CHUNK_HEIGHT, "Chunk.tile Y-coordinate over chunk height ({} >= {})", pos[1], CHUNK_HEIGHT);

        let tile_index = pos_to_index(CHUNK_WIDTH, pos);
        let tile_bit = (self.bitboard >> tile_index) & 0b1;
        tile_bit == 0b1
    }

    pub fn tile_mut(&mut self, pos: [usize; 2]) -> MutableTileRef<'_> {
        let tile_index = pos_to_index(CHUNK_WIDTH, pos);
        let tile_bit = (self.bitboard >> tile_index) & 0b1;

        MutableTileRef {
            chunk: self,
            tile_index,
            new_value: tile_bit == 0b1,
        }
    }
}

pub struct MutableTileRef<'a> {
    chunk: &'a mut Chunk,
    tile_index: usize,
    new_value: bool,
}

impl Deref for MutableTileRef<'_> {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.new_value
    }
}

impl DerefMut for MutableTileRef<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.new_value
    }
}

impl Drop for MutableTileRef<'_> {
    fn drop(&mut self) {
        let value_bit = if self.new_value { 0b1 } else { 0b0 };

        self.chunk.bitboard &= !(0b1 << self.tile_index);
        self.chunk.bitboard |= value_bit << self.tile_index;
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ChunkTickCacheManager {
    // We know the vector is sized as-per CHUNK_UNIQUE_STATE_COUNT, but it can be a bit too big to store on the stack as an array.
    tick_caches: Vec<ChunkTickCache>,
}
impl Debug for ChunkTickCacheManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChunkTickCacheManager {{ ... }}")
    }
}

impl ChunkTickCacheManager {
    pub fn generate(rules: &LifeRules) -> Self {
        let tick_caches = (0..CHUNK_UNIQUE_STATE_COUNT)
            .map(|bitboard| {
                let chunk = Chunk::from_bitboard(bitboard as ChunkBitBoard);
                let chunk_next_state = rules.get_next_chunk_state(chunk);

                ChunkTickCache {
                    next_bitboard: chunk_next_state.bitboard,
                }
            })
            .collect::<Vec<_>>();

        Self {
            tick_caches,
        }
    }

    fn get_cache(&self, chunk: &Chunk) -> &ChunkTickCache {
        // SAFETY: A properly generated cache manager will always contain an entry in tick_caches for every possible bitboard value.
        &self.tick_caches[chunk.bitboard as usize]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ChunkTickCache {
    pub next_bitboard: ChunkBitBoard,
}
impl Debug for ChunkTickCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChunkTickCache {{ ... }}")
    }
}

enum ChunkOverlapDirection {
    Top,
    Left,
    Right,
    Bottom,
}

impl ChunkOverlapDirection {
    pub fn values() -> &'static [Self] {
        &[Self::Top, Self::Left, Self::Right, Self::Bottom]
    }

    pub const fn overlaps(&self) -> &'static [ChunkOverlap] {
        #[rustfmt::skip]
        match self {
            Self::Top => &[
                ChunkOverlap { src_pos: [0, 2], dest_pos: [0, 0] },
                ChunkOverlap { src_pos: [1, 2], dest_pos: [1, 0] },
                ChunkOverlap { src_pos: [2, 2], dest_pos: [2, 0] },
                ChunkOverlap { src_pos: [3, 2], dest_pos: [3, 0] },
            ],
            Self::Bottom => &[
                ChunkOverlap { src_pos: [0, 1], dest_pos: [0, 3] },
                ChunkOverlap { src_pos: [1, 1], dest_pos: [1, 3] },
                ChunkOverlap { src_pos: [2, 1], dest_pos: [2, 3] },
                ChunkOverlap { src_pos: [3, 1], dest_pos: [3, 3] },
            ],
            Self::Left => &[
                ChunkOverlap { src_pos: [2, 0], dest_pos: [0, 0] },
                ChunkOverlap { src_pos: [2, 1], dest_pos: [0, 1] },
                ChunkOverlap { src_pos: [2, 2], dest_pos: [0, 2] },
                ChunkOverlap { src_pos: [2, 3], dest_pos: [0, 3] },
            ],
            Self::Right => &[
                ChunkOverlap { src_pos: [1, 0], dest_pos: [3, 0] },
                ChunkOverlap { src_pos: [1, 1], dest_pos: [3, 1] },
                ChunkOverlap { src_pos: [1, 2], dest_pos: [3, 2] },
                ChunkOverlap { src_pos: [1, 3], dest_pos: [3, 3] },
            ],
        }
    }
}

pub struct ChunkOverlap {
    pub src_pos: [usize; 2],
    pub dest_pos: [usize; 2],
}

fn pos_to_index(width: usize, [pos_x, pos_y]: [usize; 2]) -> usize {
    (pos_y * width) + pos_x
}

fn index_to_pos(width: usize, index: usize) -> [usize; 2] {
    let x = index % width;
    let y = index / width;
    [x, y]
}
*/
