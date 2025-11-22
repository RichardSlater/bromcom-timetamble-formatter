#!/usr/bin/env python3
"""
PDF Anonymization Script for Bromcom Timetable Formatter

This script anonymizes PDF timetables by automatically detecting and replacing
names with clearly fictional alternatives. It uses pikepdf to manipulate PDF
content streams.

Usage:
    python3 scripts/anonymize_pdf.py input/original.pdf input/anonymized.pdf

The script restricts input/output paths to a repository base directory by
default. The base directory is auto-detected by searching upward for a
.git directory or a 'Cargo.toml' file; a custom `--base-dir` may be provided.
"""

import argparse
import errno
import sys
import re
from pathlib import Path
from collections import defaultdict
from typing import Dict, Set, Optional

try:
    import pikepdf
except ImportError:
    print("Error: pikepdf is required. Install it with: pip install pikepdf")
    sys.exit(1)

# Test fixture name generators
TEACHER_FIXTURES = [
    "Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot", "Golf", "Hotel",
    "India", "Juliet", "Kilo", "Lima", "Mike", "November", "Oscar", "Papa",
    "Quebec", "Romeo", "Sierra", "Tango", "Uniform", "Victor", "Whiskey",
    "X-ray", "Yankee", "Zulu"
]

FIRST_NAMES = [
    "Test", "Sample", "Demo", "Mock", "Fixture", "Example", "Placeholder",
    "Specimen", "Model", "Instance", "Case", "Trial"
]

LAST_NAMES = [
    "Data", "User", "Person", "Student", "Subject", "Entity", "Record",
    "Entry", "Item", "Object", "Element", "Unit"
]



def generate_replacement_name(original: str, used_names: Set[str], counter: Dict[str, int]) -> str:
    """
    Generate a replacement name with the same character count as the original.

    Args:
        original: Original name to replace
        used_names: Set of already-used replacement names
        counter: Counter for generating unique fixture names

    Returns:
        Replacement name with same character count
    """
    target_len = len(original)

    # Try different combinations until we find one with the right length
    for first in FIRST_NAMES:
        for last in LAST_NAMES:
            candidate = f"{first} {last}"
            if len(candidate) == target_len and candidate not in used_names:
                used_names.add(candidate)
                return candidate

            # Try padding with spaces
            if len(candidate) < target_len:
                padded = candidate + " " * (target_len - len(candidate))
                if padded not in used_names:
                    used_names.add(padded)
                    return padded

    # Fallback: use counter-based name
    idx = counter.get('student', 0)
    counter['student'] = idx + 1
    base = f"Student{idx:03d}"
    if len(base) <= target_len:
        result = base + " " * (target_len - len(base))
        used_names.add(result)
        return result

    return "X" * target_len  # Last resort


def generate_teacher_name(title: str, surname: str, used_names: Set[str], counter: Dict[str, int]) -> str:
    """
    Generate a teacher replacement name matching the pattern "Title Surname".

    Args:
        title: Teacher title (Mr, Ms, Mrs, Miss)
        surname: Original surname
        used_names: Set of already-used names
        counter: Counter for generating unique names

    Returns:
        Replacement in format "Title Fixture" with same character count
    """
    original = f"{title} {surname}"
    target_len = len(original)
    title_len = len(title)

    # Calculate how long the fixture surname needs to be
    surname_len = target_len - title_len - 1  # -1 for space

    # Try to find a fixture name with the right length
    for fixture in TEACHER_FIXTURES:
        if len(fixture) == surname_len:
            candidate = f"{title} {fixture}"
            if candidate not in used_names:
                used_names.add(candidate)
                return candidate

        # Try padding or truncating
        if len(fixture) < surname_len:
            padded_fixture = fixture + " " * (surname_len - len(fixture))
            candidate = f"{title} {padded_fixture}"
            if candidate not in used_names:
                used_names.add(candidate)
                return candidate
        elif len(fixture) > surname_len:
            truncated = fixture[:surname_len]
            candidate = f"{title} {truncated}"
            if candidate not in used_names:
                used_names.add(candidate)
                return candidate

    # Fallback: generate numbered fixture
    idx = counter.get('teacher', 0)
    counter['teacher'] = idx + 1
    fixture = f"Teacher{idx:02d}"

    if len(fixture) <= surname_len:
        fixture = fixture + " " * (surname_len - len(fixture))
    else:
        fixture = fixture[:surname_len]

    result = f"{title} {fixture}"
    used_names.add(result)
    return result


def detect_names_in_text(text: str) -> Dict[str, str]:
    """
    Detect names in decoded PDF text and generate replacements.
    Focuses on teacher names (Title + Name) and student names.

    Args:
        text: Decoded PDF text content

    Returns:
        Dictionary mapping original names to replacement names
    """
    replacements = {}
    used_names: Set[str] = set()
    counter: Dict[str, int] = defaultdict(int)

    # Pattern for teacher names: Title + optional Initial + Surname
    # More strict to avoid false positives
    teacher_pattern = r'\b(Mr|Ms|Mrs|Miss)\s+([A-Z])\s+([A-Z][a-z]{3,})\b'

    for match in re.finditer(teacher_pattern, text):
        title = match.group(1)
        initial = match.group(2)
        surname = match.group(3)

        original = f"{title} {initial} {surname}"
        # Don't replace if surname looks like a common word
        if surname.lower() not in ['week', 'page', 'form', 'room', 'lesson']:
            fake_surname = generate_teacher_name(title, f"{initial} {surname}", used_names, counter)
            # Keep the same format with initial
            fake_parts = fake_surname.split(None, 1)
            if len(fake_parts) == 2:
                replacements[original] = f"{title} {initial} {fake_parts[1]}"

    # Pattern for student names (First Last) - look for them near form codes or at page top
    # More conservative to avoid false positives
    # Try both with and without parentheses around form code
    student_context_pattern = r'([A-Z][a-z]{3,})\s+([A-Z][a-z]{3,})\s+\(?(\d{1,2}[A-Z]{1,3})\)?'

    for match in re.finditer(student_context_pattern, text):
        first_name = match.group(1)
        last_name = match.group(2)
        form_code = match.group(3)

        full_name = f"{first_name} {last_name}"
        # Generate replacement
        replacements[full_name] = generate_replacement_name(full_name, used_names, counter)

        # Also add the form code
        digit_part = ''.join(c for c in form_code if c.isdigit())
        letter_count = len(form_code) - len(digit_part)
        replacements[form_code] = digit_part + 'X' * letter_count

    # Pattern for form codes alone (digits + letters)
    form_pattern = r'\b(\d{1,2}[A-Z]{2,3})\b'

    for match in re.finditer(form_pattern, text):
        form_code = match.group(1)
        # Replace with same pattern: digits + X's
        digit_part = ''.join(c for c in form_code if c.isdigit())
        letter_count = len(form_code) - len(digit_part)
        replacement = digit_part + 'X' * letter_count
        replacements[form_code] = replacement

    return replacements


def encode_bromcom(text: str) -> str:
    """Encode text with Bromcom's -29 character offset (for writing to PDF)."""
    result = []
    for c in text:
        code = ord(c)
        # Use wrapping subtraction like Rust's wrapping_sub
        new_code = (code - 29) % 256
        result.append(chr(new_code))
    return ''.join(result)


def decode_bromcom(text: str) -> str:
    """Decode text with Bromcom's +29 character offset (for reading from PDF)."""
    result = []
    for c in text:
        code = ord(c)
        # Use wrapping addition like Rust's wrapping_add
        new_code = (code + 29) % 256
        result.append(chr(new_code))
    return ''.join(result)


def text_to_pdf_hex(text: str) -> str:
    """Convert text to PDF hex string format <XXXX...>."""
    hex_codes = ''.join(f'{ord(c):04X}' for c in text)
    return f'<{hex_codes}>'


def anonymize_text_content(content: bytes, replacements: Dict[str, str]) -> bytes:
    """
    Replace detected names with fictional ones in PDF content stream.
    Handles both normal text, Bromcom's -29 encoding, and hex-encoded strings.

    Args:
        content: Raw PDF content stream bytes
        replacements: Dictionary mapping original names to replacement names

    Returns:
        Anonymized content stream bytes
    """
    # Decode the content stream
    try:
        text = content.decode('latin-1')
    except UnicodeDecodeError:
        # If decoding fails, try with error handling
        text = content.decode('latin-1', errors='ignore')

    # Apply all replacements in multiple forms
    for real_name, fake_name in replacements.items():
        # 1. Replace normal text
        text = text.replace(real_name, fake_name)

        # 2. Replace Bromcom-encoded text (-29 offset)
        encoded_real = encode_bromcom(real_name)
        encoded_fake = encode_bromcom(fake_name)
        text = text.replace(encoded_real, encoded_fake)

        # 3. Replace hex-encoded strings (PDF format: <XXXX...>)
        hex_real = text_to_pdf_hex(real_name)
        hex_fake = text_to_pdf_hex(fake_name)
        text = text.replace(hex_real, hex_fake)

        # 4. Replace hex-encoded Bromcom strings
        hex_bromcom_real = text_to_pdf_hex(encoded_real)
        hex_bromcom_fake = text_to_pdf_hex(encoded_fake)
        text = text.replace(hex_bromcom_real, hex_bromcom_fake)

        # 5. Handle variations without spaces
        text = text.replace(real_name.replace(" ", ""), fake_name.replace(" ", ""))
        text = text.replace(encode_bromcom(real_name.replace(" ", "")),
                          encode_bromcom(fake_name.replace(" ", "")))
        text = text.replace(text_to_pdf_hex(real_name.replace(" ", "")),
                          text_to_pdf_hex(fake_name.replace(" ", "")))

    # Re-encode
    return text.encode('latin-1', errors='ignore')


def extract_text_from_pdf(pdf: pikepdf.Pdf) -> str:
    """
    Extract and decode all text content from PDF for name detection.

    Args:
        pdf: Opened pikepdf PDF object

    Returns:
        Combined decoded text from all pages
    """
    all_text = []

    for page in pdf.pages:
        if '/Contents' in page:
            contents = page['/Contents']

            if isinstance(contents, pikepdf.Array):
                for stream in contents:
                    if isinstance(stream, pikepdf.Stream):
                        data = stream.read_bytes()
                        try:
                            text = data.decode('latin-1', errors='ignore')
                            all_text.append(text)
                        except:
                            pass
            elif isinstance(contents, pikepdf.Stream):
                data = contents.read_bytes()
                try:
                    text = data.decode('latin-1', errors='ignore')
                    all_text.append(text)
                except:
                    pass

    combined = '\n'.join(all_text)

    # Decode the raw text with Bromcom decoding
    decoded_raw = decode_bromcom(combined)

    # Extract hex-encoded strings and decode them
    hex_pattern = r'<([0-9A-Fa-f]+)>'
    decoded_texts = []

    for match in re.finditer(hex_pattern, combined):
        hex_str = match.group(1)
        try:
            # Decode hex to characters
            chars = []
            for i in range(0, len(hex_str), 4):
                if i + 4 <= len(hex_str):
                    val = int(hex_str[i:i+4], 16)
                    if 0 < val < 256:
                        chars.append(chr(val))
            decoded_hex = ''.join(chars)

            # Apply Bromcom decoding (+29 offset)
            decoded_bromcom = decode_bromcom(decoded_hex)
            decoded_texts.append(decoded_bromcom)
        except:
            pass

    # Combine original, Bromcom-decoded raw text, and decoded hex strings for comprehensive name detection
    return '\n'.join([combined, decoded_raw] + decoded_texts)

    # Re-encode
    return text.encode('latin-1', errors='ignore')


def get_default_base_dir() -> Path:
    """Return a reasonable repository root by searching upward from this file.

    The function walks parent directories looking for a VCS marker ('.git') or
    a repository marker like 'Cargo.toml'. If none is found, it falls back to
    the parent of this script.
    """

    candidate = Path(__file__).resolve()

    # Check the candidate path itself first (avoid allocating a large list)
    if (candidate / '.git').exists() or (candidate / 'Cargo.toml').exists():
        return candidate

    # Then walk upward through parents lazily
    for parent in candidate.parents:
        if (parent / '.git').exists() or (parent / 'Cargo.toml').exists():
            return parent

    # Fallback: one level up from scripts/ if nothing obvious found
    return candidate.parents[1]


def sanitize_user_path(raw_path: str, base_dir: Path) -> Path:
    """Resolve user-supplied paths relative to a trusted base directory.

    Validates that the resolved path is located within the base directory to
    prevent path traversal and unintended access outside the workspace.
    """

    resolved_base = base_dir.resolve()
    candidate = Path(raw_path)

    if candidate.is_absolute():
        resolved_candidate = candidate.resolve()
    else:
        resolved_candidate = (resolved_base / candidate).resolve()

    try:
        resolved_candidate.relative_to(resolved_base)
    except ValueError as exc:
        raise ValueError(
            f"Path '{resolved_candidate}' is outside the allowed base directory {resolved_base}"
        ) from exc

    # Additional check: prevent symlink traversal outside base dir
    for parent in resolved_candidate.parents:
        if parent == resolved_base.parent:  # Stop before leaving base_dir's ancestor chain
            break
        # Only need to check ancestors down to resolved_base (inclusive)
        if parent.is_symlink():
            raise ValueError(
                f"Symlink detected in path component: '{parent}'. Refusing to access '{resolved_candidate}'."
            )
        if parent == resolved_base:
            break

    return resolved_candidate


def parse_args() -> argparse.Namespace:
    """Parse command-line arguments for the anonymizer script."""

    parser = argparse.ArgumentParser(
        description="Anonymize Bromcom PDFs while keeping file access within a trusted directory."
    )
    parser.add_argument(
        "input_pdf",
        help="Path to the original PDF (relative to --base-dir unless absolute within it)",
    )
    parser.add_argument(
        "output_pdf",
        help="Path where the anonymized PDF will be written (relative to --base-dir)",
    )
    parser.add_argument(
        "--base-dir",
        default=str(get_default_base_dir()),
        help=(
            "Trusted base directory that user-supplied paths must reside in. "
            "Defaults to the repository root."
        ),
    )

    return parser.parse_args()



def anonymize_pdf(input_path: Path, output_path: Path) -> None:
    """
    Anonymize a PDF file by auto-detecting and replacing names.

    Args:
        input_path: Path to the original PDF
        output_path: Path where anonymized PDF will be saved
    """
    print(f"Reading PDF from: {input_path}")

    # Open the PDF
    with pikepdf.open(input_path) as pdf:
        # Extract text and detect names
        print("Detecting names in PDF...")
        pdf_text = extract_text_from_pdf(pdf)
        replacements = detect_names_in_text(pdf_text)

        print(f"Found {len(replacements)} names to replace:")
        for original, replacement in sorted(replacements.items()):
            print(f"  '{original}' -> '{replacement}'")

        # Anonymize metadata
        if '/Info' in pdf.docinfo:
            for key in ['/Author', '/Title', '/Subject', '/Creator']:
                if key in pdf.docinfo:
                    # Apply replacements to metadata
                    metadata_text = str(pdf.docinfo[key])
                    for original, replacement in replacements.items():
                        metadata_text = metadata_text.replace(original, replacement)
                    pdf.docinfo[key] = metadata_text

        # Process each page
        for page_num, page in enumerate(pdf.pages, 1):
            print(f"Processing page {page_num}...")

            # Get the content stream
            if '/Contents' in page:
                contents = page['/Contents']

                # Handle both single content streams and arrays of streams
                if isinstance(contents, pikepdf.Array):
                    for stream in contents:
                        if isinstance(stream, pikepdf.Stream):
                            original_data = stream.read_bytes()
                            anonymized_data = anonymize_text_content(original_data, replacements)
                            stream.write(anonymized_data)
                elif isinstance(contents, pikepdf.Stream):
                    original_data = contents.read_bytes()
                    anonymized_data = anonymize_text_content(original_data, replacements)
                    contents.write(anonymized_data)

        # Save the anonymized PDF
        print(f"Saving anonymized PDF to: {output_path}")
        pdf.save(output_path)

    print("✓ Anonymization complete!")


def main():
    """Main entry point for the script."""
    # Parse arguments using argparse for proper validation
    args = parse_args()

    # Determine canonical repository root directory (trusted source)
    repo_root = get_default_base_dir()

    # Validate and sanitize the user-provided base_dir using the trusted repo_root
    try:
        base_dir = sanitize_user_path(args.base_dir, repo_root)
    except ValueError as e:
        print(f"Error: Invalid --base-dir: {e}")
        sys.exit(1)

    # Now sanitize input and output paths using the validated base_dir
    try:
        input_path = sanitize_user_path(args.input_pdf, base_dir)
        output_path = sanitize_user_path(args.output_pdf, base_dir)
    except ValueError as e:
        print(f"Error: {e}")
        sys.exit(1)

    # Verify input file exists and is readable
    if not input_path.exists():
        print(f"Error: Input file not found: {input_path}")
        sys.exit(1)

    if not input_path.is_file():
        print(f"Error: Input path is not a file: {input_path}")
        sys.exit(1)

    try:
        with open(input_path, 'rb') as f:
            pass  # Just verify we can open it
    except PermissionError:
        print(f"Error: Permission denied reading input file: {input_path}")
        sys.exit(1)
    except OSError as e:
        print(f"Error: Cannot access input file: {input_path} ({e})")
        sys.exit(1)

    # Create output directory if it doesn't exist
    resolved_output_parent = output_path.parent.resolve()
    try:
        resolved_output_parent.relative_to(base_dir)
    except ValueError:
        print(f"Error: Output directory '{resolved_output_parent}' is outside the allowed base directory '{base_dir}'")
        sys.exit(1)
    try:
        resolved_output_parent.mkdir(parents=True, exist_ok=True)
    except PermissionError:
        print(f"Error: Permission denied creating output directory: {resolved_output_parent}")
        sys.exit(1)
    except OSError as e:
        print(f"Error: Cannot create output directory: {resolved_output_parent} ({e})")
        sys.exit(1)

    # Perform the anonymization with detailed error handling
    try:
        anonymize_pdf(input_path, output_path)
    except PermissionError:
        print(f"Error: Permission denied writing output file: {output_path}")
        sys.exit(1)
    except OSError as e:
        if e.errno == errno.ENOSPC:
            print(f"Error: Disk full - cannot write output file: {output_path}")
        else:
            print(f"Error: File system error: {e}")
        sys.exit(1)
    except Exception as e:
        print(f"Error during anonymization: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
if __name__ == "__main__":
    main()
