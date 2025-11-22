# TODO

This document tracks planned features, enhancements, and outstanding tasks for the Bromcom Timetable Formatter project.

## High Priority

### Documentation & Developer Experience

- [ ] **Set up GitHub Pages for rustdoc API documentation**
  - Configure GitHub Actions workflow to build and deploy rustdoc to `gh-pages` branch
  - Ensure automatic updates on releases
  - Add link to README

- [ ] **Create minimal synthetic test PDF**
  - Build a simple PDF with known coordinate structure for testing parser edge cases
  - Document expected parsing behavior
  - Use for regression testing

- [ ] **Document testing strategy**
  - Explain unit vs integration test approach
  - Define coverage targets (aim for >80% on core logic)
  - Add guide for writing new tests
  - Document test fixtures and their purpose

### Features & Enhancements

- [ ] **Add student name extraction from PDF**
  - Currently requires manual `--student-name` flag
  - Parse from PDF header/metadata when available
  - Fallback to CLI flags when extraction fails

- [ ] **Multi-week comparison view**
  - Generate side-by-side SVG comparing Week 1 vs Week 2
  - Highlight differences between weeks
  - Useful for students with alternating schedules

- [ ] **Interactive HTML output with tooltip details**
  - Convert SVG to interactive HTML/CSS/JS
  - Hover over lessons to see additional details
  - Click to expand/collapse information
  - Print-friendly CSS

- [ ] **Export to iCal format**
  - Generate `.ics` files from parsed timetable
  - Support for recurring events
  - Include room locations and teacher names
  - Enable import to Google Calendar, Outlook, etc.

- [ ] **Automated email distribution**
  - Send generated timetables via email
  - Support for batch processing multiple students
  - Template-based email content
  - Schedule regular updates

- [ ] **Web service API for on-demand generation**
  - REST API endpoint accepting PDF uploads
  - Return generated SVG/HTML/iCal
  - Rate limiting and authentication
  - Docker container for easy deployment

- [ ] **Support for other timetable PDF formats**
  - Extend parser to handle non-Bromcom formats
  - Configurable parsing rules
  - Format detection heuristics
  - Community-contributed format plugins

## Medium Priority

### Parser Improvements

- [ ] **Improve coordinate detection heuristics**
  - Better Y-direction auto-detection
  - Handle more PDF variations
  - Reduce reliance on fixed tolerance values
  - Add confidence scoring for parsed data

- [ ] **Support for merged cells and special events**
  - Handle cells spanning multiple periods
  - Parse special event markers (assemblies, trips, etc.)
  - Custom rendering for non-standard lessons

- [ ] **Better error messages and diagnostics**
  - Include PDF page/coordinate info in errors
  - Suggest fixes for common parsing failures
  - Validation warnings before rendering

### Rendering Enhancements

- [ ] **Map aspect ratio preservation**
  - Improve map scaling algorithm
  - Maintain proper proportions
  - Support different map sizes/orientations

- [ ] **Customizable output dimensions**
  - Support for different paper sizes (A3, Letter, etc.)
  - Configurable DPI for high-resolution printing
  - SVG vs PNG output options

- [ ] **Accessibility improvements**
  - WCAG-compliant color contrast
  - Screen reader-friendly SVG structure
  - Alternative text for visual elements
  - High-contrast mode support

### Configuration

- [ ] **Configuration validation**
  - Check for missing map IDs before rendering
  - Validate color formats and contrast
  - Warn about unreferenced room prefixes
  - Schema validation for TOML

- [ ] **Configuration presets**
  - Pre-built configs for common schools
  - Easy color scheme switching
  - Template configurations

## Low Priority

### Developer Tools

- [ ] **VSCode extension for config editing**
  - Syntax highlighting for config.toml
  - Autocomplete for room prefixes
  - Live preview of color choices

- [ ] **Debug mode with visualization**
  - Show parsed coordinates overlaid on PDF
  - Visualize text grouping decisions
  - Export intermediate parsing stages

### Performance

- [ ] **Parallel processing for multiple PDFs**
  - Batch process entire class directories
  - Progress reporting
  - Error handling for failed individual PDFs

- [ ] **Caching for map processing**
  - Cache processed/highlighted maps
  - Reduce re-processing overhead
  - Invalidate cache on config changes

## Completed (Move to CHANGELOG on release)

- [x] Initial CLI + core library with PDF parsing
- [x] Room-to-department mapping with colors
- [x] Per-week/day/period overrides
- [x] SVG timetable rendering with embedded maps
- [x] Cross-platform release builds (Linux, macOS, Windows, FreeBSD)
- [x] GPG-signed commits requirement
- [x] Conventional Commits enforcement
- [x] Community documentation (CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, SUPPORT)
- [x] GitHub Actions CI/CD pipelines
- [x] Pre-commit hooks for code quality

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for how to propose new features or work on existing TODO items.

When working on a TODO item:
1. Open an issue referencing this TODO
2. Create a feature branch (e.g., `feat/ical-export`)
3. Update this file to mark item as "In Progress" with issue link
4. Submit PR when complete
5. Move item to "Completed" section with PR link
