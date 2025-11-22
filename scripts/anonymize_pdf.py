#!/usr/bin/env python3
"""
PDF Anonymization Script for Bromcom Timetable Formatter

This script anonymizes PDF timetables by replacing real names with clearly
fictional alternatives. It uses pikepdf to manipulate PDF content streams.

Usage:
    python3 scripts/anonymize_pdf.py input/original.pdf input/anonymized.pdf

Dependencies:
    pip install pikepdf

The script replaces common names and teacher titles with fictional equivalents
while preserving the PDF structure and coordinate information for parsing.
"""

import sys
import re
from pathlib import Path

try:
    import pikepdf
except ImportError:
    print("Error: pikepdf is required. Install it with: pip install pikepdf")
    sys.exit(1)


# Name replacement mappings - add more as needed
NAME_REPLACEMENTS = {
    # Student names
    "Amelia Slater": "Alex Testington",
    "Richard Slater": "Alex Testington",
    
    # Teacher names (examples - extend as needed)
    "Mr Smith": "Professor Sample",
    "Ms Jones": "Dr. Example",
    "Mrs Brown": "Ms. Demo",
    "Miss Wilson": "Miss Fictional",
    "Mr Johnson": "Mr. Placeholder",
    "Ms Davis": "Ms. Anonymous",
    
    # Form codes
    "11RD": "11XX",
    "9X1": "9YY",
}


def anonymize_text_content(content: bytes) -> bytes:
    """
    Replace real names with fictional ones in PDF content stream.
    
    Args:
        content: Raw PDF content stream bytes
        
    Returns:
        Anonymized content stream bytes
    """
    # Decode the content stream
    try:
        text = content.decode('latin-1')
    except UnicodeDecodeError:
        # If decoding fails, try with error handling
        text = content.decode('latin-1', errors='ignore')
    
    # Apply all replacements
    for real_name, fake_name in NAME_REPLACEMENTS.items():
        # Replace in text strings (both regular and hex-encoded)
        text = text.replace(real_name, fake_name)
        
        # Also handle potential encoding variations
        text = text.replace(real_name.replace(" ", ""), fake_name.replace(" ", ""))
    
    # Re-encode
    return text.encode('latin-1', errors='ignore')


def anonymize_pdf(input_path: Path, output_path: Path) -> None:
    """
    Anonymize a PDF file by replacing names in content streams and metadata.
    
    Args:
        input_path: Path to the original PDF
        output_path: Path where anonymized PDF will be saved
    """
    print(f"Reading PDF from: {input_path}")
    
    # Open the PDF
    with pikepdf.open(input_path) as pdf:
        # Anonymize metadata
        if '/Info' in pdf.docinfo:
            for key in ['/Author', '/Title', '/Subject', '/Creator']:
                if key in pdf.docinfo:
                    if 'Slater' in str(pdf.docinfo[key]) or 'Amelia' in str(pdf.docinfo[key]):
                        pdf.docinfo[key] = "Anonymous Student"
        
        # Process each page
        for page_num, page in enumerate(pdf.pages, 1):
            print(f"Processing page {page_num}...")
            
            # Get the content stream
            if '/Contents' in page:
                contents = page['/Contents']
                
                # Handle both single content streams and arrays of streams
                if isinstance(contents, pikepdf.Array):
                    for i, stream in enumerate(contents):
                        if isinstance(stream, pikepdf.Stream):
                            original_data = stream.read_bytes()
                            anonymized_data = anonymize_text_content(original_data)
                            stream.write(anonymized_data)
                elif isinstance(contents, pikepdf.Stream):
                    original_data = contents.read_bytes()
                    anonymized_data = anonymize_text_content(original_data)
                    contents.write(anonymized_data)
        
        # Save the anonymized PDF
        print(f"Saving anonymized PDF to: {output_path}")
        pdf.save(output_path)
    
    print("✓ Anonymization complete!")


def main():
    """Main entry point for the script."""
    if len(sys.argv) != 3:
        print("Usage: python3 anonymize_pdf.py <input_pdf> <output_pdf>")
        print()
        print("Example:")
        print("  python3 scripts/anonymize_pdf.py \\")
        print("    input/original.pdf \\")
        print("    input/Sample_Student_Timetable.pdf")
        sys.exit(1)
    
    input_path = Path(sys.argv[1])
    output_path = Path(sys.argv[2])
    
    if not input_path.exists():
        print(f"Error: Input file not found: {input_path}")
        sys.exit(1)
    
    # Create output directory if it doesn't exist
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    try:
        anonymize_pdf(input_path, output_path)
    except Exception as e:
        print(f"Error during anonymization: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
