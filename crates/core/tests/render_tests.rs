use std::fs;
use timetable_core::config::Config;
use timetable_core::parser::Week;
use timetable_core::renderer::render_timetable;

#[test]
fn render_timetable_without_map_produces_svg() {
    // Create a minimal Week struct with no lessons
    let week = Week {
        week_name: "Test Week".to_string(),
        lessons: Vec::new(),
        student_name: None,
        form: None,
    };

    // Create a minimal Config (no mappings needed for this test)
    let config = Config {
        mappings: Vec::new(),
        overrides: Vec::new(),
    };

    let mut out_path = std::env::temp_dir();
    out_path.push(format!(
        "timetable_test_output_week_{}.svg",
        std::process::id()
    ));
    let _ = fs::remove_file(&out_path);

    render_timetable(&week, &config, "", &out_path).expect("render should succeed");

    let svg = fs::read_to_string(&out_path).expect("read output");

    // When map_content is empty, renderer should still produce a valid svg string
    assert!(svg.contains("<svg"));
    // And should not contain nested <svg> for the map area (we expect only the root svg)
    let occurrences = svg.matches("<svg").count();
    assert_eq!(
        occurrences, 1,
        "expected only root <svg> when no map provided"
    );
}
