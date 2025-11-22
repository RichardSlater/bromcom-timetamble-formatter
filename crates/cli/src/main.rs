use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use timetable_core::config::Config;
use timetable_core::parser::parse_pdf;
use timetable_core::processor::{process_map, MapHighlight};
use timetable_core::renderer::render_timetable;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the Bromcom PDF timetable
    #[arg(short, long)]
    input: PathBuf,

    /// Path to the configuration TOML file
    #[arg(short, long)]
    config: PathBuf,

    /// Path to the map SVG file
    #[arg(short, long)]
    map: Option<PathBuf>,

    /// Output directory for generated SVGs
    #[arg(short, long)]
    output: PathBuf,

    /// Student name (optional, e.g., "Alex Testington")
    #[arg(short, long)]
    student_name: Option<String>,

    /// Student form/class (optional, e.g., "11XX")
    #[arg(short, long)]
    form: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("Processing timetable from: {:?}", cli.input);

    // 1. Load Config
    let config = Config::load(&cli.config).context("Failed to load config")?;

    // 2. Parse PDF
    let mut weeks = parse_pdf(&cli.input).context("Failed to parse PDF")?;
    println!("Found {} weeks.", weeks.len());

    // 3. Apply overrides from config
    config.apply_overrides(&mut weeks);

    // Ensure output directory exists
    fs::create_dir_all(&cli.output).context("Failed to create output directory")?;

    // 4. Process each week
    for (i, week) in weeks.iter().enumerate() {
        println!("Processing {}", week.week_name);
        println!("  Total lessons: {}", week.lessons.len());

        // Override student name and form if provided via CLI
        let mut week_with_info = week.clone();
        if let Some(name) = &cli.student_name {
            week_with_info.student_name = Some(name.clone());
        }
        if let Some(form_code) = &cli.form {
            week_with_info.form = Some(form_code.clone());
        }

        // Debug: Show period distribution
        let mut period_counts = [0usize; 6];
        for lesson in &week.lessons {
            if lesson.period_index < 6 {
                period_counts[lesson.period_index] += 1;
            }
        }
        println!(
            "  Period distribution: PD={}, L1={}, L2={}, L3={}, L4={}, L5={}",
            period_counts[0],
            period_counts[1],
            period_counts[2],
            period_counts[3],
            period_counts[4],
            period_counts[5]
        );

        // Debug: Show first few PD lessons
        for lesson in week.lessons.iter().filter(|l| l.period_index == 0).take(2) {
            println!(
                "  PD Lesson: subject='{}', room='{}', teacher='{}'",
                lesson.subject, lesson.room, lesson.teacher
            );
        }

        // Identify highlights for this week
        let mut highlights = Vec::new();
        // We want to highlight departments used in this week.
        // We can iterate over lessons, find the room, look up the mapping, and add to highlights.
        // We should deduplicate.

        let mut seen_ids = std::collections::HashSet::new();

        for lesson in &week_with_info.lessons {
            if let Some(mapping) = config.get_style_for_room(&lesson.room) {
                if seen_ids.insert(mapping.map_id.clone()) {
                    highlights.push(MapHighlight {
                        id: mapping.map_id.clone(),
                        color: mapping.bg_color.clone(),
                    });
                }
            }
        }

        // 4. Process Map (optional)
        let map_svg = if let Some(map_path) = &cli.map {
            process_map(map_path, &highlights).context("Failed to process map")?
        } else {
            // No map provided — renderer will skip embedding
            String::new()
        };

        // 5. Render
        // Use a safe filename
        let safe_name = week_with_info
            .week_name
            .replace(|c: char| !c.is_alphanumeric() && c != ' ', "_");
        let filename = format!("{}_{}.svg", safe_name, i + 1);
        let output_path = cli.output.join(filename);

        render_timetable(&week_with_info, &config, &map_svg, &output_path)
            .context("Failed to render timetable")?;
        println!("Generated: {:?}", output_path);
    }

    Ok(())
}
