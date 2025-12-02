# rust04: Hex grid pathfinding demo

A small Dijkstra-based tool for working with hexagonal cost grids. Keep it simple: generate a grid file, then analyze it for a minimum-cost path (and optionally an expensive one on small grids).

## Usage

```bash
cargo run --release --bin rust04 -- generate <width> <height> <output>
# writes a hex grid like "0A 2F ..." to the file

cargo run --release --bin rust04 -- analyze <mapfile> [--both] [--visualize]
# shows grid info, a minimum path, and optionally a (search-limited) maximum path
```

## Notes
- Costs are stored as two-digit hexadecimal bytes separated by spaces.
- The start is the top-left cell; the goal is the bottom-right cell.
- Maximum-path search only runs when the grid has at most 64 cells to keep the DFS simple.
- Neighbor layout uses axial-style coordinates: six directions (up, down, left, right, up-right, down-left).
