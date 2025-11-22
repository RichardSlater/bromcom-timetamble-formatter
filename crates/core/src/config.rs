//! Configuration management for timetable formatting.
//!
//! This module handles loading TOML configuration files, managing room-to-department
//! mappings, and applying lesson overrides.

use serde::Deserialize;
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during configuration operations.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// I/O error reading configuration file
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// TOML parsing error
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_style_for_room_longest_prefix() {
        let toml = r###"
            [[mappings]]
            prefix = "M"
            bg_color = "#fff"
            fg_color = "#000"
            map_id = "M_rooms"

            [[mappings]]
            prefix = "MA"
            bg_color = "#abc"
            fg_color = "#111"
            map_id = "MA_rooms"
        "###;

        let cfg: Config = toml::from_str(toml).unwrap();
        let m = cfg.get_style_for_room("MA12").unwrap();
        assert_eq!(m.prefix, "MA");
        assert_eq!(m.bg_color, "#abc");
    }

    #[test]
    fn test_default_fg_color() {
        let toml = r###"
            [[mappings]]
            prefix = "EN"
            color = "#ddeeff"
            map_id = "EN_rooms"
        "###;

        let cfg: Config = toml::from_str(toml).unwrap();
        let m = cfg.get_style_for_room("EN4").unwrap();
        assert_eq!(m.fg_color, "#231f20");
    }

    #[test]
    fn test_apply_overrides_updates_lesson() {
        use crate::parser::{Lesson, Week};

        let lessons = vec![Lesson {
            subject: "Maths".into(),
            room: "MA3".into(),
            teacher: "Mr A".into(),
            class_code: "MA3".into(),
            day_index: 3,    // Thursday
            period_index: 1, // L1
        }];

        let mut weeks = vec![Week {
            lessons,
            week_name: "Week 1".into(),
            student_name: None,
            form: None,
        }];

        let toml = r###"
            mappings = []
            [[overrides]]
            week = 1
            day = "Thursday"
            period = "L1"
            room = "SC6"
            teacher = "Mr Test B"
        "###;

        let cfg: Config = toml::from_str(toml).unwrap();
        cfg.apply_overrides(&mut weeks);

        let lesson = &weeks[0].lessons[0];
        assert_eq!(lesson.room, "SC6");
        assert_eq!(lesson.teacher, "Mr Test B");
    }
}

/// Configuration for timetable formatting and room mappings.
///
/// Loaded from a TOML file containing room-to-department mappings and
/// optional per-lesson overrides.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Room-to-department mapping rules
    pub mappings: Vec<Mapping>,
    /// Per-week/day/period lesson overrides
    #[serde(default)]
    pub overrides: Vec<Override>,
}

/// Maps a room code prefix to visual styling and map element.
///
/// Used to color-code timetable cells and highlight map regions
/// based on the room where a lesson takes place.
#[derive(Debug, Deserialize)]
pub struct Mapping {
    /// Room code prefix to match (e.g., "MA" matches MA1, MA2, MA3, etc.)
    pub prefix: String,
    /// Background color for cell and map (hex code, e.g., "#fcdcd8")
    #[serde(alias = "color")]
    pub bg_color: String,
    /// Foreground/text color for labels (hex code, defaults to "#231f20")
    #[serde(default = "default_fg_color")]
    pub fg_color: String,
    /// SVG element ID in map file to highlight
    pub map_id: String,
    /// Human-readable department label (e.g., "Maths", "Science")
    pub label: Option<String>,
}

/// Override for a specific lesson in the timetable.
///
/// Allows correcting parsing errors or making manual adjustments
/// to specific lessons by week, day, and period.
#[derive(Debug, Deserialize, Clone)]
pub struct Override {
    /// Week number (1-based, e.g., 1 = Week 1, 2 = Week 2)
    pub week: usize,
    /// Day name ("Monday", "Tuesday", etc. or abbreviated "Mon", "Tue")
    pub day: String,
    /// Period identifier ("PD", "L1", "L2", "L3", "L4", "L5")
    pub period: String,
    /// Override subject name (optional)
    pub subject: Option<String>,
    /// Override room code (optional)
    pub room: Option<String>,
    /// Override teacher name (optional)
    pub teacher: Option<String>,
    /// Override class code (optional)
    pub class_code: Option<String>,
}

fn default_fg_color() -> String {
    "#231f20".to_string()
}

impl Config {
    /// Load configuration from a TOML file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the config.toml file
    ///
    /// # Returns
    ///
    /// A parsed [`Config`] structure.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] if:
    /// - The file cannot be read
    /// - The TOML syntax is invalid
    /// - Required fields are missing
    ///
    /// # Example
    ///
    /// ```no_run
    /// use timetable_core::config::Config;
    /// use std::path::Path;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::load(Path::new("config.toml"))?;
    /// println!("Loaded {} room mappings", config.mappings.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Find the mapping for a given room code.
    ///
    /// Returns the mapping with the longest matching prefix, allowing
    /// specific overrides (e.g., "MA" matches "MA3" but "MA1" would match
    /// a more specific "MA1" prefix if configured).
    ///
    /// # Arguments
    ///
    /// * `room_code` - Room code to look up (e.g., "MA3", "SC8")
    ///
    /// # Returns
    ///
    /// The matching [`Mapping`], or `None` if no prefix matches.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use timetable_core::config::Config;
    /// # use std::path::Path;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let config = Config::load(Path::new("config.toml"))?;
    /// if let Some(mapping) = config.get_style_for_room("MA3") {
    ///     println!("Room MA3 maps to: {}", mapping.map_id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_style_for_room(&self, room_code: &str) -> Option<&Mapping> {
        // Find the longest matching prefix
        self.mappings
            .iter()
            .filter(|m| room_code.starts_with(&m.prefix))
            .max_by_key(|m| m.prefix.len())
    }

    /// Apply configured overrides to parsed weeks.
    ///
    /// Modifies lessons in-place based on override rules. Each override
    /// specifies a week, day, and period, and can update any combination
    /// of subject, room, teacher, or class code.
    ///
    /// # Arguments
    ///
    /// * `weeks` - Mutable slice of week data to modify
    ///
    /// # Warnings
    ///
    /// Prints warnings to stderr if:
    /// - Week number is out of range
    /// - Day or period name is invalid
    /// - No matching lesson is found for an override
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use timetable_core::{config::Config, parser::parse_pdf};
    /// # use std::path::Path;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::load(Path::new("config.toml"))?;
    /// let mut weeks = parse_pdf(Path::new("input/timetable.pdf"))?;
    /// 
    /// config.apply_overrides(&mut weeks);
    /// # Ok(())
    /// # }
    /// ```
    pub fn apply_overrides(&self, weeks: &mut [crate::parser::Week]) {
        for override_rule in &self.overrides {
            // Find the target week (1-based index)
            if override_rule.week == 0 || override_rule.week > weeks.len() {
                eprintln!(
                    "Warning: Override week {} is out of range",
                    override_rule.week
                );
                continue;
            }

            let week = &mut weeks[override_rule.week - 1];

            // Parse day to index
            let day_index = match override_rule.day.to_lowercase().as_str() {
                "monday" | "mon" => 0,
                "tuesday" | "tue" => 1,
                "wednesday" | "wed" => 2,
                "thursday" | "thu" => 3,
                "friday" | "fri" => 4,
                _ => {
                    eprintln!("Warning: Unknown day '{}'", override_rule.day);
                    continue;
                }
            };

            // Parse period to index
            let period_index = match override_rule.period.to_uppercase().as_str() {
                "PD" => 0,
                "L1" => 1,
                "L2" => 2,
                "L3" => 3,
                "L4" => 4,
                "L5" => 5,
                _ => {
                    eprintln!("Warning: Unknown period '{}'", override_rule.period);
                    continue;
                }
            };

            // Find and update the lesson
            if let Some(lesson) = week
                .lessons
                .iter_mut()
                .find(|l| l.day_index == day_index && l.period_index == period_index)
            {
                if let Some(subject) = &override_rule.subject {
                    lesson.subject = subject.clone();
                }
                if let Some(room) = &override_rule.room {
                    lesson.room = room.clone();
                }
                if let Some(teacher) = &override_rule.teacher {
                    lesson.teacher = teacher.clone();
                }
                if let Some(class_code) = &override_rule.class_code {
                    lesson.class_code = class_code.clone();
                }
                println!(
                    "Applied override: Week {}, {}, {}",
                    override_rule.week, override_rule.day, override_rule.period
                );
            } else {
                eprintln!(
                    "Warning: No lesson found for Week {}, {}, {}",
                    override_rule.week, override_rule.day, override_rule.period
                );
            }
        }
    }
}
