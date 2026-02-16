use anyhow::{bail, Context, Result};
use clap::Parser;
use gsnake_levels::solver::{load_level, solve_level};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Parser, Debug)]
#[command(name = "profile_solver")]
#[command(about = "Benchmark solve_level runtime across level fixtures")]
struct Args {
    /// Root directory containing difficulty subfolders (easy/medium/hard)
    #[arg(long, default_value = "levels")]
    levels_root: PathBuf,

    /// Number of repeated runs for each level
    #[arg(long, default_value = "5")]
    iterations: usize,

    /// Maximum search depth passed to the solver
    #[arg(short = 'd', long = "max-depth", default_value = "500")]
    max_depth: usize,

    /// Comma-delimited difficulty list, e.g. easy,medium
    #[arg(long, value_delimiter = ',', default_value = "easy,medium,hard")]
    difficulties: Vec<String>,
}

#[derive(Debug, Clone)]
struct LevelTarget {
    difficulty: String,
    path: PathBuf,
}

#[derive(Debug, Default, Clone, Copy)]
struct LevelStats {
    total: Duration,
    min: Option<Duration>,
    max: Option<Duration>,
    solves: usize,
    total_moves: usize,
}

impl LevelStats {
    fn record(&mut self, elapsed: Duration, moves: usize) {
        self.total += elapsed;
        self.solves += 1;
        self.total_moves += moves;
        self.min = Some(self.min.map_or(elapsed, |current| current.min(elapsed)));
        self.max = Some(self.max.map_or(elapsed, |current| current.max(elapsed)));
    }

    fn avg_ms(self) -> f64 {
        if self.solves == 0 {
            return 0.0;
        }
        duration_ms(self.total) / self.solves as f64
    }

    fn avg_moves(self) -> f64 {
        if self.solves == 0 {
            return 0.0;
        }
        self.total_moves as f64 / self.solves as f64
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    if args.iterations == 0 {
        bail!("--iterations must be greater than zero");
    }

    let normalized_difficulties = normalize_difficulties(&args.difficulties);
    if normalized_difficulties.is_empty() {
        bail!("No valid difficulties provided");
    }

    let targets = discover_levels(&args.levels_root, &normalized_difficulties)?;
    if targets.is_empty() {
        bail!(
            "No level JSON files found under {}",
            args.levels_root.display()
        );
    }

    let total_solves = targets.len() * args.iterations;
    let mut level_stats: BTreeMap<PathBuf, LevelStats> = BTreeMap::new();
    let mut difficulty_totals: BTreeMap<String, Duration> = BTreeMap::new();
    let total_start = Instant::now();

    for _ in 0..args.iterations {
        for target in &targets {
            let level_start = Instant::now();
            let level = load_level(&target.path)?;
            let solution = solve_level(level, args.max_depth).with_context(|| {
                format!(
                    "Failed to solve {} (difficulty {})",
                    target.path.display(),
                    target.difficulty
                )
            })?;
            let elapsed = level_start.elapsed();
            level_stats
                .entry(target.path.clone())
                .or_default()
                .record(elapsed, solution.len());
            *difficulty_totals
                .entry(target.difficulty.clone())
                .or_default() += elapsed;
        }
    }

    let wall_time = total_start.elapsed();
    println!("Solver benchmark");
    println!("levels root: {}", args.levels_root.display());
    println!("difficulties: {}", normalized_difficulties.join(","));
    println!("iterations per level: {}", args.iterations);
    println!("max depth: {}", args.max_depth);
    println!("levels benchmarked: {}", targets.len());
    println!("total solves: {}", total_solves);
    println!("wall time: {:.3} s", duration_s(wall_time));
    println!(
        "mean solve time: {:.3} ms",
        duration_ms(wall_time) / total_solves as f64
    );

    println!("\nPer-difficulty cumulative time:");
    for difficulty in &normalized_difficulties {
        let total = difficulty_totals
            .get(difficulty)
            .copied()
            .unwrap_or(Duration::ZERO);
        println!("  - {}: {:.3} s", difficulty, duration_s(total));
    }

    let mut hotspots: Vec<(&PathBuf, &LevelStats)> = level_stats.iter().collect();
    hotspots.sort_by(|a, b| {
        b.1.total
            .cmp(&a.1.total)
            .then_with(|| a.0.as_os_str().cmp(b.0.as_os_str()))
    });

    println!("\nHotspot summary (top 3 by cumulative time):");
    for (index, (path, stats)) in hotspots.into_iter().take(3).enumerate() {
        println!(
            "  {}. {} | total {:.3} s | avg {:.3} ms | min {:.3} ms | max {:.3} ms | avg moves {:.1}",
            index + 1,
            path.display(),
            duration_s(stats.total),
            stats.avg_ms(),
            duration_ms(stats.min.unwrap_or_default()),
            duration_ms(stats.max.unwrap_or_default()),
            stats.avg_moves()
        );
    }

    Ok(())
}

fn discover_levels(levels_root: &Path, difficulties: &[String]) -> Result<Vec<LevelTarget>> {
    let mut targets = Vec::new();

    for difficulty in difficulties {
        let difficulty_dir = levels_root.join(difficulty);
        if !difficulty_dir.exists() {
            bail!(
                "Difficulty directory not found: {}",
                difficulty_dir.display()
            );
        }

        let mut files = Vec::new();
        for entry in fs::read_dir(&difficulty_dir)
            .with_context(|| format!("Failed to read {}", difficulty_dir.display()))?
        {
            let path = entry
                .with_context(|| format!("Failed to read entry in {}", difficulty_dir.display()))?
                .path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                files.push(path);
            }
        }
        files.sort();

        for path in files {
            targets.push(LevelTarget {
                difficulty: difficulty.clone(),
                path,
            });
        }
    }

    Ok(targets)
}

fn normalize_difficulties(raw_difficulties: &[String]) -> Vec<String> {
    let mut normalized = Vec::new();

    for difficulty in raw_difficulties {
        let trimmed = difficulty.trim().to_lowercase();
        if trimmed.is_empty() {
            continue;
        }
        if !normalized.iter().any(|existing| existing == &trimmed) {
            normalized.push(trimmed);
        }
    }

    normalized
}

fn duration_s(duration: Duration) -> f64 {
    duration.as_secs_f64()
}

fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn normalize_difficulties_trims_lowercases_and_deduplicates() {
        let raw = vec![
            " Easy ".to_string(),
            "medium".to_string(),
            "EASY".to_string(),
            "".to_string(),
        ];
        let normalized = normalize_difficulties(&raw);
        assert_eq!(normalized, vec!["easy".to_string(), "medium".to_string()]);
    }

    #[test]
    fn discover_levels_returns_sorted_json_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let levels_root = temp_dir.path();
        fs::create_dir_all(levels_root.join("easy"))?;
        fs::create_dir_all(levels_root.join("medium"))?;

        fs::write(levels_root.join("easy").join("b.json"), "{}")?;
        fs::write(levels_root.join("easy").join("a.json"), "{}")?;
        fs::write(levels_root.join("easy").join("ignore.txt"), "x")?;
        fs::write(levels_root.join("medium").join("m.json"), "{}")?;

        let difficulties = vec!["easy".to_string(), "medium".to_string()];
        let discovered = discover_levels(levels_root, &difficulties)?;

        let paths: Vec<String> = discovered
            .iter()
            .map(|target| {
                target
                    .path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            })
            .collect();
        assert_eq!(paths, vec!["a.json", "b.json", "m.json"]);
        Ok(())
    }
}
