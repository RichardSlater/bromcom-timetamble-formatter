# Examples and Usage Guide

This directory contains example files and comprehensive guidance for using the Bromcom Timetable Formatter.

## Table of Contents

- [Example Files](#example-files)
- [Bromcom PDF Format](#bromcom-pdf-format)
- [Creating School Map SVGs](#creating-school-map-svgs)
- [Color Selection Guidelines](#color-selection-guidelines)
- [Common Workflows](#common-workflows)

## Example Files

### Sample PDF

A sample anonymized Bromcom timetable PDF is available at:

```
input/Sample_Student_Timetable.pdf
```

This PDF demonstrates the expected format with:
- Two weeks of timetable data
- Student name and form information
- Multiple subjects across different rooms
- Standard Bromcom coordinate-based text layout

**Note**: Real student data has been anonymized using clearly fictional names.

### School Map SVG

An example school map SVG is included at:

```
resources/SchoolMap.svg
```

The map shows:
- Department room blocks with labeled IDs
- Proper SVG structure for highlighting
- Example of how to organize map elements

### Configuration

See `config.toml` in the repository root for a complete configuration example with:
- Multiple room prefix mappings
- Color schemes for different departments
- Override examples for correcting parsing errors

### Generated Output

Example output SVG files can be found in:

```
output/Week_1_1.svg
output/Week_2_2.svg
```

These demonstrate the final formatted timetable with:
- Color-coded cells based on room mappings
- Embedded highlighted school map
- Student name and week identifier headers
- Break and lunch period rows

## Bromcom PDF Format

### Expected Structure

Bromcom exports timetables as PDFs with the following characteristics:

1. **Coordinate-Based Text**:  every text element has an `(x, y)` position
2. **No Table Structure**: PDFs don't preserve HTML/table semantics
3. **Week Markers**: Text like "Week 1", "Week 2" separates different weeks
4. **Day Headers**: "Monday", "Tuesday", "Wednesday", "Thursday", "Friday"
5. **Period Labels**: "PD" (Personal Development), "L1" through "L5" (Lessons 1-5)
6. **Lesson Data**: Subject, Room, Teacher, Class Code grouped by proximity

### Text Encoding

Bromcom PDFs use a non-standard character encoding with a +29 offset. The parser automatically handles this:

```rust
// Parser automatically decodes: 
// PDF byte 'a' + 29 → actual character
fn decode_bromcom_text(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            let code = c as u32;
            if code >= 29 { char::from_u32(code - 29).unwrap_or(c) } 
            else { c }
        })
        .collect()
}
```

### Coordinate System

PDFs may have Y-coordinates that increase downward OR upward. The parser auto-detects this:

- **Downward Y**: Most common (0 at top, increasing downward)
- **Upward Y**: Some PDFs (0 at bottom, increasing upward)

The parser analyzes coordinate distribution and normalizes to a consistent orientation.

### Text Grouping

Text items are grouped into cells based on proximity:

- **X-axis tolerance**: ±45px (horizontal grouping)
- **Y-axis tolerance**: ±25px (vertical grouping)

Text within these tolerances is considered part of the same cell.

### Pattern Recognition

The parser uses regex patterns to classify text:

| Pattern | Regex | Examples |
|---------|-------|----------|
| Room Code | `^[A-Z]{2,}\d+.*$` | MA3, SC8, HU5, IT12 |
| Teacher | `^(Mr|Ms|Mrs|Miss)\s+.*$` | Ms Test A, Mr P Ball |
| Period | `^(PD|L[1-5]|Reg)$` | PD, L1, L2, L3, L4, L5 |
| Day | `Monday|Tuesday|...` | Monday, Tuesday, etc. |
| Week | `Week\s+(\d+)` | Week 1, Week 2 |

Everything else is considered a subject name.

## Creating School Map SVGs

### Requirements

Your school map SVG must:

1. Be valid XML/SVG format
2. Have identifiable elements for each department
3. Use `id` or `data-name` attributes for element identification

### Basic Structure

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1000 800">
  <!-- Maths Department -->
  <g id="Maths_Rooms">
    <rect x="100" y="100" width="150" height="100" fill="#ddd" />
    <text x="175" y="155">Maths</text>
  </g>
  
  <!-- Science Department -->
  <g id="Science_Rooms" data-name="Science">
    <rect x="300" y="100" width="150" height="100" fill="#ddd" />
    <text x="375" y="155">Science</text>
  </g>
  
  <!-- Add more departments... -->
</svg>
```

### Element Identification

The processor looks for elements with:

- `id` attribute (preferred): `<g id="Maths_Rooms">`
- `data-name` attribute (fallback): `<g data-name="Maths">`

These values must match the `map_id` in your `config.toml`.

### Naming Conventions

**Good ID examples**:
- `Maths_Rooms`
- `Science_Labs`
- `English_Block`
- `Music_x2C_Art_x2C_DT_x26_FT_Rooms` (URL-encoded special chars)

**Avoid**:
- Spaces in IDs (use underscores or hyphens)
- Special characters without URL encoding
- Generic names like `room1`, `area_A` (be descriptive)

### Grouping Rooms

Group related rooms under a common parent element:

```svg
<g id="Science_Labs">
  <!-- All science labs will be highlighted together -->
  <rect id="SC1" x="100" y="200" width="50" height="40" />
  <rect id="SC2" x="160" y="200" width="50" height="40" />
  <rect id="SC3" x="220" y="200" width="50" height="40" />
</g>
```

### Creating From School Floor Plans

1. **Start with a base image**: Scan or photograph floor plans
2. **Trace in vector editor**: Use Inkscape, Adobe Illustrator, or similar
3. **Add layers/groups**: Organize by department
4. **Label elements**: Add `id` attributes to each department group
5. **Simplify paths**: Reduce complexity for faster processing
6. **Export as SVG**: Save as plain SVG (not Inkscape SVG)
7. **Test highlights**: Run the tool and verify colors appear correctly

### SVG Editing Tools

**Free/Open Source**:
- [Inkscape](https://inkscape.org/) - Full-featured vector editor
- [Boxy SVG](https://boxy-svg.com/) - Simpler, web-based option
- [SVG-Edit](https://github.com/SVG-Edit/svgedit) - Online SVG editor

**Commercial**:
- Adobe Illustrator
- Affinity Designer
- CorelDRAW

### Testing Your Map

1. Create a minimal config with one mapping
2. Run the tool and check the output
3. Verify the correct elements are highlighted
4. Adjust `map_id` values if needed
5. Expand to all departments once working

## Color Selection Guidelines

### Accessibility Considerations

Choose colors that are:

1. **Distinguishable**: Clearly different from adjacent departments
2. **High Contrast**: Ensure text is readable against backgrounds
3. **Colorblind-Friendly**: Test with colorblind simulation tools
4. **Print-Friendly**: Work well in grayscale/black & white printing

### WCAG Contrast Standards

For text readability, aim for:

- **Normal Text**: Minimum 4.5:1 contrast ratio
- **Large Text**: Minimum 3:1 contrast ratio

Use tools like:
- [WebAIM Contrast Checker](https://webaim.org/resources/contrastchecker/)
- [Coolors Contrast Checker](https://coolors.co/contrast-checker)

### Recommended Color Palettes

**Pastel Scheme (default in config.toml)**:
```toml
# Maths - Pink
bg_color = "#fcdcd8"
fg_color = "#f0503f"

# English - Blue
bg_color = "#daeaf6"
fg_color = "#4799d4"

# Science - Magenta
bg_color = "#fad7e6"
fg_color = "#e93982"

# Humanities - Yellow
bg_color = "#faeed2"
fg_color = "#e8c570"
```

**Bold Scheme**:
```toml
# High contrast, vibrant colors
# Maths
bg_color = "#ff6b6b"
fg_color = "#2c003e"

# English
bg_color = "#4ecdc4"
fg_color = "#1a535c"

# Science
bg_color = "#ffe66d"
fg_color = "#4a4e69"
```

**Colorblind-Friendly Scheme**:
```toml
# Based on colorblind-safe palette
# Maths - Blue
bg_color = "#88ccee"
fg_color = "#004c6d"

# English - Orange
bg_color = "#ffcc88"
fg_color = "#7f4f00"

# Science - Green
bg_color = "#ccddaa"
fg_color = "#557722"
```

### Testing Colors

1. **Visual Test**: Check on screen in different lighting
2. **Print Test**: Print a sample on your target printer
3. **Colorblind Simulation**: Use tools like [Coblis](https://www.color-blindness.com/coblis-color-blindness-simulator/)
4. **Student Feedback**: Ask your target audience if colors are helpful

### Avoiding Color Fatigue

- Don't use more than 8-10 distinct colors
- Group similar departments with similar color families
- Use consistent color schemes across related subjects

## Common Workflows

### Workflow 1: First-Time Setup

1. Export timetable PDF from Bromcom
2. Create/obtain school map SVG with labeled departments
3. Create `config.toml` with room mappings
4. Run tool with sample data to test
5. Adjust colors and mappings as needed
6. Add overrides for any parsing errors

### Workflow 2: Weekly Updates

1. Export new PDF from Bromcom
2. Run tool with existing config
3. Check output for accuracy
4. Add overrides to config if needed
5. Regenerate with corrected config
6. Distribute SVG to student/family

### Workflow 3: New School Year

1. Update room mappings if departments moved
2. Update map SVG if building layout changed
3. Test with first timetable of the year
4. Document any new room prefixes discovered
5. Share updated config with other users

### Workflow 4: Bulk Processing

Process multiple student timetables:

```bash
#!/bin/bash
# Process all PDFs in a directory

for pdf in input/*.pdf; do
  student_name=$(basename "$pdf" .pdf)
  mkdir -p "output/$student_name"
  
  ./timetable_cli \
    --input "$pdf" \
    --config config.toml \
    --map resources/SchoolMap.svg \
    --output "output/$student_name"
  
  echo "Processed: $student_name"
done
```

### Workflow 5: Automated Distribution

Set up automated processing and email distribution:

1. Create script to check for new PDFs
2. Process with timetable_cli
3. Convert SVG to PDF (optional)
4. Email to recipients using sendmail/SMTP
5. Archive processed files

See [docs/TODO.md](../docs/TODO.md) for planned features like built-in email distribution.

## Troubleshooting Examples

### Example 1: Map Elements Not Highlighted

**Problem**: Config has `map_id = "Maths_Rooms"` but nothing highlights.

**Solution**:

1. Open `resources/SchoolMap.svg` in a text editor
2. Search for "Maths" - find the actual ID:
   ```svg
   <g id="maths-block" data-name="Maths Rooms">
   ```
3. Update config to match:
   ```toml
   map_id = "maths-block"  # OR
   map_id = "Maths Rooms"  # if using data-name
   ```

### Example 2: Wrong Room Prefix

**Problem**: Room "MA10" is being colored with "M" (Music) instead of "MA" (Maths).

**Solution**:

Order matters! Longer prefixes should be listed first OR the parser already prioritizes longest match. Check order in config:

```toml
# WRONG order (but actually parser handles this)
[[mappings]]
prefix = "M"   # Will never match if MA exists

[[mappings]]
prefix = "MA"  # Longer prefix

# CORRECT (parser auto-prioritizes longest)
# Either order works, parser chooses longest matching prefix
```

### Example 3: Lesson Not Found for Override

**Problem**: Override configured but warning says "No lesson found".

**Solution**:

1. Verify week number (1-based, not 0-based)
2. Check day spelling (case-insensitive but must be correct)
3. Confirm period format (PD, L1-L5, not "Lesson 1")
4. Check if PDF actually has a lesson in that slot
5. Look at console output for parsed lessons to debug

## Contributing Examples

Have a useful example, color scheme, or workflow? Contributions welcome!

1. Fork the repository
2. Add your example to this directory
3. Update this README with documentation
4. Submit a pull request

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

---

For more help, see:
- [README.md](../README.md) - Main documentation
- [ARCHITECTURE.md](../ARCHITECTURE.md) - Technical architecture
- [docs/TODO.md](../docs/TODO.md) - Planned features
- [SUPPORT.md](../SUPPORT.md) - Getting help
