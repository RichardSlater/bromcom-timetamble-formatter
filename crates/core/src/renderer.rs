//! SVG timetable rendering with embedded maps.
//!
//! This module generates A4-sized SVG documents containing a formatted weekly
//! timetable grid with color-coded cells and an embedded school map.

use crate::config::Config;
use crate::parser::Week;
use std::fs;
use std::path::Path;
use svg::node::element::{Group, Rectangle, Text};
use svg::Document;
use thiserror::Error;

/// Errors that can occur during SVG rendering.
#[derive(Error, Debug)]
pub enum RenderError {
    /// SVG file writing error
    #[error("SVG generation error: {0}")]
    Svg(#[from] std::io::Error),
}

/// Render a timetable week to an SVG file.
///
/// Generates an A4-sized (210mm × 297mm) SVG document containing:
/// - A formatted timetable grid with student name, week identifier, and lessons
/// - Color-coded cells based on room-to-department mappings
/// - Break and lunch period rows
/// - An embedded school map with highlighted departments
///
/// # Arguments
///
/// * `week` - The week data to render
/// * `config` - Configuration for room mappings and styling
/// * `map_content` - Processed SVG map content (from [`process_map`](crate::processor::process_map))
/// * `output_path` - Path where the SVG file will be written
///
/// # Returns
///
/// `Ok(())` if the SVG was successfully generated and written.
///
/// # Errors
///
/// Returns [`RenderError`] if:
/// - The output file cannot be created or written
/// - The output directory doesn't exist
///
/// # Example
///
/// ```no_run
/// use timetable_core::{config::Config, parser::{parse_pdf, Week}, renderer::render_timetable};
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::load(Path::new("config.toml"))?;
/// let weeks = parse_pdf(Path::new("input/timetable.pdf"))?;
/// let map_svg = "<svg></svg>"; // Processed map content
///
/// for (i, week) in weeks.iter().enumerate() {
///     let output = format!("output/week_{}.svg", i + 1);
///     render_timetable(week, &config, map_svg, Path::new(&output))?;
/// }
/// # Ok(())
/// # }
/// ```
pub fn render_timetable(
    week: &Week,
    config: &Config,
    map_content: &str,
    output_path: &Path,
) -> Result<(), RenderError> {
    // A4 @ 96 DPI ~= 794 x 1123
    let width = 794;
    let height = 1123;

    let timetable_height = 650;
    let _map_height = height - timetable_height;

    let mut document = Document::new()
        .set("viewBox", (0, 0, width, height))
        .set("width", "210mm")
        .set("height", "297mm");

    // Add white background rectangle for the entire page
    let background = Rectangle::new()
        .set("x", 0)
        .set("y", 0)
        .set("width", width)
        .set("height", height)
        .set("fill", "#ffffff");
    document = document.add(background);

    // Inject Styles matching the diagram
    let styles = r#"
        .detail {
            font-family: 'Bahnschrift Light', Bahnschrift, Arial, sans-serif;
            font-size: 11px;
            font-weight: 300;
            fill: #231f20;
        }

        .subject {
            font-family: Bahnschrift, Arial, sans-serif;
            font-size: 11px;
            font-weight: 400;
            fill: #231f20;
        }

        .room {
            font-family: 'Bahnschrift SemiBold', Bahnschrift, Arial, sans-serif;
            font-size: 18px;
            font-weight: 600;
            fill: #231f20;
            text-anchor: middle;
            dominant-baseline: middle;
        }

        .label {
            font-family: 'Bahnschrift SemiBold', Bahnschrift, Arial, sans-serif;
            font-size: 11px;
            font-weight: 600;
            fill: #231f20;
        }

        .box {
            fill: none;
            stroke: #231f20;
            stroke-width: 1;
            stroke-miterlimit: 10;
        }

        .period-label {
            font-family: 'Bahnschrift SemiBold', Bahnschrift, Arial, sans-serif;
            font-size: 12px;
            font-weight: 600;
            fill: #231f20;
            text-anchor: middle;
        }

        .header-text {
            font-family: Bahnschrift, Arial, sans-serif;
            font-size: 14px;
            font-weight: 400;
            fill: #231f20;
        }

        .week-label {
            font-family: 'Bahnschrift SemiBold', Bahnschrift, Arial, sans-serif;
            font-size: 16px;
            font-weight: 600;
            fill: #231f20;
        }
    "#;

    let style_element = svg::node::element::Style::new(styles);
    let defs = svg::node::element::Definitions::new().add(style_element);
    document = document.add(defs);

    // 1. Draw Timetable
    let timetable_group = draw_timetable_grid(week, config, width, timetable_height);
    document = document.add(timetable_group);

    // 2. Embed Map
    // We wrap the map content in a nested <svg> to handle positioning
    // The map_content is a full <svg> string. We need to strip the xml declaration if present,
    // and maybe wrap it in a <g> with transform.
    // Or better: use <svg x="..." y="..." width="..." height="..."> ... </svg>
    // But we have the content as a string.

    // We can't easily add a raw string to `svg::Document`.
    // So we will serialize the document so far, and then inject the map string.

    let mut svg_string = document.to_string();

    // Remove the closing </svg>
    if svg_string.ends_with("</svg>") {
        svg_string.truncate(svg_string.len() - 6);
    }

    // Inject the map if provided (map_content non-empty). If empty, skip embedding.
    if !map_content.trim().is_empty() {
        // We place it at the bottom.
        let map_y = timetable_height + 20;
        let map_area_height = height - map_y - 20; // Leave 20px margin at bottom

        svg_string.push_str(&format!(
            "<svg x=\"0\" y=\"{}\" width=\"{}\" height=\"{}\">",
            map_y, width, map_area_height
        ));

        // Strip <?xml ... ?> if exists
        let clean_map = map_content.trim_start_matches(|c| c != '<');
        let clean_map = if clean_map.starts_with("<?xml") {
            if let Some(idx) = clean_map.find("?>") {
                &clean_map[idx + 2..]
            } else {
                clean_map
            }
        } else {
            clean_map
        };

        svg_string.push_str(clean_map);
        svg_string.push_str("</svg>");
    }

    // Close the root svg
    svg_string.push_str("</svg>");

    fs::write(output_path, svg_string)?;

    Ok(())
}

fn draw_timetable_grid(week: &Week, config: &Config, width: i32, height: i32) -> Group {
    let mut group = Group::new().set("id", "timetable");

    // Grid dimensions
    let cols = 5; // Mon-Fri
    let periods = 6; // PD + L1-L5

    let left_margin = 60; // Space for period labels
    let top_margin = 80; // Space for student name and week
    let right_margin = 30;
    let bottom_margin = 40; // Space for update date

    let grid_width = width - left_margin - right_margin;
    let grid_height = height - top_margin - bottom_margin;

    let break_height = 24;
    let lunch_height = 24;

    let total_gap_height = break_height + lunch_height;
    let row_height = (grid_height - total_gap_height) / periods;
    let col_width = grid_width / cols;

    // Add student name and form at top left
    let student_info = if let (Some(name), Some(form)) = (&week.student_name, &week.form) {
        format!("{} ({})", name, form)
    } else if let Some(name) = &week.student_name {
        name.clone()
    } else {
        String::from("Student Timetable")
    };

    let text_student = Text::new(student_info.as_str())
        .set("x", left_margin)
        .set("y", 30)
        .set("class", "header-text");
    group = group.add(text_student);

    // Add week label at top center
    let text_week = Text::new(week.week_name.as_str())
        .set("x", width / 2)
        .set("y", 30)
        .set("text-anchor", "middle")
        .set("class", "week-label");
    group = group.add(text_week);

    // Draw day headers (Monday-Friday)
    let days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"];
    for (i, day) in days.iter().enumerate() {
        let x = left_margin + (i as i32 * col_width) + (col_width / 2);
        let y = top_margin - 15;
        let text = Text::new(*day)
            .set("x", x)
            .set("y", y)
            .set("text-anchor", "middle")
            .set("class", "header-text");
        group = group.add(text);
    }

    // Period labels and rows
    let period_labels = ["PD", "L1", "L2", "L3", "L4", "L5"];

    for (period_idx, label) in period_labels.iter().enumerate() {
        let mut y = top_margin + (period_idx as i32 * row_height);

        // Adjust for breaks - break comes after L2 (index 2)
        if period_idx > 2 {
            y += break_height;
        }
        // Lunch comes after L4 (index 4)
        if period_idx > 4 {
            y += lunch_height;
        }

        // Draw period label on left
        let text_period = Text::new(*label)
            .set("x", 30)
            .set("y", y + (row_height / 2))
            .set("dominant-baseline", "middle")
            .set("class", "period-label");
        group = group.add(text_period);

        // Draw break after L2 (period_idx 2)
        if period_idx == 2 {
            let cell_padding = 3;
            let break_y = y + row_height + cell_padding;
            // Calculate actual content width (5 columns worth of cells)
            let total_content_width = col_width * cols;
            let rect_break = Rectangle::new()
                .set("x", left_margin + cell_padding)
                .set("y", break_y)
                .set("width", total_content_width - (cell_padding * 2))
                .set("height", break_height - (cell_padding * 2))
                .set("fill", "#eeeeee")
                .set("stroke", "#231f20")
                .set("stroke-width", 1);
            group = group.add(rect_break);

            let text_break = Text::new("Break (11:00 - 11:30)")
                .set("x", left_margin + (total_content_width / 2))
                .set("y", break_y + ((break_height - (cell_padding * 2)) / 2) + 1)
                .set("text-anchor", "middle")
                .set("dominant-baseline", "middle")
                .set("class", "detail");
            group = group.add(text_break);
        }

        // Draw lunch after L4 (period_idx 4)
        if period_idx == 4 {
            let cell_padding = 3;
            let lunch_y = y + row_height + cell_padding;
            // Calculate actual content width (5 columns worth of cells)
            let total_content_width = col_width * cols;
            let rect_lunch = Rectangle::new()
                .set("x", left_margin + cell_padding)
                .set("y", lunch_y)
                .set("width", total_content_width - (cell_padding * 2))
                .set("height", lunch_height - (cell_padding * 2))
                .set("fill", "#eeeeee")
                .set("stroke", "#231f20")
                .set("stroke-width", 1);
            group = group.add(rect_lunch);

            let text_lunch = Text::new("Lunch (13:30 - 14:10)")
                .set("x", left_margin + (total_content_width / 2))
                .set("y", lunch_y + (lunch_height / 2) - 2)
                .set("text-anchor", "middle")
                .set("dominant-baseline", "middle")
                .set("class", "detail");
            group = group.add(text_lunch);
        }
    }

    // Draw lessons
    for lesson in &week.lessons {
        let x = left_margin + (lesson.day_index as i32 * col_width);

        // Calculate Y based on period and gaps
        let mut y = top_margin + (lesson.period_index as i32 * row_height);
        if lesson.period_index > 2 {
            y += break_height;
        }
        if lesson.period_index > 4 {
            y += lunch_height;
        }

        // Handle Unknown room - use dark grey
        let is_unknown_room = lesson.room == "Unknown" || lesson.room == "DEFAULT";

        // Get color mapping from config
        let (bg_color, fg_color) = if is_unknown_room {
            ("#e0e0e0", "#4a4a4a") // Light grey bg, dark grey fg for unknown
        } else {
            config
                .get_style_for_room(&lesson.room)
                .map(|m| (m.bg_color.as_str(), m.fg_color.as_str()))
                .unwrap_or(("#ffffff", "#231f20"))
        };

        let cell_padding = 3; // Space between cells
        let label_width = 30; // Width of the vertical label section on right

        // Main cell area (white background)
        let main_width = col_width - label_width - (cell_padding * 2);
        let rect_main = Rectangle::new()
            .set("x", x + cell_padding)
            .set("y", y + cell_padding)
            .set("width", main_width)
            .set("height", row_height - (cell_padding * 2))
            .set("fill", "#ffffff")
            .set("stroke", "#231f20")
            .set("stroke-width", 1);
        group = group.add(rect_main);

        // Right label area (colored background)
        let label_x = x + col_width - label_width - cell_padding;
        let rect_label = Rectangle::new()
            .set("x", label_x)
            .set("y", y + cell_padding)
            .set("width", label_width)
            .set("height", row_height - (cell_padding * 2))
            .set("fill", bg_color)
            .set("stroke", "#231f20")
            .set("stroke-width", 1);
        group = group.add(rect_label);

        // Text: Subject (top left, bold)
        // Split long subjects into multiple lines if needed
        let subject_words: Vec<&str> = lesson.subject.split_whitespace().collect();
        let max_chars_per_line = 18;

        if lesson.subject.len() > max_chars_per_line && subject_words.len() > 1 {
            // Multi-line subject
            let mut lines = Vec::new();
            let mut current_line = String::new();

            for word in subject_words {
                if current_line.is_empty() {
                    current_line = word.to_string();
                } else if current_line.len() + word.len() < max_chars_per_line {
                    current_line.push(' ');
                    current_line.push_str(word);
                } else {
                    lines.push(current_line.clone());
                    current_line = word.to_string();
                }
            }
            if !current_line.is_empty() {
                lines.push(current_line);
            }

            // Render each line
            for (line_idx, line) in lines.iter().enumerate() {
                let text_subject_line = Text::new(line.as_str())
                    .set("x", x + cell_padding + 5)
                    .set("y", y + cell_padding + 12 + (line_idx as i32 * 11))
                    .set("class", "subject")
                    .set("font-weight", "bold");
                group = group.add(text_subject_line);
            }
        } else {
            // Single line subject
            let text_subject = Text::new(lesson.subject.as_str())
                .set("x", x + cell_padding + 5)
                .set("y", y + cell_padding + 14)
                .set("class", "subject")
                .set("font-weight", "bold");
            group = group.add(text_subject);
        }

        // Text: Room code (above teacher) - only if not Unknown
        if lesson.room != "Unknown" {
            let text_room = Text::new(lesson.room.as_str())
                .set("x", x + cell_padding + 5)
                .set("y", y + row_height - cell_padding - 22)
                .set("class", "detail");
            group = group.add(text_room);
        }

        // Text: Teacher (bottom, smaller text) - only if not Unknown
        if lesson.teacher != "Unknown" {
            let text_teacher = Text::new(lesson.teacher.as_str())
                .set("x", x + cell_padding + 5)
                .set("y", y + row_height - cell_padding - 8)
                .set("class", "detail")
                .set("font-size", "9px");
            group = group.add(text_teacher);
        }

        // Text: Class code as vertical label on right (rotated 90°, large font, saturated color)
        // Use class_code if available, otherwise use subject for Unknown rooms, otherwise room code
        let label_text = if !lesson.class_code.is_empty() {
            &lesson.class_code
        } else if is_unknown_room {
            &lesson.subject // Use subject for unknown rooms
        } else {
            &lesson.room
        };

        let class_x = label_x + (label_width / 2) - 2;
        let class_y = y + (row_height / 2);

        let text_class = Text::new(label_text)
            .set("x", class_x)
            .set("y", class_y)
            .set("transform", format!("rotate(90 {} {})", class_x, class_y))
            .set("text-anchor", "middle")
            .set("dominant-baseline", "middle")
            .set(
                "font-family",
                "'Bahnschrift SemiBold', Bahnschrift, Arial, sans-serif",
            )
            .set("font-size", "20px")
            .set("font-weight", "600")
            .set("fill", fg_color);
        group = group.add(text_class);
    }

    // Add update date footer
    let update_date = chrono::Local::now().format("%d %B %Y").to_string();
    let text_update = Text::new(format!("Updated: {}", update_date).as_str())
        .set("x", width - right_margin)
        .set("y", height - 10)
        .set("text-anchor", "end")
        .set("class", "detail");
    group = group.add(text_update);

    group
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Mapping};
    use crate::parser::Lesson;
    use std::env;

    fn sample_week() -> Week {
        let lessons = vec![
            Lesson {
                subject: "Maths".into(),
                room: "MA3".into(),
                teacher: "Ms Test A".into(),
                class_code: "MA3".into(),
                day_index: 0,
                period_index: 1,
            },
            Lesson {
                subject: "Science".into(),
                room: "SC8".into(),
                teacher: "Mr Test B".into(),
                class_code: "SC8".into(),
                day_index: 1,
                period_index: 2,
            },
        ];

        Week {
            lessons,
            week_name: "Week Test".into(),
            student_name: Some("Test Student".into()),
            form: Some("9X1".into()),
        }
    }

    #[test]
    fn render_timetable_generates_svg_with_expected_content() {
        let cfg = Config {
            mappings: vec![
                Mapping {
                    prefix: "MA".into(),
                    bg_color: "#fcdcd8".into(),
                    fg_color: "#e8a490".into(),
                    map_id: "Maths_Rooms".into(),
                    label: Some("Maths".into()),
                },
                Mapping {
                    prefix: "SC".into(),
                    bg_color: "#fad7e6".into(),
                    fg_color: "#e68cb8".into(),
                    map_id: "Science_Rooms".into(),
                    label: Some("Science".into()),
                },
            ],
            overrides: vec![],
        };

        let map_svg = "<svg><g id=\"Maths_Rooms\"><path d=\"M0\"/></g><g id=\"Science_Rooms\"><path d=\"M0\"/></g></svg>";
        let week = sample_week();

        let mut out_path = env::temp_dir();
        out_path.push("timetable_test_output.svg");

        let res = render_timetable(&week, &cfg, map_svg, &out_path);
        assert!(res.is_ok());

        let content = std::fs::read_to_string(&out_path).expect("output svg exists");
        // basic checks: student name, one subject, and a room label
        assert!(content.contains("Test Student"));
        assert!(content.contains("Maths"));
        assert!(content.contains("MA3"));

        // cleanup
        let _ = std::fs::remove_file(&out_path);
    }
}
