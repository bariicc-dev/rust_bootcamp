use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new() -> Self {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        Self {
            state: since_the_epoch.as_nanos() as u64,
        }
    }

    fn next(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    fn range(&mut self, min: u32, max: u32) -> u32 {
        if min >= max {
            return min;
        }
        min + (self.next() % (max - min))
    }
}

struct Config {
    map_file: Option<String>,
    generate: Option<String>,
    output: Option<String>,
    visualize: bool,
    both: bool,
    animate: bool,
    help: bool,
}

impl Config {
    fn new() -> Self {
        Self {
            map_file: None,
            generate: None,
            output: None,
            visualize: false,
            both: false,
            animate: false,
            help: false,
        }
    }
}

struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Vec<u8>>,
}

impl Grid {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![vec![0; width]; height],
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    cost: u32,
    x: usize,
    y: usize,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.x.cmp(&other.x))
            .then_with(|| self.y.cmp(&other.y))
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn print_help() {
    println!("Usage: hexpath [OPTIONS]");
    println!();
    println!("Find min/max cost paths in hexadecimal grid");
    println!();
    println!("Arguments:");
    println!("Map file (hex values, space separated)");
    println!();
    println!("Map format:");
    println!("- Each cell: 00-FF (hexadecimal)");
    println!("- Start: top-left (must be 00)");
    println!("- End: bottom-right (must be FF)");
    println!("- Moves: up, down, left, right");
    println!();
    println!("Options:");
    println!("--generate Generate random map (e.g., 8x4, 10x10)");
    println!("--output Save generated map to file");
    println!("--visualize Show colored map");
    println!("--both Show both min and max paths");
    println!("--animate Animate pathfinding");
    println!("-h, --help");
}

fn parse_args() -> Config {
    let args: Vec<String> = env::args().collect();
    let mut config = Config::new();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                config.help = true;
                return config;
            }
            "--generate" => {
                if i + 1 < args.len() {
                    config.generate = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    config.output = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--visualize" => config.visualize = true,
            "--both" => config.both = true,
            "--animate" => config.animate = true,
            arg => {
                if arg.starts_with('-') {
                    eprintln!("error: Unknown option {}", arg);
                    std::process::exit(2);
                }
                config.map_file = Some(arg.to_string());
            }
        }
        i += 1;
    }
    config
}

fn generate_map(size_str: &str) -> Result<Grid, String> {
    let parts: Vec<&str> = size_str.split('x').collect();
    if parts.len() != 2 {
        return Err("Invalid size format. Use WxH (e.g., 12x8)".to_string());
    }
    let width: usize = parts[0].parse().map_err(|_| "Invalid width")?;
    let height: usize = parts[1].parse().map_err(|_| "Invalid height")?;

    println!("Generating {}x{} hexadecimal grid...", width, height);

    let mut grid = Grid::new(width, height);
    let mut rng = Lcg::new();

    for y in 0..height {
        for x in 0..width {
            grid.cells[y][x] = rng.range(0, 256) as u8;
        }
    }

    grid.cells[0][0] = 0x00;
    grid.cells[height - 1][width - 1] = 0xFF;

    Ok(grid)
}

fn save_map(grid: &Grid, filename: &str) -> io::Result<()> {
    let mut file = File::create(filename)?;
    for y in 0..grid.height {
        let line: Vec<String> = grid.cells[y].iter().map(|c| format!("{:02X}", c)).collect();
        writeln!(file, "{}", line.join(" "))?;
    }
    println!("Map saved to: {}", filename);
    Ok(())
}

fn load_map(filename: &str) -> Result<Grid, String> {
    let mut file = File::open(filename).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let mut cells = Vec::new();
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let row: Result<Vec<u8>, _> = line
            .split_whitespace()
            .map(|s| u8::from_str_radix(s, 16))
            .collect();
        match row {
            Ok(r) => cells.push(r),
            Err(_) => return Err("Invalid hex value in map file".to_string()),
        }
    }

    if cells.is_empty() {
        return Err("Empty map file".to_string());
    }

    let height = cells.len();
    let width = cells[0].len();

    for row in &cells {
        if row.len() != width {
            return Err("Inconsistent row lengths".to_string());
        }
    }

    Ok(Grid {
        width,
        height,
        cells,
    })
}

fn get_ansi_color(val: u8) -> String {
    let (r, g, b) = if val < 43 {
        (255, (val as f32 * 6.0) as u8, 0)
    } else if val < 85 {
        (255 - ((val - 43) as f32 * 6.0) as u8, 255, 0)
    } else if val < 128 {
        (0, 255, ((val - 85) as f32 * 6.0) as u8)
    } else if val < 170 {
        (0, 255 - ((val - 128) as f32 * 6.0) as u8, 255)
    } else if val < 213 {
        (((val - 170) as f32 * 6.0) as u8, 0, 255)
    } else {
        (255, 0, 255)
    };

    format!("\x1b[38;2;{};{};{}m", r, g, b)
}

fn print_grid(grid: &Grid, path: Option<&Vec<(usize, usize)>>, is_max: bool) {
    let reset = "\x1b[0m";

    if path.is_some() {
        if is_max {
            println!("MAXIMUM COST PATH (shown in RED):");
        } else {
            println!("MINIMUM COST PATH (shown in WHITE):");
        }
        println!("=====================================");
        println!();
    } else {
        println!("HEXADECIMAL GRID (rainbow gradient):");
        println!("============================================================================");
        println!();
    }

    for y in 0..grid.height {
        for x in 0..grid.width {
            let val = grid.cells[y][x];
            let color = get_ansi_color(val);

            let mut is_in_path = false;
            if let Some(p) = path {
                if p.contains(&(x, y)) {
                    is_in_path = true;
                }
            }

            if is_in_path {
                if is_max {
                    print!("\x1b[31m{:02X}{} ", val, reset);
                } else {
                    print!("\x1b[37m{:02X}{} ", val, reset);
                }
            } else {
                print!("{}{:02X}{} ", color, val, reset);
            }
        }
        println!();
    }
    println!();
}

fn solve_dijkstra(
    grid: &Grid,
    maximize: bool,
    animate: bool,
) -> Option<(u32, Vec<(usize, usize)>)> {
    let mut dist = vec![vec![u32::MAX; grid.width]; grid.height];
    let mut parent = vec![vec![None; grid.width]; grid.height];
    let mut heap = BinaryHeap::new();

    dist[0][0] = 0;

    heap.push(State {
        cost: 0,
        x: 0,
        y: 0,
    });

    let mut step_count = 0;

    if animate {
        println!(
            "Searching for {} cost path...",
            if maximize { "maximum" } else { "minimum" }
        );
        println!();
    }

    while let Some(State { cost, x, y }) = heap.pop() {
        if animate {
            step_count += 1;
            if step_count <= 2 || (x == grid.width - 1 && y == grid.height - 1) {
                println!(
                    "Step {}: Exploring ({},{}) - cost: {}",
                    step_count, x, y, cost
                );
                for (ay, row) in dist.iter().enumerate() {
                    for (ax, &d) in row.iter().enumerate() {
                        if ax == x && ay == y {
                            print!("[*]");
                        } else if d != u32::MAX {
                            print!("[✓]");
                        } else {
                            print!("[ ]");
                        }
                    }
                    println!();
                }
                println!();
                if step_count <= 2 {
                    println!("[Animation continues...]");
                    println!();
                }
                thread::sleep(Duration::from_millis(100));
            }
        }

        if x == grid.width - 1 && y == grid.height - 1 {
            // Found target
            if animate {
                println!("Step {}: Path found!", step_count);
                for (ay, row) in dist.iter().enumerate() {
                    for (ax, &d) in row.iter().enumerate() {
                        if ax == x && ay == y {
                            print!("[✓]"); // Target reached
                        } else if d != u32::MAX {
                            print!("[✓]");
                        } else {
                            print!("[ ]");
                        }
                    }
                    println!();
                }
                println!();
            }

            let mut path = Vec::new();
            let mut curr = (x, y);
            path.push(curr);
            while let Some(p) = parent[curr.1][curr.0] {
                path.push(p);
                curr = p;
            }
            path.reverse();

            let mut true_cost = 0;

            for (px, py) in &path {
                true_cost += grid.cells[*py][*px] as u32;
            }

            return Some((true_cost, path));
        }

        if cost > dist[y][x] {
            continue;
        }

        let dx = [0, 0, 1, -1];
        let dy = [1, -1, 0, 0];

        for i in 0..4 {
            let nx = x as i32 + dx[i];
            let ny = y as i32 + dy[i];

            if nx >= 0 && nx < grid.width as i32 && ny >= 0 && ny < grid.height as i32 {
                let nx = nx as usize;
                let ny = ny as usize;

                let cell_val = grid.cells[ny][nx] as u32;
                let move_cost = if maximize { 255 - cell_val } else { cell_val };

                let new_cost = cost + move_cost;

                if new_cost < dist[ny][nx] {
                    dist[ny][nx] = new_cost;
                    parent[ny][nx] = Some((x, y));
                    heap.push(State {
                        cost: new_cost,
                        x: nx,
                        y: ny,
                    });
                }
            }
        }
    }

    None
}

fn print_path_details(grid: &Grid, cost: u32, path: &[(usize, usize)], is_max: bool) {
    if is_max {
        println!("MAXIMUM COST PATH:");
    } else {
        println!("MINIMUM COST PATH:");
    }
    println!("=====================");
    println!("Total cost: 0x{:X} ({} decimal)", cost, cost);
    println!("Path length: {} steps", path.len());

    println!("Path:");
    let path_str: Vec<String> = path.iter().map(|(x, y)| format!("({},{})", x, y)).collect();
    println!("{}", path_str.join("->"));
    println!();

    println!("Step-by-step costs:");

    if let Some(first) = path.first() {
        let val = grid.cells[first.1][first.0];
        println!("Start 0x{:02X} ({},{})", val, first.0, first.1);
    }

    let mut current_accumulated = 0;

    for (x, y) in path.iter().skip(1) {
        let val = grid.cells[*y][*x];
        current_accumulated += val as u32;
        println!("→ 0x{:02X} ({},{}) +{}", val, x, y, current_accumulated);
    }
    println!("Total: 0x{:X} ({})", cost, cost);
    println!();
}

fn main() {
    let config = parse_args();

    if config.help {
        print_help();
        return;
    }

    let grid = if let Some(size_str) = &config.generate {
        match generate_map(size_str) {
            Ok(g) => {
                if let Some(outfile) = &config.output {
                    if let Err(e) = save_map(&g, outfile) {
                        eprintln!("Error saving map: {}", e);
                        return;
                    }
                }

                println!();
                if config.output.is_none() {
                    println!("Generated map:");
                    for y in 0..g.height {
                        let line: Vec<String> =
                            g.cells[y].iter().map(|c| format!("{:02X}", c)).collect();
                        println!("{}", line.join(" "));
                    }
                    println!();
                }
                g
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                return;
            }
        }
    } else if let Some(filename) = &config.map_file {
        match load_map(filename) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("Error: {}", e);
                return;
            }
        }
    } else {
        print_help();
        return;
    };

    let should_solve =
        config.map_file.is_some() || config.visualize || config.both || config.animate;

    if !should_solve {
        return;
    }

    if config.map_file.is_some() {
        println!("Analyzing hexadecimal grid...");
        println!("Grid size: {}x{}", grid.width, grid.height);
        println!("Start: (0,0) = 0x{:02X}", grid.cells[0][0]);
        println!(
            "End: ({},{}) = 0x{:02X}",
            grid.width - 1,
            grid.height - 1,
            grid.cells[grid.height - 1][grid.width - 1]
        );
        println!();
    } else {
        println!("Finding optimal paths...");
        println!();
    }
    let min_res = solve_dijkstra(&grid, false, config.animate);

    let max_res = if config.both {
        solve_dijkstra(&grid, true, false)
    } else {
        None
    };

    if config.visualize {
        if config.both {
            if let Some((_, ref path)) = min_res {
                print_grid(&grid, Some(path), false);
            }

            if let Some((_, ref path)) = max_res {
                print_grid(&grid, Some(path), true);
            }
        } else if let Some((_, ref path)) = min_res {
            print_grid(&grid, Some(path), false);
        } else {
            print_grid(&grid, None, false);
        }
    } else {
        if let Some((cost, ref path)) = min_res {
            print_path_details(&grid, cost, path, false);
        }

        if let Some((cost, ref path)) = max_res {
            print_path_details(&grid, cost, path, true);
        }
    }
}
