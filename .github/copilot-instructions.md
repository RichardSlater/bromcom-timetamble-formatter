# Copilot Instructions: Bromcom Timetable Formatter

## Project Overview

This is a Rust workspace project that parses Bromcom PDF timetables and generates formatted SVG visualizations. The output includes a color-coded weekly timetable and an integrated school map showing highlighted department locations.

## GPG Signing

GPG Signing is an absolute requirement, under not circumstances are you to disable code signing even if the signing fails, simply provide the command for the user to commit the files with GPG signing enabled.

## Conventional Commits

All commits must follow the [Conventional Commits](https://www.conventionalcommits.org/) specification (e.g., `feat: add map overlay`, `fix: correct renderer panic`). Ensure every commit message includes the proper type, optional scope, and succinct description so automated tooling can parse changelogs accurately.

## Architecture

### Workspace Structure

- **`crates/core`**: Library containing all business logic
  - `config.rs`: Configuration loading and room-to-department mapping
  - `parser.rs`: PDF parsing and text extraction from Bromcom PDFs
  - `processor.rs`: SVG map manipulation and highlighting
  - `renderer.rs`: Timetable SVG generation
- **`crates/cli`**: Binary for command-line execution

### Key Dependencies

- `lopdf`: PDF parsing and text extraction
- `serde`, `toml`: Configuration management
- `svg`: Programmatic SVG generation
- `roxmltree`: XML/SVG parsing and manipulation
- `clap`: CLI argument parsing
- `regex`: Pattern matching for parsing

## Data Flow

1. **Input**: Bromcom PDF timetable → Parser extracts lessons with coordinates
2. **Configuration**: TOML file maps room prefixes → colors & map IDs, defines overrides
3. **Override Application**: Config overrides are applied to parsed lessons
4. **Processing**: Lessons organized by day/period, map SVG highlighted
5. **Output**: Combined SVG with timetable grid + school map

## Configuration (`config.toml`)

### Room Mappings

Maps room prefixes to visual styling with separate foreground and background colors:

```toml
[[mappings]]
prefix = "MA"              # Room prefix (MA1, MA2, etc.)
bg_color = "#fcdcd8"       # Background color for label sidebar and map
fg_color = "#e8a490"       # Foreground/text color for class code labels
map_id = "Maths_Rooms"     # SVG element ID in map file
label = "Maths"            # Display label

[[mappings]]
prefix = "SC"
bg_color = "#fad7e6"       # Light pink background
fg_color = "#e68cb8"       # Darker pink text
map_id = "Science_Rooms"
label = "Science"
```

**Note**: The old `color` field is still supported as an alias for `bg_color` for backward compatibility. If `fg_color` is not specified, it defaults to `#231f20` (dark gray).

### Lesson Overrides

Override specific lessons by week, day, and period. All fields except `week`, `day`, and `period` are optional:

```toml
[[overrides]]
week = 2                   # Week number (1-based)
day = "Wednesday"          # Day name (Monday-Friday, case-insensitive)
period = "L3"              # Period (PD, L1-L5)
subject = "Geography"      # Override subject (optional)
room = "HU3"               # Override room (optional)
teacher = "Mr Smith"       # Override teacher (optional)
class_code = "HU9"         # Override class code (optional)

# Partial override example - only change teacher
[[overrides]]
week = 1
day = "Monday"
period = "L1"
            teacher = "Mr Test B"    # Only teacher is overridden
```

**Override Behavior**:

- Overrides are applied after PDF parsing, before rendering
- Only specified fields are modified; others remain from PDF
- Week numbers are 1-based (Week 1, Week 2, etc.)
- Day names support full names or abbreviations (Monday/Mon, Tuesday/Tue, etc.)
- Period must be one of: PD, L1, L2, L3, L4, L5
- If no matching lesson is found, a warning is printed to console

## Timetable Format Requirements

### Layout Structure (Target Output)

- **Header**: Student name and form (e.g., "Alex Testington (11XX)"), Week identifier
- **Grid**: 5 columns (Mon-Fri) × 6 rows (PD + L1-L5)
- **Breaks**:
  - Break row after L2 (11:00-11:30)
  - Lunch row after L4 (13:30-14:10)

### Cell Content (In Order)

1. **Subject** (bold, larger font) - e.g., "Personal Dev. Intervention", "French", "Mathematics"
2. **Class code** (right-aligned label) - e.g., "HU9", "LA4", "MA3"
3. **Room code** (centered, bold, colored background) - e.g., "HU5", "LA4", "MA3"
4. **Teacher** (smaller text) - e.g., "Ms Test A", "Mr P Ball"

### Styling Specifications

- **Fonts**:
  - Subject: Bahnschrift regular, 11px, font-weight 400
  - Detail (teacher, class): Bahnschrift Light, 11px, font-weight 300
  - Room: Bahnschrift SemiBold, large size, font-weight 600
  - Label (class code): Bahnschrift SemiBold, 11px, font-weight 600
- **Colors**:
  - Text: `#231f20` (dark gray/black)
  - Cell borders: `#231f20`, stroke-miterlimit: 10
  - Cell backgrounds: Department colors from config
  - Break/Lunch rows: Light gray `#eeeeee`
- **Alignment**:
  - Subject/Teacher: Left-aligned
  - Room: Center-aligned (both horizontal and vertical)
  - Class code: Right-aligned with vertical label
- **Cell dimensions**: Must accommodate break (30px) and lunch (50px) rows

### Special Layout Elements

- **PD Row**: First row, typically "Personal Dev. Intervention"
- **Break Row**: Gray background, centered "Break (11:00 - 11:30)" label
- **Lunch Row**: Gray background, centered "Lunch (13:30 - 14:10)" label
- **Footer**: "Updated: [date]" in bottom-right corner

## Parser Behavior (`parser.rs`)

### Text Extraction

- Extracts text with `(x, y)` coordinates from PDF using lopdf
- **Special decoding**: Bromcom uses character offset (+29) - decode with `decode_bromcom_text()`
- Handles multiple PDF pages for different weeks

### Grid Reconstruction

1. **Week Detection**: Regex `Week\s+(\d+)` to find week boundaries
2. **Y-direction handling**: Auto-detect if Y-coordinates increase down or up
3. **Day Headers**: Detect "Monday" through "Friday" to establish columns
4. **Period Markers**: Find "L1"-"L5", "PD", "Reg" to establish rows
5. **Cell Tolerance**: X ±45px, Y ±25px to group text into cells

### Lesson Parsing

- **Room pattern**: `^[A-Z]{2,}\d+.*$` (e.g., SC8, HU5, MA3)
- **Teacher pattern**: `^(Mr|Ms|Mrs|Miss)\s+.*$`
- **Subject**: First text item not matching other patterns
- **Period mapping**: 0-5 for PD, L1-L5

## Renderer Behavior (`renderer.rs`)

### Current Implementation

- A4 dimensions: 794×1123px (210mm×297mm)
- Timetable occupies top 600px
- Map embedded below with auto-scaling
- CSS styles injected via `<defs><style>`
- Grid with dynamic row heights accounting for break/lunch gaps

### Required Improvements (Based on Diagram)

1. **Add student name/form header** at top of timetable
2. **Add week label** ("Week 2") prominently
3. **Vertical class code labels** on right side of cells
4. **Period labels** (PD, L1-L5) on left side
5. **Time labels** for breaks/lunch
6. **Footer with update date**
7. **Proper text alignment**:
   - Room codes centered vertically and horizontally
   - Class codes rotated 90° on right edge
   - Teacher/subject left-aligned

## Map Processing (`processor.rs`)

### SVG Manipulation

- Loads school map SVG using `roxmltree`
- Finds elements by `id` or `data-name` attributes matching `map_id`
- Injects `fill` attributes to highlight departments
- Applies colors from config to all descendant shapes
- Deduplicates highlights by department

## Common Patterns

### Error Handling

- Use `thiserror` for custom error types
- Propagate with `?` operator
- Provide context with `anyhow::Context`

### Testing Strategy

- Unit tests for individual parsers (room regex, teacher regex)
- Integration tests with sample PDF pages
- Visual regression tests for SVG output

## Code Style

### Rust Conventions

- Use `snake_case` for functions and variables
- Use `CamelCase` for types and structs
- Prefer `Result<T, E>` over panics
- Use `Option` for nullable values
- Leverage iterator chains over loops where readable

### Documentation

- Module-level docs explaining purpose
- Struct field docs for non-obvious fields
- Function docs for public API with examples
- Inline comments for complex algorithms (especially coordinate math)

## Known Issues & Limitations

1. **Coordinate detection**: Relies on heuristics for Y-direction (some PDFs may vary)
2. **Text grouping**: Fixed tolerance values may need tuning per PDF source
3. **Map scaling**: Currently basic - may need aspect ratio preservation
4. **Font availability**: SVG output assumes Bahnschrift font is available
5. **Week detection**: Assumes "Week N" format - other formats may fail

## Development Workflow

### Building

```bash
cargo build --release
```

### Running

```bash
./target/release/timetable_cli \
  --input input/timetable.pdf \
  --config config.toml \
  --map resources/map.svg \
  --output output/
```

### Testing

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

## Future Enhancements (from TODO.md)

- [ ] Add student name extraction from PDF
- [ ] Multi-week comparison view
- [ ] Interactive HTML output with tooltip details
- [ ] Export to iCal format
- [ ] Automated email distribution
- [ ] Web service API for on-demand generation
- [ ] Support for other timetable PDF formats

## When Writing Code

### For Parser Changes

- Always test with actual Bromcom PDFs
- Validate coordinate grouping with debug prints
- Handle edge cases (empty cells, merged cells, special events)

### For Renderer Changes

- Reference the attached diagram for exact layout
- Test SVG output in multiple browsers
- Verify print layout at A4 size
- Ensure accessibility (alt text, semantic structure)

### For Config Changes

- Update example config.toml
- Document new fields in comments
- Maintain backward compatibility where possible

### For Map Processing

- Preserve original SVG structure
- Test with various SVG editors (Inkscape, Adobe Illustrator)
- Handle missing map IDs gracefully (log warning, don't crash)

### For Running Python

- Use the PyLance MCP server
- Use a virtual environment for dependencies
- Ensure compatibility with Python 3.12+

## Dependencies to Respect

- **lopdf**: PDF structure may change between versions - pin version
- **svg crate**: Limited XML manipulation - we manually inject map content
- **roxmltree**: Read-only parser - we use string manipulation for edits
