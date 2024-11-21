#!/usr/bin/env python3
import os
import re
import argparse
import csv
from pathlib import Path

class WPHooksAnalyzer:
    def __init__(self):
        # Patterns to match WordPress hooks and shortcodes
        self.patterns = {
            'filter': r'apply_filters\([\'"][\w-]+[\'"]',
            'action': r'do_action\([\'"][\w-]+[\'"]',
            'shortcode': r'add_shortcode\([\'"][\w-]+[\'"]'
        }

    def find_matches(self, file_path):
        """Find all matches in a given file."""
        results = []
        try:
            with open(file_path, 'r', encoding='utf-8') as file:
                for line_num, line in enumerate(file.readlines(), 1):
                    line = line.strip()
                    for hook_type, pattern in self.patterns.items():
                        matches = re.finditer(pattern, line)
                        for match in matches:
                            # Extract the actual hook name
                            hook_name = re.search(r'[\'"][\w-]+[\'"]', match.group()).group()
                            # Create highlighted version
                            highlighted_line = line.replace(match.group(), f'`{match.group()}`')
                            results.append({
                                'type': hook_type,
                                'line_number': line_num,
                                'file_path': str(file_path),
                                'original_line': line,
                                'highlighted_line': highlighted_line,
                                'hook_name': hook_name.strip('\'"')
                            })
        except Exception as e:
            print(f"Error processing {file_path}: {str(e)}")
        return results

    def scan_directory(self, directory):
        """Recursively scan directory for PHP files."""
        results = []
        directory = Path(directory)
        for file_path in directory.rglob('*.php'):
            results.extend(self.find_matches(file_path))
        return results

    def export_markdown(self, results, output_file):
        """Export results to markdown format."""
        with open(output_file, 'w', encoding='utf-8') as f:
            f.write("# WordPress Hooks Analysis\n\n")

            # Group by type
            for hook_type in ['filter', 'action', 'shortcode']:
                type_results = [r for r in results if r['type'] == hook_type]
                if type_results:
                    f.write(f"\n## {hook_type.title()}s\n\n")
                    for result in type_results:
                        f.write(f"- **File:** {result['file_path']}:{result['line_number']}\n")
                        f.write(f"  - **Hook:** {result['hook_name']}\n")
                        f.write(f"  - **Line:** {result['highlighted_line']}\n\n")

    def export_csv(self, results, output_file):
        """Export results to CSV format."""
        with open(output_file, 'w', newline='', encoding='utf-8') as f:
            writer = csv.DictWriter(f, fieldnames=[
                'type', 'hook_name', 'file_path', 'line_number',
                'original_line', 'highlighted_line'
            ])
            writer.writeheader()
            writer.writerows(results)

def main():
    parser = argparse.ArgumentParser(description='Analyze WordPress plugin hooks and shortcodes')
    parser.add_argument('directory', help='Directory to scan')
    parser.add_argument('--format', choices=['md', 'csv'], default='md',
                      help='Output format (md or csv)')
    parser.add_argument('--output', '-o', help='Output file path')

    args = parser.parse_args()

    # Set default output filename if not provided
    if not args.output:
        args.output = f'wp_hooks_analysis.{args.format}'

    analyzer = WPHooksAnalyzer()
    results = analyzer.scan_directory(args.directory)

    if args.format == 'md':
        analyzer.export_markdown(results, args.output)
    else:
        analyzer.export_csv(results, args.output)

    print(f"Analysis complete! Results saved to {args.output}")
    print(f"Found {len(results)} hooks and shortcodes:")
    print(f"- Filters: {len([r for r in results if r['type'] == 'filter'])}")
    print(f"- Actions: {len([r for r in results if r['type'] == 'action'])}")
    print(f"- Shortcodes: {len([r for r in results if r['type'] == 'shortcode'])}")

if __name__ == "__main__":
    main()
