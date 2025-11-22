//! # Timetable Core Library
//!
//! This library provides functionality for parsing Bromcom PDF timetables and generating
//! formatted SVG visualizations with color-coded department maps.
//!
//! ## Features
//!
//! - Parse Bromcom timetable PDFs using coordinate-based text extraction
//! - Configure room-to-department mappings with customizable colors
//! - Apply per-week/day/period overrides to correct parsing errors
//! - Highlight school map SVGs based on department locations
//! - Render A4-sized SVG timetables with embedded maps
//!
//! ## Example Usage
//!
//! ```no_run
//! use timetable_core::{config::Config, parser::parse_pdf, renderer::render_timetable, processor::{process_map, MapHighlight}};
//! use std::path::Path;
//! use std::collections::HashSet;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Load configuration
//! let config = Config::load(Path::new("config.toml"))?;
//!
//! // Parse PDF timetable
//! let mut weeks = parse_pdf(Path::new("input/timetable.pdf"))?;
//!
//! // Apply overrides from config
//! config.apply_overrides(&mut weeks);
//!
//! // Render each week to SVG
//! for (i, week) in weeks.iter().enumerate() {
//!     // Build highlights for departments used in this week
//!     let mut highlights = Vec::new();
//!     let mut seen_ids = HashSet::new();
//!     
//!     for lesson in &week.lessons {
//!         if let Some(mapping) = config.get_style_for_room(&lesson.room) {
//!             if seen_ids.insert(mapping.map_id.clone()) {
//!                 highlights.push(MapHighlight {
//!                     id: mapping.map_id.clone(),
//!                     color: mapping.bg_color.clone(),
//!                 });
//!             }
//!         }
//!     }
//!     
//!     // Process school map with highlights
//!     let map_svg = process_map(Path::new("resources/map.svg"), &highlights)?;
//!     
//!     // Render to output file
//!     let output_path = format!("output/week_{}.svg", i + 1);
//!     render_timetable(week, &config, &map_svg, Path::new(&output_path))?;
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Modules
//!
//! - [`config`]: Configuration loading and room-to-department mapping
//! - [`parser`]: PDF parsing and text extraction from Bromcom PDFs
//! - [`processor`]: SVG map manipulation and department highlighting
//! - [`renderer`]: Timetable SVG generation with embedded maps

pub mod config;
pub mod parser;
pub mod processor;
pub mod renderer;

pub fn hello() {
    println!("Hello from core!");
}
