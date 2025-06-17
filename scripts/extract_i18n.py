#!/usr/bin/env python3
"""
Extract i18n macros from Rust code and generate main.ftl file.

This script scans all Rust files in the project for tr! and tr_with_context! macro calls
and generates a main.ftl file with all the extracted strings.
"""

import os
import re
import argparse
from pathlib import Path
from typing import Set, Dict, List, Tuple
import json
import collections

def find_rust_files(project_root: Path) -> List[Path]:
    """Find all Rust files in the project."""
    rust_files = []
    for root, dirs, files in os.walk(project_root):
        # Skip target and .git directories
        dirs[:] = [d for d in dirs if d not in ['target', '.git', 'node_modules']]
        
        for file in files:
            if file.endswith('.rs'):
                rust_files.append(Path(root) / file)
    
    return rust_files

def extract_tr_macros_with_lines(content: str, file_path: str) -> Dict[str, list]:
    """Extract tr! macro calls from Rust code with optional comments and line numbers."""
    matches = []
    
    # Find all tr! macro calls using regex to get the full macro content
    tr_pattern = r'tr!\s*\(([^)]*)\)'
    
    lines = content.split('\n')
    for line_num, line in enumerate(lines, 1):
        for match in re.findall(tr_pattern, line):
            # Parse the arguments from the macro content
            args = parse_macro_arguments(match)
            if len(args) >= 1:
                key = args[0].strip()
                comment = args[1].strip() if len(args) > 1 else ""
                
                if not any(skip in key.lower() for skip in [
                    '/', '\\', '.ftl', '.rs', 'http', 'https', 'www', '@',
                    'crates/', 'src/', 'target/', 'build.rs']):
                    matches.append((key, comment, line_num, file_path))
    
    return matches

def extract_tr_with_context_macros_with_lines(content: str, file_path: str) -> Dict[Tuple[str, str], list]:
    """Extract tr_with_context! macro calls from Rust code with line numbers and comments."""
    # Pattern for tr_with_context!("string", "context") or tr_with_context!("string", "context", "comment")
    # Updated to handle new syntax: tr_with_context!("string", "param" => value, "param2" => value2)
    tr_context_pattern = r'tr_with_context!\s*\(\s*["\']([^"\']{1,100})["\']\s*,\s*["\']([^"\']{1,50})["\'](?:\s*,\s*["\']([^"\']{1,200})["\'])?\s*\)'
    
    # Also look for the new syntax with named parameters
    # This pattern captures the string and the first parameter name
    tr_context_new_pattern = r'tr_with_context!\s*\(\s*["\']([^"\']{1,100})["\']\s*,\s*["\']([^"\']{1,50})["\']\s*=>\s*[^,)]+'
    
    matches = re.findall(tr_context_pattern, content)
    new_matches = re.findall(tr_context_new_pattern, content)
    
    # Combine both patterns
    all_matches = matches + new_matches
    
    # Filter out obvious false positives and collect with line numbers and comments
    filtered_matches = {}
    for line_num, line in enumerate(content.split('\n'), 1):
        for match in all_matches:
            base = match[0].strip()  # Strip leading/trailing whitespace
            context = match[1].strip()  # Strip leading/trailing whitespace
            comment = match[2] if len(match) > 2 and match[2] else ""
            
            if not any(skip in base.lower() for skip in [
                '/', '\\', '.ftl', '.rs', 'http', 'https', 'www', '@',
                'crates/', 'src/', 'target/', 'build.rs'
            ]):
                if (base, context) not in filtered_matches:
                    filtered_matches[(base, context)] = []
                filtered_matches[(base, context)].append((comment, f"{file_path}:{line_num}"))
    
    return filtered_matches

def extract_tr_plural_macros_with_lines(content: str, file_path: str) -> Dict[str, list]:
    tr_plural_pattern = r'tr_plural!\s*\(\s*["\']([^"\']{1,100})["\']\s*,(?:\s*["\']([^"\']{1,200})["\']\s*,)?'
    matches = []
    for i, line in enumerate(content.splitlines(), 1):
        for m in re.finditer(tr_plural_pattern, line):
            key = m.group(1).strip()  # Strip leading/trailing whitespace
            comment = m.group(2) if m.lastindex and m.lastindex >= 2 and m.group(2) else ""
            if not any(skip in key.lower() for skip in [
                '/', '\\', '.ftl', '.rs', 'http', 'https', 'www', '@',
                'crates/', 'src/', 'target/', 'build.rs']):
                matches.append((key, comment, i, file_path))
    return matches

def parse_macro_arguments(content: str) -> List[str]:
    """Parse macro arguments, handling quoted strings with apostrophes and escaped quotes."""
    args = []
    i = 0
    content = content.strip()
    
    while i < len(content):
        # Skip whitespace
        while i < len(content) and content[i].isspace():
            i += 1
        
        if i >= len(content):
            break
            
        # Check if we're at a quoted string
        if content[i] in ['"', "'"]:
            quote_char = content[i]
            i += 1  # Skip the opening quote
            current_arg = ""
            
            # Read until the closing quote
            while i < len(content):
                char = content[i]
                if char == quote_char:
                    # Check if it's escaped
                    if i > 0 and content[i-1] == '\\':
                        current_arg = current_arg[:-1] + char  # Remove backslash, add quote
                    else:
                        i += 1  # Skip the closing quote
                        break
                else:
                    current_arg += char
                i += 1
            
            args.append(current_arg)
        else:
            # Non-quoted argument - read until comma or end
            current_arg = ""
            while i < len(content):
                char = content[i]
                if char == ',':
                    i += 1  # Skip the comma
                    break
                elif char.isspace():
                    i += 1
                else:
                    current_arg += char
                    i += 1
            
            if current_arg.strip():
                args.append(current_arg.strip())
    
    return args

def extract_tr_macros(content: str) -> Dict[str, str]:
    """Extract tr! macro calls from Rust code with optional comments."""
    # Use a more robust approach to handle apostrophes and escaped quotes
    filtered_matches = {}
    
    # Find all tr! macro calls using regex to get the full macro content
    tr_pattern = r'tr!\s*\(([^)]*)\)'
    matches = re.findall(tr_pattern, content)
    
    for match in matches:
        # Parse the arguments from the macro content
        args = parse_macro_arguments(match)
        if len(args) >= 1:
            key = args[0].strip()
            comment = args[1].strip() if len(args) > 1 else ""
            
            # Skip file paths, URLs, and other non-UI strings
            if not any(skip in key.lower() for skip in [
                '/', '\\', '.ftl', '.rs', 'http', 'https', 'www', '@', 
                'crates/', 'src/', 'target/', 'build.rs'
            ]):
                filtered_matches[key] = comment
    
    return filtered_matches

def extract_tr_with_context_macros(content: str) -> Dict[Tuple[str, str], str]:
    """Extract tr_with_context! macro calls from Rust code with optional comments."""
    # Pattern for tr_with_context!("string", "context") or tr_with_context!("string", "context", "comment")
    # Updated to handle new syntax: tr_with_context!("string", "param" => value, "param2" => value2)
    tr_context_pattern = r'tr_with_context!\s*\(\s*["\']([^"\']{1,100})["\']\s*,\s*["\']([^"\']{1,50})["\'](?:\s*,\s*["\']([^"\']{1,200})["\'])?\s*\)'
    matches = re.findall(tr_context_pattern, content)
    
    # Also look for the new syntax with named parameters
    # This pattern captures the string and the first parameter name
    tr_context_new_pattern = r'tr_with_context!\s*\(\s*["\']([^"\']{1,100})["\']\s*,\s*["\']([^"\']{1,50})["\']\s*=>\s*[^,)]+'
    new_matches = re.findall(tr_context_new_pattern, content)
    
    # Filter out obvious false positives and collect with comments
    filtered_matches = {}
    for match in matches:
        base = match[0].strip()  # Strip leading/trailing whitespace
        context = match[1].strip()  # Strip leading/trailing whitespace
        comment = match[2] if len(match) > 2 and match[2] else ""
        
        if not any(skip in base.lower() for skip in [
            '/', '\\', '.ftl', '.rs', 'http', 'https', 'www', '@',
            'crates/', 'src/', 'target/', 'build.rs'
        ]):
            filtered_matches[(base, context)] = comment
    
    # Handle new syntax matches
    for match in new_matches:
        base = match[0].strip()  # Strip leading/trailing whitespace
        context = match[1].strip()  # Strip leading/trailing whitespace
        comment = ""  # No comment in new syntax
        
        if not any(skip in base.lower() for skip in [
            '/', '\\', '.ftl', '.rs', 'http', 'https', 'www', '@',
            'crates/', 'src/', 'target/', 'build.rs'
        ]):
            filtered_matches[(base, context)] = comment
    
    return filtered_matches

def extract_tr_plural_macros(content: str) -> Dict[str, str]:
    """Extract tr_plural! macro calls from Rust code with optional comments."""
    # Pattern for tr_plural!("string", count) or tr_plural!("string", count, "comment")
    tr_plural_pattern = r'tr_plural!\s*\(\s*["\']([^"\']{1,100})["\']\s*,(?:\s*["\']([^"\']{1,200})["\']\s*,)?'
    matches = re.findall(tr_plural_pattern, content)
    
    # Filter out obvious false positives and collect with comments
    filtered_matches = {}
    for match in matches:
        key = match[0].strip()  # Strip leading/trailing whitespace
        comment = match[1] if len(match) > 1 and match[1] else ""
        
        if not any(skip in key.lower() for skip in [
            '/', '\\', '.ftl', '.rs', 'http', 'https', 'www', '@',
            'crates/', 'src/', 'target/', 'build.rs'
        ]):
            filtered_matches[key] = comment
    
    return filtered_matches

def normalize_ftl_key(key: str) -> str:
    """Normalize a string to a valid FTL key: only letters, digits, hyphens, and underscores."""
    # Replace all invalid characters with underscores
    normalized = re.sub(r'[^a-zA-Z0-9_-]', '_', key)
    # If the key starts with an underscore, prefix it with "key"
    if normalized.startswith('_'):
        normalized = "key" + normalized
    return normalized

def pseudolocalize(text: str) -> str:
    """Convert English text to pseudolocalized text for testing."""
    # Common pseudolocalization patterns
    replacements = {
        'a': 'à', 'e': 'é', 'i': 'í', 'o': 'ó', 'u': 'ú',
        'A': 'À', 'E': 'É', 'I': 'Í', 'O': 'Ó', 'U': 'Ú',
        'n': 'ñ', 'N': 'Ñ', 'c': 'ç', 'C': 'Ç'
    }
    
    # Apply character replacements
    result = text
    for char, replacement in replacements.items():
        result = result.replace(char, replacement)
    
    # Add brackets around the text to make it more obvious
    result = f"[{result}]"
    
    # Expand short strings by repeating them
    if len(result) < 10:
        result = result * 2
    
    return result

def generate_ftl_content(tr_strings: Dict[str, str], 
                        context_strings: Dict[Tuple[str, str], str], 
                        plural_strings: Dict[str, str],
                        tr_occurrences: Dict[Tuple[str, str], list],
                        context_occurrences: Dict[Tuple[str, str, str], list],
                        plural_occurrences: Dict[Tuple[str, str], list],
                        pseudolocalize_content: bool = False) -> str:
    """Generate FTL file content from extracted strings with comments."""
    
    lines = [
        "# Main translation file for Notedeck",
        "# This file contains common UI strings used throughout the application",
        "# Auto-generated by extract_i18n.py - DO NOT EDIT MANUALLY",
        "",
    ]
    
    # Sort strings for consistent output
    sorted_tr = sorted(tr_strings.items())
    sorted_context = sorted(context_strings.items())
    sorted_plural = sorted(plural_strings.items())
    
    # Add regular tr! strings
    if sorted_tr:
        lines.append("# Regular strings")
        for string, comment in sorted_tr:
            # Skip if it's a context string (will be handled separately)
            if not any(string == base and context for (base, context), _ in context_strings.items()):
                # Collect all comments for this key from all files
                all_comments = set()
                for (file_path, key), occurrences in tr_occurrences.items():
                    if key == string:
                        for comment, line in occurrences:
                            if comment:
                                all_comments.add(comment)
                for c in all_comments:
                    lines.append(f"# {c}")
                norm_key = normalize_ftl_key(string)
                # Apply pseudolocalization if requested
                value = pseudolocalize(string) if pseudolocalize_content else string
                lines.append(f"{norm_key} = {value}")
        lines.append("")
    
    # Add context-aware strings
    if sorted_context:
        lines.append("# Context-specific translations")
        current_base = None
        for (base, context), comment in sorted_context:
            if base != current_base:
                if current_base is not None:
                    lines.append("")
                current_base = base
            
                # Collect all comments for this context key from all files
                all_comments = set()
                for (file_path, b, c), occurrences in context_occurrences.items():
                    if b == base and c == context:
                        for cmt, _ in occurrences:
                            if cmt:  # Only add non-empty comments
                                all_comments.add(cmt)
                
                # Write all unique comments
                for c in sorted(all_comments):
                    lines.append(f"# {c}")
                
                # If no comments were found, use the current comment or default
                if not all_comments:
                    if comment:
                        lines.append(f"# {comment}")
                    else:
                        lines.append(f"# {base} used as {context}")
            
            norm_key = normalize_ftl_key(f"{base}#{context}")
            # Apply pseudolocalization if requested
            # For context strings, we need to pseudolocalize the full string with placeholders
            if pseudolocalize_content:
                # Replace placeholders with pseudolocalized versions
                pseudolocalized_base = base
                if "{error:?}" in base:
                    pseudolocalized_base = base.replace("{error:?}", "{érrór:?}")
                if "{list_kind:?}" in base:
                    pseudolocalized_base = base.replace("{list_kind:?}", "{líst_kíñd:?}")
                value = pseudolocalize(pseudolocalized_base)
            else:
                value = base
            lines.append(f"{norm_key} = {value}")
        lines.append("")
    
    # Add pluralized strings
    if sorted_plural:
        lines.append("# Pluralized strings")
        for string, comment in sorted_plural:
            # Generate basic pluralization pattern
            if "$count" in string:
                # Extract the base form (e.g., "minutes ago" from "$count minutes ago")
                base_form = string.replace("$count ", "")
                singular_form = base_form.replace("s ", " ", 1) if base_form.startswith("s ") else base_form
                
                # Collect all comments for this key from all files
                all_comments = set()
                for (file_path, key), occurrences in plural_occurrences.items():
                    if key == string:
                        for c, _ in occurrences:
                            if c:  # Only add non-empty comments
                                all_comments.add(c)
                
                # Write all unique comments
                for c in sorted(all_comments):
                    lines.append(f"# {c}")
                
                # If no comments were found, use the current comment or default
                if not all_comments:
                    if comment:
                        lines.append(f"# {comment}")
                    else:
                        lines.append(f"# {string} with pluralization")
                
                norm_key = normalize_ftl_key(string)
                # Apply pseudolocalization if requested
                if pseudolocalize_content:
                    base_form = pseudolocalize(base_form)
                    singular_form = pseudolocalize(singular_form)
                
                lines.append(f'{norm_key} = {{ $count ->')
                lines.append(f'    [1] 1 {singular_form}')
                lines.append(f'    *[other] {{ $count }} {base_form}')
                lines.append(f'}}')
                lines.append("")
    
    return "\n".join(lines)

def read_existing_ftl(ftl_path: Path) -> Dict[str, str]:
    """Read existing FTL file to preserve comments and custom translations."""
    if not ftl_path.exists():
        return {}
    
    existing_translations = {}
    with open(ftl_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Extract key-value pairs
    pattern = r'^([^#\s][^=]*?)\s*=\s*(.+)$'
    for line in content.split('\n'):
        match = re.match(pattern, line.strip())
        if match:
            key = match.group(1).strip()
            value = match.group(2).strip()
            norm_key = normalize_ftl_key(key)
            existing_translations[norm_key] = value
    
    return existing_translations

def main():
    parser = argparse.ArgumentParser(description='Extract i18n macros and generate FTL file')
    parser.add_argument('--project-root', type=str, default='.', 
                       help='Project root directory (default: current directory)')
    parser.add_argument('--output', type=str, 
                       default='crates/notedeck/src/i18n/locales/en-US/main.ftl',
                       help='Output FTL file path')
    parser.add_argument('--dry-run', action='store_true',
                       help='Show what would be generated without writing to file')
    parser.add_argument('--fail-on-collisions', action='store_true',
                       help='Exit with error if key collisions are detected')
    parser.add_argument('--pseudolocalize', action='store_true',
                       help='Generate pseudolocalized content for testing')
    
    args = parser.parse_args()
    
    project_root = Path(args.project_root)
    output_path = Path(args.output)
    
    print(f"Scanning Rust files in {project_root}...")
    
    # Find all Rust files
    rust_files = find_rust_files(project_root)
    print(f"Found {len(rust_files)} Rust files")
    
    # Extract strings from all files
    all_tr_strings = {}
    all_context_strings = {}
    all_plural_strings = {}
    
    # Track collisions
    tr_collisions = {}
    context_collisions = {}
    plural_collisions = {}
    
    # Track all occurrences for intra-file collision detection
    tr_occurrences = collections.defaultdict(list)
    context_occurrences = collections.defaultdict(list)
    plural_occurrences = collections.defaultdict(list)
    
    for rust_file in rust_files:
        try:
            with open(rust_file, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # For intra-file collision detection
            tr_lines = extract_tr_macros_with_lines(content, str(rust_file))
            for key, comment, line, file_path in tr_lines:
                tr_occurrences[(file_path, key)].append((comment, line))
            context_lines = extract_tr_with_context_macros_with_lines(content, str(rust_file))
            for (base, context), occurrences in context_lines.items():
                for comment, line_info in occurrences:
                    context_occurrences[(file_path, base, context)].append((comment, line_info))
            plural_lines = extract_tr_plural_macros_with_lines(content, str(rust_file))
            for key, comment, line, file_path in plural_lines:
                plural_occurrences[(file_path, key)].append((comment, line))
            
            tr_strings = extract_tr_macros(content)
            context_strings = extract_tr_with_context_macros(content)
            plural_strings = extract_tr_plural_macros(content)
            
            if tr_strings or context_strings or plural_strings:
                print(f"  {rust_file}: {len(tr_strings)} tr!, {len(context_strings)} tr_with_context!, {len(plural_strings)} tr_plural!")
            
            # Check for collisions in tr! strings
            for key, comment in tr_strings.items():
                if key in all_tr_strings and all_tr_strings[key] != comment:
                    if key not in tr_collisions:
                        tr_collisions[key] = []
                    tr_collisions[key].append((rust_file, all_tr_strings[key]))
                    tr_collisions[key].append((rust_file, comment))
                all_tr_strings[key] = comment
            
            # Check for collisions in context strings
            for (base, context), comment in context_strings.items():
                if (base, context) in all_context_strings and all_context_strings[(base, context)] != comment:
                    if (base, context) not in context_collisions:
                        context_collisions[(base, context)] = []
                    context_collisions[(base, context)].append((rust_file, all_context_strings[(base, context)]))
                    context_collisions[(base, context)].append((rust_file, comment))
                all_context_strings[(base, context)] = comment
            
            # Check for collisions in plural strings
            for key, comment in plural_strings.items():
                if key in all_plural_strings and all_plural_strings[key] != comment:
                    if key not in plural_collisions:
                        plural_collisions[key] = []
                    plural_collisions[key].append((rust_file, all_plural_strings[key]))
                    plural_collisions[key].append((rust_file, comment))
                all_plural_strings[key] = comment
            
        except Exception as e:
            print(f"Error reading {rust_file}: {e}")
    
    # Intra-file collision detection
    has_intra_file_collisions = False
    for (file_path, key), occurrences in tr_occurrences.items():
        comments = set(c for c, _ in occurrences)
        if len(occurrences) > 1 and len(comments) > 1:
            has_intra_file_collisions = True
            print(f"\n⚠️  Intra-file key collision in {file_path} for '{key}':")
            for comment, line in occurrences:
                comment_text = f" (comment: '{comment}')" if comment else " (no comment)"
                print(f"    Line {line}{comment_text}")
    for (file_path, base, context), occurrences in context_occurrences.items():
        comments = set(c for c, _ in occurrences)
        if len(occurrences) > 1 and len(comments) > 1:
            has_intra_file_collisions = True
            print(f"\n⚠️  Intra-file key collision in {file_path} for '{base}#{context}':")
            for comment, line in occurrences:
                comment_text = f" (comment: '{comment}')" if comment else " (no comment)"
                print(f"    Line {line}{comment_text}")
    for (file_path, key), occurrences in plural_occurrences.items():
        comments = set(c for c, _ in occurrences)
        if len(occurrences) > 1 and len(comments) > 1:
            has_intra_file_collisions = True
            print(f"\n⚠️  Intra-file key collision in {file_path} for '{key}':")
            for comment, line in occurrences:
                comment_text = f" (comment: '{comment}')" if comment else " (no comment)"
                print(f"    Line {line}{comment_text}")
    if has_intra_file_collisions and args.fail_on_collisions:
        print(f"❌ Exiting due to intra-file key collisions (--fail-on-collisions flag)")
        exit(1)
    
    # Report collisions
    has_collisions = False
    
    if tr_collisions:
        has_collisions = True
        print(f"\n⚠️  Key collisions detected in tr! strings:")
        for key, collisions in tr_collisions.items():
            print(f"  '{key}':")
            for file_path, comment in collisions:
                comment_text = f" (comment: '{comment}')" if comment else " (no comment)"
                print(f"    {file_path}{comment_text}")
    
    if context_collisions:
        has_collisions = True
        print(f"\n⚠️  Key collisions detected in tr_with_context! strings:")
        for (base, context), collisions in context_collisions.items():
            print(f"  '{base}#{context}':")
            for file_path, comment in collisions:
                comment_text = f" (comment: '{comment}')" if comment else " (no comment)"
                print(f"    {file_path}{comment_text}")
    
    if plural_collisions:
        has_collisions = True
        print(f"\n⚠️  Key collisions detected in tr_plural! strings:")
        for key, collisions in plural_collisions.items():
            print(f"  '{key}':")
            for file_path, comment in collisions:
                comment_text = f" (comment: '{comment}')" if comment else " (no comment)"
                print(f"    {file_path}{comment_text}")
    
    if has_collisions:
        print(f"\n💡 Collision resolution: The last occurrence of each key will be used.")
        if args.fail_on_collisions:
            print(f"❌ Exiting due to key collisions (--fail-on-collisions flag)")
            exit(1)
    
    print(f"\nExtracted strings:")
    print(f"  Regular strings: {len(all_tr_strings)}")
    print(f"  Context strings: {len(all_context_strings)}")
    print(f"  Plural strings: {len(all_plural_strings)}")
    
    # Generate FTL content
    ftl_content = generate_ftl_content(all_tr_strings, all_context_strings, all_plural_strings, tr_occurrences, context_occurrences, plural_occurrences, args.pseudolocalize)
    
    if args.dry_run:
        print(f"\n--- Generated FTL content ---")
        print(ftl_content)
        print(f"--- End of content ---")
    else:
        # Ensure output directory exists
        output_path.parent.mkdir(parents=True, exist_ok=True)
        
        # Write to file
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(ftl_content)
        
        print(f"\nGenerated FTL file: {output_path}")
        print(f"Total strings: {len(all_tr_strings) + len(all_context_strings) + len(all_plural_strings)}")

if __name__ == '__main__':
    main() 