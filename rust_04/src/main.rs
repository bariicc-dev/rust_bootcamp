use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::env;
use std::fs;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

type Grid = Vec<u8>;

fn usage() {
    eprintln!("Usage:");
    eprintln!("  rust04 generate <width> <height> <output>");
    eprintln!("  rust04 analyze <mapfile> [--both] [--visualize]");
}

fn main() {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        usage();
        std::process::exit(1);
    }

    match args.remove(0).as_str() {
        "generate" => {
            if args.len() != 3 {
                usage();
                std::process::exit(1);
            }
            let width: usize = args[0].parse().unwrap_or(0);
            let height: usize = args[1].parse().unwrap_or(0);
            if width == 0 || height == 0 {
                eprintln!("width and height must be positive numbers");
                std::process::exit(1);
            }
            let path = &args[2];
            let grid = generate_grid(width, height);
            if let Err(err) = write_grid(&grid, width, path) {
                eprintln!("failed to write {}: {}", path, err);
                std::process::exit(1);
            }
            println!("saved grid to {} ({}x{})", path, width, height);
        }
        "analyze" => {
            if args.is_empty() {
                usage();
                std::process::exit(1);
            }
            let file = args.remove(0);
            let both = args.contains(&"--both".to_string());
            let visualize = args.contains(&"--visualize".to_string());
            match read_grid(&file) {
                Ok((grid, width, height)) => {
                    analyze(&grid, width, height, both, visualize);
                }
                Err(err) => {
                    eprintln!("failed to read {}: {}", file, err);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            usage();
            std::process::exit(1);
        }
    }
}

fn generate_grid(width: usize, height: usize) -> Grid {
    let mut rng = SimpleRng::new();
    let mut grid = Vec::with_capacity(width * height);
    for _ in 0..width * height {
        grid.push((rng.next_u32() % 256) as u8);
    }
    grid
}

fn write_grid(grid: &Grid, width: usize, path: &str) -> io::Result<()> {
    let mut out = String::new();
    for (i, value) in grid.iter().enumerate() {
        if i % width == 0 && i != 0 {
            out.push('\n');
        }
        if i % width != 0 {
            out.push(' ');
        }
        out.push_str(&format!("{:02X}", value));
    }
    fs::write(path, out)
}

fn read_grid(path: &str) -> io::Result<(Grid, usize, usize)> {
    let content = fs::read_to_string(path)?;
    let mut grid = Vec::new();
    let mut width = 0usize;
    let mut height = 0usize;
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let values: Vec<u8> = line
            .split_whitespace()
            .map(|chunk| u8::from_str_radix(chunk, 16).unwrap_or(0))
            .collect();
        if width == 0 {
            width = values.len();
        } else if width != values.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "inconsistent row width in grid",
            ));
        }
        height += 1;
        grid.extend(values);
    }
    if width == 0 || height == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "grid file must contain at least one row",
        ));
    }
    Ok((grid, width, height))
}

fn idx(row: usize, col: usize, width: usize) -> usize {
    row * width + col
}

fn neighbors(row: usize, col: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
    let mut result = Vec::with_capacity(6);
    let dirs = [
        (row.wrapping_sub(1), col),
        (row + 1, col),
        (row, col.wrapping_sub(1)),
        (row, col + 1),
        (row.wrapping_sub(1), col + 1),
        (row + 1, col.wrapping_sub(1)),
    ];
    for &(r, c) in &dirs {
        if r < height && c < width {
            result.push((r, c));
        }
    }
    result
}

#[derive(Clone, Debug)]
struct PathResult {
    cost: u32,
    steps: Vec<(usize, usize)>,
}

fn dijkstra_min(grid: &Grid, width: usize, height: usize) -> Option<PathResult> {
    let total = width * height;
    let mut dist = vec![u32::MAX; total];
    let mut prev = vec![None::<usize>; total];
    let start = 0;
    let goal = total - 1;
    dist[start] = grid[start] as u32;

    let mut heap = BinaryHeap::new();
    heap.push((Reverse(dist[start]), start));

    while let Some((Reverse(cost), pos)) = heap.pop() {
        if cost != dist[pos] {
            continue;
        }
        if pos == goal {
            break;
        }
        let row = pos / width;
        let col = pos % width;
        for (nr, nc) in neighbors(row, col, width, height) {
            let nidx = idx(nr, nc, width);
            let next_cost = cost + grid[nidx] as u32;
            if next_cost < dist[nidx] {
                dist[nidx] = next_cost;
                prev[nidx] = Some(pos);
                heap.push((Reverse(next_cost), nidx));
            }
        }
    }

    if dist[goal] == u32::MAX {
        return None;
    }

    let mut path = Vec::new();
    let mut cur = goal;
    path.push((cur / width, cur % width));
    while let Some(p) = prev[cur] {
        cur = p;
        path.push((cur / width, cur % width));
    }
    path.reverse();

    Some(PathResult {
        cost: dist[goal],
        steps: path,
    })
}

fn dfs_max(
    grid: &Grid,
    width: usize,
    height: usize,
    pos: usize,
    goal: usize,
    visited: &mut Vec<bool>,
    current_cost: u32,
    best: &mut Option<PathResult>,
    current_path: &mut Vec<(usize, usize)>,
) {
    if pos == goal {
        let result = PathResult {
            cost: current_cost,
            steps: current_path.clone(),
        };
        if best.as_ref().map(|b| result.cost > b.cost).unwrap_or(true) {
            *best = Some(result);
        }
        return;
    }

    let row = pos / width;
    let col = pos % width;
    for (nr, nc) in neighbors(row, col, width, height) {
        let nidx = idx(nr, nc, width);
        if visited[nidx] {
            continue;
        }
        visited[nidx] = true;
        current_path.push((nr, nc));
        dfs_max(
            grid,
            width,
            height,
            nidx,
            goal,
            visited,
            current_cost + grid[nidx] as u32,
            best,
            current_path,
        );
        current_path.pop();
        visited[nidx] = false;
    }
}

fn find_max_path(grid: &Grid, width: usize, height: usize) -> Option<PathResult> {
    let cells = width * height;
    if cells > 64 {
        return None;
    }
    let start = 0usize;
    let goal = cells - 1;
    let mut visited = vec![false; cells];
    visited[start] = true;
    let mut path = vec![(0, 0)];
    let mut best = None;
    dfs_max(
        grid,
        width,
        height,
        start,
        goal,
        &mut visited,
        grid[start] as u32,
        &mut best,
        &mut path,
    );
    best
}

fn analyze(grid: &Grid, width: usize, height: usize, both: bool, visualize: bool) {
    println!("grid size: {} x {}", width, height);
    if let Some(min_path) = dijkstra_min(grid, width, height) {
        println!("minimum path cost: {}", min_path.cost);
        print_path(&min_path.steps);
        if visualize {
            println!();
            print_visual(grid, width, &min_path.steps, "MIN");
        }
    } else {
        println!("no path found");
    }

    if both {
        match find_max_path(grid, width, height) {
            Some(max_path) => {
                println!("\nmaximum path cost (simple DFS): {}", max_path.cost);
                print_path(&max_path.steps);
                if visualize {
                    println!();
                    print_visual(grid, width, &max_path.steps, "MAX");
                }
            }
            None => println!("\nskipping maximum-path search (grid too large)"),
        }
    }
}

fn print_path(steps: &[(usize, usize)]) {
    println!("steps (row,col):");
    for (i, (r, c)) in steps.iter().enumerate() {
        println!("  {:>3}: ({}, {})", i, r, c);
    }
}

fn print_visual(grid: &Grid, width: usize, steps: &[(usize, usize)], label: &str) {
    println!("{} path overlay:", label);
    let mut marks = vec![false; grid.len()];
    for &(r, c) in steps {
        marks[idx(r, c, width)] = true;
    }
    for (i, value) in grid.iter().enumerate() {
        let r = i / width;
        let c = i % width;
        if c == 0 && r != 0 {
            println!();
        }
        let cell = if marks[i] {
            format!("[{value:02X}]")
        } else {
            format!(" {value:02X} ")
        };
        print!("{}", cell);
        if c + 1 < width {
            print!(" ");
        }
    }
    println!();
}

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1);
        Self {
            state: seed ^ 0xA5A5_A5A5_1234_5678,
        }
    }

    fn next_u32(&mut self) -> u32 {
        // A tiny LCG for simple randomness.
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_write_roundtrip() {
        let grid = vec![0x0A, 0x0B, 0x0C, 0x0D];
        let path = std::env::temp_dir().join("rust04_roundtrip.txt");
        let path_str = path.to_string_lossy().to_string();
        write_grid(&grid, 2, &path_str).unwrap();
        let (back, w, h) = read_grid(&path_str).unwrap();
        assert_eq!((w, h), (2, 2));
        assert_eq!(back, grid);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn dijkstra_small_grid() {
        let grid = vec![1, 2, 3, 4];
        let res = dijkstra_min(&grid, 2, 2).unwrap();
        assert_eq!(res.cost, 1 + 2 + 4);
        assert_eq!(res.steps, vec![(0, 0), (0, 1), (1, 1)]);
    }
}
