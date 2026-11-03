//! The named cells of a map: the vocabulary for places on every map, spots or no spots (the metal-maps design,
//! `docs/design/2026-11-01-metal-maps.md` §1.2). A cell is a fixed size in elmos, not a share of the map (the user,
//! 2026-11-02: "a fixed elmo count rather than a subdivision quantity. Larger maps should have larger cell counts"):
//! the map is cut into `ceil(width / CELL)` columns and `ceil(height / CELL)` rows of equal size, so a cell is
//! [`CELL`] across on the usual map sizes (multiples of 1,024) and never under three quarters of it. Columns are
//! lettered west to east, rows numbered north to south: Comet Catcher (8,192 by 6,144) is A1 to H6, SpeedMetal
//! (13,312 by 2,048) A1 to M2, All That Glitters Extended (15,360 by 10,240) A1 to O10. The one definition the
//! scout, the report, the sheet, the `mark` tool, the picture, the record's header and the viewer share.

use bot_protocol::Vec3;

/// A cell's side, on a map whose size is a multiple of it.
pub const CELL: f32 = 1024.0;

/// One letter a column: BAR's largest maps (24,576 elmos) reach X.
const MAX_COLUMNS: f32 = 26.0;

/// A rectangle of cells: (first column, first row, last column, last row), 0-based, inclusive.
pub type Cells = (u8, u8, u8, u8);

/// The grid of one map.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    pub columns: u8,
    pub rows: u8,
    width: f32,
    height: f32,
}

impl Default for Grid {
    /// One cell over nothing: a brain before its map is known.
    fn default() -> Grid {
        Grid::of(0.0, 0.0)
    }
}

impl Grid {
    /// The grid of a map `width` by `height` elmos.
    pub fn of(width: f32, height: f32) -> Grid {
        let count = |extent: f32| (extent / CELL).ceil().clamp(1.0, MAX_COLUMNS) as u8;
        Grid { columns: count(width), rows: count(height), width: width.max(1.0), height: height.max(1.0) }
    }

    pub fn count(&self) -> usize {
        self.columns as usize * self.rows as usize
    }

    /// A cell's size: `CELL` square on the usual maps.
    pub fn cell_size(&self) -> (f32, f32) {
        (self.width / self.columns as f32, self.height / self.rows as f32)
    }

    /// The column and row holding `pos` (a point off the map is in the nearest edge cell).
    pub fn cell_of(&self, pos: Vec3) -> (u8, u8) {
        let of = |value: f32, extent: f32, count: u8| ((value / extent * count as f32) as i32).clamp(0, count as i32 - 1) as u8;
        (of(pos.x, self.width, self.columns), of(pos.z, self.height, self.rows))
    }

    /// The name of the cell holding `pos`: "E3".
    pub fn name(&self, pos: Vec3) -> String {
        let (column, row) = self.cell_of(pos);
        Grid::name_of(column, row)
    }

    /// The name of a cell by its column and row, 0-based: (4, 2) is "E3".
    pub fn name_of(column: u8, row: u8) -> String {
        format!("{}{}", (b'A' + column) as char, row + 1)
    }

    /// The cell a name means, when it is one on this grid: "E3", "e3", " E3 " give (4, 2); "E7" on a six-row grid,
    /// "E", "3", "spot_3" give None.
    pub fn parse(&self, text: &str) -> Option<(u8, u8)> {
        let mut chars = text.trim().chars();
        let column = chars.next().filter(char::is_ascii_alphabetic)?.to_ascii_uppercase() as u8 - b'A';
        let digits = chars.as_str();
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let row = digits.parse::<u32>().ok().filter(|r| *r >= 1)? - 1;
        (column < self.columns && row < self.rows as u32).then_some((column, row as u8))
    }

    /// Whether `text` is a cell name on this grid.
    pub fn is_cell(&self, text: &str) -> bool {
        self.parse(text).is_some()
    }

    /// A cell's centre.
    pub fn centre(&self, column: u8, row: u8) -> Vec3 {
        let (w, h) = self.cell_size();
        Vec3 { x: (column as f32 + 0.5) * w, y: 0.0, z: (row as f32 + 0.5) * h }
    }

    /// The ground a rectangle of cells covers: its north-west and south-east corners.
    pub fn rect(&self, cells: Cells) -> (Vec3, Vec3) {
        let (w, h) = self.cell_size();
        let (c0, r0, c1, r1) = cells;
        (Vec3 { x: c0 as f32 * w, y: 0.0, z: r0 as f32 * h }, Vec3 { x: (c1 as f32 + 1.0) * w, y: 0.0, z: (r1 as f32 + 1.0) * h })
    }

    /// The cells as the picture writes them: "E1-H8", or "E4" for one cell.
    pub fn words(cells: Cells) -> String {
        let (c0, r0, c1, r1) = cells;
        if (c0, r0) == (c1, r1) { Grid::name_of(c0, r0) } else { format!("{}-{}", Grid::name_of(c0, r0), Grid::name_of(c1, r1)) }
    }

    /// The cells a rectangle of ground touches (`x0, z0` north-west, `x1, z1` south-east, in elmos).
    pub fn cells_touching(&self, x0: f32, z0: f32, x1: f32, z1: f32) -> Cells {
        let (c0, r0) = self.cell_of(Vec3 { x: x0 + 1.0, y: 0.0, z: z0 + 1.0 });
        let (c1, r1) = self.cell_of(Vec3 { x: x1 - 1.0, y: 0.0, z: z1 - 1.0 });
        (c0.min(c1), r0.min(r1), c0.max(c1), r0.max(r1))
    }

    /// Every cell, column by column, each column north to south: the order the scout's glance lists them.
    pub fn cells(&self) -> impl Iterator<Item = (u8, u8)> + '_ {
        (0..self.columns).flat_map(move |column| (0..self.rows).map(move |row| (column, row)))
    }

    /// A cell's index in [`Grid::cells`]'s order.
    pub fn index(&self, column: u8, row: u8) -> usize {
        column as usize * self.rows as usize + row as usize
    }

    /// The cells sharing an edge with (`column`, `row`): north, east, south, west, those on the map.
    pub fn neighbours(&self, column: u8, row: u8) -> Vec<(u8, u8)> {
        let (c, r) = (column as i32, row as i32);
        [(c, r - 1), (c + 1, r), (c, r + 1), (c - 1, r)]
            .into_iter()
            .filter(|(c, r)| (0..self.columns as i32).contains(c) && (0..self.rows as i32).contains(r))
            .map(|(c, r)| (c as u8, r as u8))
            .collect()
    }

    /// The grid in words for the model: "8x6 cells of 1,024 by 1,024 elmos; columns A-H run west to east (x), rows
    /// 1-6 run north to south (z)".
    pub fn description(&self) -> String {
        let (w, h) = self.cell_size();
        format!(
            "{}x{} cells of {} by {} elmos; columns A-{} run west to east (x), rows 1-{} run north to south (z)",
            self.columns, self.rows, thousands(w), thousands(h), (b'A' + self.columns - 1) as char, self.rows
        )
    }

    /// The first and last cell: "A1 to H6".
    pub fn range_words(&self) -> String {
        format!("A1 to {}", Grid::name_of(self.columns - 1, self.rows - 1))
    }
}

fn thousands(value: f32) -> String {
    let n = value.round() as i64;
    if n >= 1000 { format!("{},{:03}", n / 1000, n % 1000) } else { n.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: f32, z: f32) -> Vec3 {
        Vec3 { x, y: 0.0, z }
    }

    /// The maps played: Comet 8 by 6, SpeedMetal 13 by 2, Full Metal Plate 12 by 12, Glitters Extended 15 by 10,
    /// SD-129 14 by 14, Great Divide 3 by 4; a cell is 1,024 square on each.
    #[test]
    fn the_cell_is_a_fixed_size_so_larger_maps_have_more_cells() {
        let grids = [((8192.0, 6144.0), (8, 6)), ((13312.0, 2048.0), (13, 2)), ((12288.0, 12288.0), (12, 12)), ((15360.0, 10240.0), (15, 10)), ((14336.0, 14336.0), (14, 14)), ((3072.0, 4096.0), (3, 4))];
        for ((w, h), (columns, rows)) in grids {
            let grid = Grid::of(w, h);
            assert_eq!((grid.columns, grid.rows), (columns, rows), "{w} by {h}");
            assert_eq!(grid.cell_size(), (CELL, CELL));
        }
        // An odd size is cut into equal cells no smaller than three quarters of one: 7,680 gives eight of 960.
        assert_eq!(Grid::of(7680.0, 7680.0).cell_size(), (960.0, 960.0));
        assert_eq!(Grid::of(24576.0, 1.0).range_words(), "A1 to X1");
        assert_eq!(Grid::default().count(), 1);
    }

    #[test]
    fn names_parse_back_including_two_digit_rows() {
        let grid = Grid::of(15360.0, 10240.0);
        assert_eq!(grid.name(at(4500.0, 9800.0)), "E10");
        assert_eq!(grid.parse("E10"), Some((4, 9)));
        assert_eq!(grid.parse(" e3 "), Some((4, 2)));
        assert_eq!(grid.parse("O10"), Some((14, 9)));
        assert_eq!(grid.parse("P1"), None, "off the east edge");
        assert_eq!(grid.parse("E11"), None, "off the south edge");
        assert_eq!(grid.parse("E0"), None);
        assert_eq!(grid.parse("E"), None);
        assert_eq!(grid.parse("3"), None);
        assert_eq!(grid.parse("spot_3"), None);
        assert_eq!(grid.parse("E3x"), None);
        assert_eq!(grid.name(at(-5.0, 20000.0)), "A10", "a point off the map is in the nearest edge cell");
        assert_eq!(grid.centre(4, 2), at(4608.0, 2560.0));
        assert_eq!(grid.rect((4, 0, 7, 9)), (at(4096.0, 0.0), at(8192.0, 10240.0)));
        assert_eq!(Grid::words((4, 0, 7, 9)), "E1-H10");
        assert_eq!(Grid::words((4, 2, 4, 2)), "E3");
        assert_eq!(grid.cells_touching(4100.0, 0.0, 8192.0, 10240.0), (4, 0, 7, 9));
    }

    #[test]
    fn neighbours_share_an_edge_and_stay_on_the_map() {
        let grid = Grid::of(8192.0, 6144.0);
        assert_eq!(grid.neighbours(0, 0), vec![(1, 0), (0, 1)]);
        assert_eq!(grid.neighbours(4, 2), vec![(4, 1), (5, 2), (4, 3), (3, 2)]);
        assert_eq!(grid.neighbours(7, 5), vec![(7, 4), (6, 5)]);
        assert_eq!(grid.cells().count(), 48);
        assert_eq!(grid.cells().nth(grid.index(4, 2)), Some((4, 2)));
        assert_eq!(grid.description(), "8x6 cells of 1,024 by 1,024 elmos; columns A-H run west to east (x), rows 1-6 run north to south (z)");
        assert_eq!(grid.range_words(), "A1 to H6");
    }
}
