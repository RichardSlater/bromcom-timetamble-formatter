//! PDF parsing for Bromcom timetables.
//!
//! This module extracts text with coordinates from Bromcom PDF files and reconstructs
//! the timetable grid structure using heuristics for day/period detection.

use lopdf::{Document, Object};
use regex::Regex;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during PDF parsing.
#[derive(Error, Debug)]
pub enum ParserError {
    /// PDF document parsing error
    #[error("PDF parsing error: {0}")]
    Pdf(#[from] lopdf::Error),
    /// I/O error reading PDF file
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Failed to extract text from PDF
    #[error("Failed to extract text from PDF")]
    ExtractionFailed,
}

/// A single lesson entry in the timetable.
#[derive(Debug, Clone)]
pub struct Lesson {
    /// Subject name (e.g., "Mathematics", "French")
    pub subject: String,
    /// Room code (e.g., "MA3", "SC8")
    pub room: String,
    /// Teacher name (e.g., "Ms Test A", "Mr Test B")
    pub teacher: String,
    /// Class code (e.g., "MA3", "HU9")
    pub class_code: String,
    /// Day of week (0 = Monday, 4 = Friday)
    pub day_index: usize,
    /// Period index (0 = PD, 1 = L1, 2 = L2, etc.)
    pub period_index: usize,
}

/// A week of timetable data containing multiple lessons.
#[derive(Debug, Clone)]
pub struct Week {
    /// All lessons for this week
    pub lessons: Vec<Lesson>,
    /// Week identifier (e.g., "Week 1", "Week 2")
    pub week_name: String,
    /// Student name extracted from PDF (e.g., "Alex Testington")
    pub student_name: Option<String>,
    /// Form/class code (e.g., "11XX")
    pub form: Option<String>,
}

/// Internal representation of text item with coordinates.
#[derive(Debug, Clone)]
struct TextItem {
    x: f64,
    y: f64,
    text: String,
}

/// Parse a Bromcom PDF timetable file.
///
/// Extracts text with coordinates from each page and reconstructs the timetable grid
/// by detecting week boundaries, day columns, and period rows.
///
/// # Arguments
///
/// * `path` - Path to the Bromcom PDF file
///
/// # Returns
///
/// A vector of [`Week`] structures, one for each week found in the PDF.
///
/// # Errors
///
/// Returns [`ParserError`] if:
/// - The PDF file cannot be opened or read
/// - The PDF structure is invalid
/// - Text extraction fails
///
/// # Example
///
/// ```no_run
/// use timetable_core::parser::parse_pdf;
/// use std::path::Path;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let weeks = parse_pdf(Path::new("input/timetable.pdf"))?;
/// println!("Found {} weeks", weeks.len());
/// for week in weeks {
///     println!("{}  has {} lessons", week.week_name, week.lessons.len());
/// }
/// # Ok(())
/// # }
/// ```
pub fn parse_pdf(path: &Path) -> Result<Vec<Week>, ParserError> {
    let doc = Document::load(path)?;
    let mut weeks = Vec::new();

    for (page_num, page_id) in doc.get_pages() {
        let text_items = extract_text_from_page(&doc, page_id)?;
        if text_items.is_empty() {
            continue;
        }

        let page_weeks = process_page_text(text_items, page_num);
        weeks.extend(page_weeks);
    }

    Ok(weeks)
}

fn extract_text_from_page(
    doc: &Document,
    page_id: (u32, u16),
) -> Result<Vec<TextItem>, ParserError> {
    let content_bytes = doc.get_page_content(page_id)?;
    let content = lopdf::content::Content::decode(&content_bytes)?;
    let mut text_items = Vec::new();

    let mut current_x = 0.0;
    let mut current_y = 0.0;

    for operation in content.operations.iter() {
        match operation.operator.as_str() {
            "BT" => {
                current_x = 0.0;
                current_y = 0.0;
            }
            "Tm" => {
                if operation.operands.len() == 6 {
                    if let (Ok(e), Ok(f)) = (
                        operation.operands[4].as_float(),
                        operation.operands[5].as_float(),
                    ) {
                        current_x = e as f64;
                        current_y = f as f64;
                    }
                }
            }
            "Td" | "TD" => {
                if operation.operands.len() == 2 {
                    if let (Ok(tx), Ok(ty)) = (
                        operation.operands[0].as_float(),
                        operation.operands[1].as_float(),
                    ) {
                        current_x += tx as f64;
                        current_y += ty as f64;
                    }
                }
            }
            "Tj" => {
                if let Some(text) = decode_text_object(&operation.operands[0]) {
                    text_items.push(TextItem {
                        x: current_x,
                        y: current_y,
                        text: decode_bromcom_text(&text),
                    });
                }
            }
            "TJ" => {
                if let Ok(arr) = operation.operands[0].as_array() {
                    let mut full_text = String::new();
                    for item in arr {
                        if let Some(text) = decode_text_object(item) {
                            full_text.push_str(&text);
                        }
                    }
                    text_items.push(TextItem {
                        x: current_x,
                        y: current_y,
                        text: decode_bromcom_text(&full_text),
                    });
                }
            }
            _ => {}
        }
    }

    Ok(text_items)
}

fn decode_text_object(obj: &Object) -> Option<String> {
    match obj {
        Object::String(bytes, _) => String::from_utf8(bytes.clone()).ok(),
        _ => None,
    }
}

fn decode_bromcom_text(text: &str) -> String {
    text.chars()
        .filter(|&c| c != '\0')
        .map(|c| {
            let code = c as u8;
            let new_code = code.wrapping_add(29);
            new_code as char
        })
        .collect()
}

fn process_page_text(items: Vec<TextItem>, _page_num: u32) -> Vec<Week> {
    let mut weeks = Vec::new();

    let week_regex = Regex::new(r"Week\s+(\d+)").unwrap();

    // Collect headers with their week number
    let mut week_headers: Vec<(&TextItem, u32)> = items
        .iter()
        .filter_map(|i| {
            week_regex
                .captures(&i.text)
                .map(|cap| (i, cap[1].parse::<u32>().unwrap_or(0)))
        })
        .collect();

    // Sort by Week Number (Ascending) -> This ensures Top-to-Bottom order
    week_headers.sort_by_key(|k| k.1);

    if week_headers.is_empty() {
        return weeks;
    }

    // Determine Y direction
    // If we have multiple headers, we can check if Y increases or decreases
    let y_increases_down = if week_headers.len() > 1 {
        week_headers[1].0.y > week_headers[0].0.y
    } else {
        // Fallback: Check if most items are below or above the header
        let header_y = week_headers[0].0.y;
        let items_below_y_down = items.iter().filter(|i| i.y > header_y).count();
        let items_below_y_up = items.iter().filter(|i| i.y < header_y).count();
        items_below_y_down > items_below_y_up
    };

    for (i, (header, _week_num)) in week_headers.iter().enumerate() {
        let start_y = header.y;
        let end_y = if i + 1 < week_headers.len() {
            week_headers[i + 1].0.y
        } else if y_increases_down {
            f64::MAX
        } else {
            0.0
        };

        // Define range [min, max]
        // let (min_y, max_y) = if start_y < end_y {
        //     (start_y, end_y)
        // } else {
        //     (end_y, start_y)
        // };

        // Filter items for this week
        // We want items strictly between the headers (or header and page edge)
        // But we must include the header line itself if we want to parse it?
        // Actually we pass `week_items` to `parse_week_items`.
        // `parse_week_items` needs the day headers which are usually below the Week header.

        // If Y increases down: Header is at min_y. Content is > min_y and < max_y.
        // If Y increases up: Header is at max_y. Content is < max_y and > min_y.

        let week_items: Vec<&TextItem> = items
            .iter()
            .filter(|item| {
                if y_increases_down {
                    item.y >= start_y && item.y < end_y
                } else {
                    item.y <= start_y && item.y > end_y
                }
            })
            .collect();

        // Extract "Week X" from header
        let week_name = if let Some(mat) = week_regex.find(&header.text) {
            mat.as_str().to_string()
        } else {
            "Unknown Week".to_string()
        };

        let lessons = parse_week_items(&week_items);

        // Try to extract student name and form from the page
        let (student_name, form) = extract_student_info(&week_items);

        if !lessons.is_empty() {
            weeks.push(Week {
                lessons,
                week_name,
                student_name,
                form,
            });
        }
    }

    weeks
}

fn parse_week_items(items: &[&TextItem]) -> Vec<Lesson> {
    let mut lessons = Vec::new();

    // 1. Find Day Headers to establish X columns
    let days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"];
    let mut day_cols: Vec<(usize, f64)> = Vec::new(); // (day_index, x_center)

    for (i, day) in days.iter().enumerate() {
        if let Some(header) = items.iter().find(|item| {
            item.text.trim().eq_ignore_ascii_case(day)
                || item.text.to_lowercase().contains(&day.to_lowercase())
        }) {
            day_cols.push((i, header.x));
            // println!("  Found Day: {} at X={}", day, header.x);
        }
    }

    if day_cols.is_empty() {
        // println!("  WARNING: No day headers found! Checking first few items:");
        // for item in items.iter().take(10) {
        //     println!("    '{}'", item.text);
        // }
        return lessons;
    }

    // 2. Find Period Rows (Y coordinates)
    // We look for markers and group them by period index.
    // Markers: L1..L5, PD.
    // We map them to period indices 0..5 (PD=0, L1=1, L2=2, L3=3, L4=4, L5=5)
    let marker_map = [
        ("PD", 0),
        ("Reg", 0),
        ("L1", 1),
        ("1", 1),
        ("L2", 2),
        ("2", 2),
        ("L3", 3),
        ("3", 3),
        ("L4", 4),
        ("4", 4),
        ("L4/", 4),
        ("L5", 5),
        ("5", 5),
    ];

    let mut period_rows: Vec<(usize, f64)> = Vec::new(); // (period_index, y_center)

    for (marker_text, period_idx) in marker_map.iter() {
        // Find all items matching this marker
        let matching_items: Vec<&f64> = items
            .iter()
            .filter(|item| {
                let text = item.text.trim();
                text == *marker_text ||
                // Also match if text contains the marker (e.g., "PD" in larger text)
                (marker_text.len() == 2 && text.starts_with(marker_text))
            })
            .map(|item| &item.y)
            .collect();

        if !matching_items.is_empty() {
            // Average Y
            let avg_y: f64 =
                matching_items.iter().cloned().sum::<f64>() / matching_items.len() as f64;
            // Only add if we haven't already added this period index
            if !period_rows.iter().any(|(idx, _)| idx == period_idx) {
                period_rows.push((*period_idx, avg_y));
            }
        }
    }

    // 3. Iterate Grid (Days x Periods)
    // Pre-compile teacher regex so it's not recreated inside the inner loop
    let teacher_regex_filter = Regex::new(r"^(Mr|Ms|Mrs|Miss)\s+.*$").unwrap();
    for (day_idx, day_x) in &day_cols {
        for (period_idx, period_y) in &period_rows {
            // Define cell bounds
            // We look for items near (day_x, period_y)
            // For cell content (subject, room, class): Y +/- 25
            // For teachers: Y tolerance needs to be larger (they're positioned below)
            // So we'll use a two-pass approach

            // First pass: get main cell items (subject, room, class code)
            let main_items: Vec<&&TextItem> = items
                .iter()
                .filter(|item| {
                    (item.x - day_x).abs() < 45.0 &&
                    (item.y - period_y).abs() < 25.0 &&
                    // Exclude markers and day headers
                    !days.iter().any(|d| item.text.trim().eq_ignore_ascii_case(d)) &&
                    !marker_map.iter().any(|(m, _)| item.text.trim() == *m)
                })
                .collect();

            // Second pass: find teachers in a slightly wider Y range, but only below the period marker
            let teacher_items: Vec<&&TextItem> = items
                .iter()
                .filter(|item| {
                    (item.x - day_x).abs() < 45.0 &&
                    item.y > *period_y && // Only below the period marker
                    (item.y - period_y).abs() < 35.0 &&
                    teacher_regex_filter.is_match(item.text.trim())
                })
                .collect();

            // Combine both sets
            let mut cell_items: Vec<&&TextItem> = main_items;
            cell_items.extend(teacher_items);

            if !cell_items.is_empty() {
                let lesson = parse_lesson_content(cell_items, *day_idx, *period_idx);
                lessons.push(lesson);
            }
        }
    }

    lessons
}

fn parse_lesson_content(items: Vec<&&TextItem>, day_index: usize, period_index: usize) -> Lesson {
    // Sort by Y (top to bottom), then by X (left to right)
    let mut sorted_items = items.clone();
    sorted_items.sort_by(|a, b| {
        a.y.partial_cmp(&b.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
    });

    let mut subject_parts: Vec<String> = Vec::new();
    let mut room = "Unknown".to_string();
    let mut teacher = "Unknown".to_string();
    let mut class_code = String::new();

    let room_regex = Regex::new(r"^[A-Z]{2,3}\d+[A-Z]?$").unwrap(); // e.g. SC8, HU5, MA3 - strict format
    let teacher_regex = Regex::new(r"^(Mr|Ms|Mrs|Miss)\s+.*$").unwrap();
    let class_regex = Regex::new(r"^\d[A-Z].*$").unwrap(); // e.g. 8A1/Co
    let days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"];

    // Words that are location indicators (not room codes, not part of subject)
    let location_indicators = ["DEFAULT", "DS"];

    for item in sorted_items {
        let text = item.text.trim();
        if text.is_empty() {
            continue;
        }

        // Skip day names if they accidentally got included
        if days.iter().any(|d| text.eq_ignore_ascii_case(d)) {
            continue;
        }

        // Skip location indicator words (DEFAULT, DS)
        if location_indicators.contains(&text) {
            continue;
        }

        if room_regex.is_match(text) && room == "Unknown" {
            // Only capture first room code found, excluding common false positives
            room = text.to_string();
        } else if teacher_regex.is_match(text) {
            teacher = text.to_string();
        } else if class_regex.is_match(text) {
            class_code = text.to_string();
        } else {
            // Accumulate subject parts
            subject_parts.push(text.to_string());
        }
    }

    // Join subject parts with spaces
    let subject = if subject_parts.is_empty() {
        "Unknown".to_string()
    } else {
        subject_parts.join(" ")
    };

    Lesson {
        subject,
        room,
        teacher,
        class_code,
        day_index,
        period_index,
    }
}

fn extract_student_info(items: &[&TextItem]) -> (Option<String>, Option<String>) {
    // Look for student name and form code as separate items
    // Name is typically "Firstname Lastname" and form is like "11RD" or "917"
    // Accept common forms such as:
    //  - Digits only: "917", "1017"
    //  - Digits with letters: "11RD", "10A"
    //  - 2-4 digit starting codes with optional letters/digits afterwards
    let form_in_parens_regex = Regex::new(r"^(.+?)\s*\(([0-9]{2,4}[A-Z0-9]*)\)$").unwrap();
    let form_code_regex = Regex::new(r"^[0-9]{2,4}[A-Z0-9]*$").unwrap(); // Like 11RD, 917, 1017, 10A

    if items.is_empty() {
        return (None, None);
    }

    // Sort by Y position (top to bottom), then X (left to right)
    let mut sorted = items.to_vec();
    sorted.sort_by(|a, b| {
        a.y.partial_cmp(&b.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
    });

    let mut student_name = None;
    let mut form = None;

    // Excluded words that are never student names
    let excluded = [
        "Week",
        "Term",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Page",
        "of",
        "Personal",
        "Development",
        "Intervention",
    ];

    // First pass: look for combined "Name (Form)" pattern
    for item in sorted.iter().take(50) {
        let text = item.text.trim();

        if let Some(cap) = form_in_parens_regex.captures(text) {
            let name = cap[1].trim();
            if name.len() > 3 {
                return (Some(name.to_string()), Some(cap[2].to_string()));
            }
        }
    }

    // Second pass: look for separate name and form items near the top
    for (i, item) in sorted.iter().take(50).enumerate() {
        let text = item.text.trim();

        if text.is_empty() || excluded.iter().any(|&e| text.contains(e)) {
            continue;
        }

        // Check if this is a form code
        if form.is_none() && form_code_regex.is_match(text) {
            form = Some(text.to_string());

            // Look for name nearby - check previous items on similar Y coordinate
            if student_name.is_none() {
                for j in (0..i).rev() {
                    let prev = sorted[j];
                    let prev_text = prev.text.trim();

                    // Check if on similar Y (same line) and looks like a name
                    if (prev.y - item.y).abs() < 5.0
                        && prev_text.len() > 3
                        && prev_text.contains(' ')
                        && !excluded.iter().any(|&e| prev_text.contains(e))
                        && !prev_text.starts_with("Mr")
                        && !prev_text.starts_with("Ms")
                        && !prev_text.starts_with("Mrs")
                        && !prev_text.starts_with("Miss")
                    {
                        student_name = Some(prev_text.to_string());
                        break;
                    }
                }
            }
        }

        // If we found both, stop
        if student_name.is_some() && form.is_some() {
            break;
        }
    }

    (student_name, form)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(x: f64, y: f64, text: &str) -> TextItem {
        TextItem {
            x,
            y,
            text: text.to_string(),
        }
    }

    #[test]
    fn parse_lesson_with_room_and_teacher() {
        let src = [
            make_item(100.0, 100.0, "Personal"),
            make_item(100.0, 110.0, "Development"),
            make_item(100.0, 120.0, "Intervention"),
            make_item(100.0, 130.0, "HU9"),
            make_item(100.0, 145.0, "Ms Test A"),
        ];

        let refs: Vec<&TextItem> = src.iter().collect();
        let refsrefs: Vec<&&TextItem> = refs.iter().collect();

        let lesson = parse_lesson_content(refsrefs, 0, 0);
        assert_eq!(lesson.subject, "Personal Development Intervention");
        assert_eq!(lesson.room, "HU9");
        assert_eq!(lesson.teacher, "Ms Test A");
    }

    #[test]
    fn parse_lesson_detects_classcode() {
        let src = [
            make_item(100.0, 200.0, "Science"),
            make_item(100.0, 210.0, "8A1/Co"),
            make_item(100.0, 220.0, "Mr Test B"),
        ];

        let refs: Vec<&TextItem> = src.iter().collect();
        let refsrefs: Vec<&&TextItem> = refs.iter().collect();

        let lesson = parse_lesson_content(refsrefs, 1, 2);
        assert_eq!(lesson.subject, "Science");
        assert_eq!(lesson.class_code, "8A1/Co");
        assert_eq!(lesson.teacher, "Mr Test B");
    }

    #[test]
    fn extract_student_info_parens() {
        let src = [make_item(10.0, 10.0, "Alex Testington (11XX)")];
        let items: Vec<&TextItem> = src.iter().collect();

        let (name, form) = extract_student_info(&items);
        assert_eq!(name.unwrap(), "Alex Testington");
        assert_eq!(form.unwrap(), "11XX");
    }

    #[test]
    fn extract_student_info_separate() {
        let src = [
            make_item(10.0, 10.0, "Alex Testington"),
            make_item(10.0, 12.0, "11XX"),
        ];
        let items: Vec<&TextItem> = src.iter().collect();

        let (name, form) = extract_student_info(&items);
        assert_eq!(name.unwrap(), "Alex Testington");
        assert_eq!(form.unwrap(), "11XX");
    }

    #[test]
    fn extract_student_info_parens_numeric() {
        let src = [make_item(10.0, 10.0, "Alex Testington (917)")];
        let items: Vec<&TextItem> = src.iter().collect();

        let (name, form) = extract_student_info(&items);
        assert_eq!(name.unwrap(), "Alex Testington");
        assert_eq!(form.unwrap(), "917");
    }

    #[test]
    fn extract_student_info_separate_alpha_num() {
        let src = [
            make_item(10.0, 10.0, "Ann Example"),
            make_item(10.0, 13.0, "10A"),
        ];
        let items: Vec<&TextItem> = src.iter().collect();

        let (name, form) = extract_student_info(&items);
        assert_eq!(name.unwrap(), "Ann Example");
        assert_eq!(form.unwrap(), "10A");
    }
}
