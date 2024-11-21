#!/usr/bin/env python3
import os
import re
import argparse
import csv
from pathlib import Path
from collections import defaultdict

class WPHooksAnalyzer:
    def __init__(self):
        self.patterns = {
            'action': [
                r'add_action\([\'"][\w-]+[\'"]',
                r'do_action\([\'"][\w-]+[\'"]',
                r'has_action\([\'"][\w-]+[\'"]',
                r'remove_action\([\'"][\w-]+[\'"]',
                r'remove_all_actions\([\'"][\w-]+[\'"]',
                r'did_action\([\'"][\w-]+[\'"]',
                r'do_action_ref_array\([\'"][\w-]+[\'"]'
            ],
            'filter': [
                r'add_filter\([\'"][\w-]+[\'"]',
                r'apply_filters\([\'"][\w-]+[\'"]',
                r'has_filter\([\'"][\w-]+[\'"]',
                r'remove_filter\([\'"][\w-]+[\'"]',
                r'remove_all_filters\([\'"][\w-]+[\'"]',
                r'current_filter\(',
                r'apply_filters_ref_array\([\'"][\w-]+[\'"]'
            ],
            'shortcode': [
                r'add_shortcode\([\'"][\w-]+[\'"]',
                r'do_shortcode\([\'"][\w-]+[\'"]',
                r'has_shortcode\([\'"][\w-]+[\'"]',
                r'remove_shortcode\([\'"][\w-]+[\'"]',
                r'remove_all_shortcodes\(',
                r'shortcode_atts\('
            ],
            'hook': [
                r'wp_hook\([\'"][\w-]+[\'"]'
            ]
        }

        self.function_types = {
            'add_action': 'Action Registration',
            'do_action': 'Action Execution',
            'has_action': 'Action Check',
            'remove_action': 'Action Removal',
            'remove_all_actions': 'All Actions Removal',
            'did_action': 'Action Execution Check',
            'do_action_ref_array': 'Action Execution (Reference)',
            'add_filter': 'Filter Registration',
            'apply_filters': 'Filter Application',
            'has_filter': 'Filter Check',
            'remove_filter': 'Filter Removal',
            'remove_all_filters': 'All Filters Removal',
            'current_filter': 'Current Filter Check',
            'apply_filters_ref_array': 'Filter Application (Reference)',
            'add_shortcode': 'Shortcode Registration',
            'do_shortcode': 'Shortcode Execution',
            'has_shortcode': 'Shortcode Check',
            'remove_shortcode': 'Shortcode Removal',
            'remove_all_shortcodes': 'All Shortcodes Removal',
            'shortcode_atts': 'Shortcode Attributes',
            'wp_hook': 'Hook Creation'
        }

    def get_function_name(self, match_text):
        """Extract the function name from the matched text."""
        return match_text.split('(')[0]

    def find_matches(self, file_path):
        """Find all matches in a given file."""
        results = []
        try:
            with open(file_path, 'r', encoding='utf-8') as file:
                for line_num, line in enumerate(file.readlines(), 1):
                    line = line.strip()
                    for category, patterns in self.patterns.items():
                        for pattern in patterns:
                            matches = re.finditer(pattern, line)
                            for match in matches:
                                func_name = self.get_function_name(match.group())
                                hook_name = "N/A"
                                hook_match = re.search(r'[\'"][\w-]+[\'"]', match.group())
                                if hook_match:
                                    hook_name = hook_match.group().strip('\'"')

                                highlighted_line = line.replace(match.group(), f'`{match.group()}`')

                                results.append({
                                    'category': category,
                                    'function': func_name,
                                    'function_type': self.function_types.get(func_name, 'Unknown'),
                                    'hook_name': hook_name,
                                    'line_number': line_num,
                                    'file_path': str(file_path),
                                    'original_line': line,
                                    'highlighted_line': highlighted_line
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

    def export_markdown(self, results, output_file, project_name):
        """Export results to markdown format."""
        with open(output_file, 'w', encoding='utf-8') as f:
            # Add project name as first line
            f.write(f"# {project_name}\n\n")
            f.write("## WordPress Hooks Analysis\n\n")

            # Group by category
            for category in ['action', 'filter', 'shortcode', 'hook']:
                category_results = [r for r in results if r['category'] == category]
                if category_results:
                    f.write(f"\n### {category.title()}s\n\n")

                    # Group by function type
                    by_function = defaultdict(list)
                    for result in category_results:
                        by_function[result['function']].append(result)

                    for func_name, func_results in sorted(by_function.items()):
                        f.write(f"#### {self.function_types.get(func_name, func_name)}\n\n")
                        for result in func_results:
                            f.write(f"- **File:** {result['file_path']}:{result['line_number']}\n")
                            if result['hook_name'] != "N/A":
                                f.write(f"  - **Hook:** {result['hook_name']}\n")
                            f.write(f"  - **Line:** {result['highlighted_line']}\n\n")

    def export_csv(self, results, output_file, project_name):
        """Export results to CSV format."""
        with open(output_file, 'w', newline='', encoding='utf-8') as f:
            # Add project name as first line
            f.write(f"Project: {project_name}\n")

            writer = csv.DictWriter(f, fieldnames=[
                'category', 'function', 'function_type', 'hook_name',
                'file_path', 'line_number', 'original_line', 'highlighted_line'
            ])
            writer.writeheader()
            writer.writerows(results)

def main():
    parser = argparse.ArgumentParser(description='Analyze WordPress plugin hooks, filters, and shortcodes')
    parser.add_argument('directory', help='Directory to scan')
    parser.add_argument('--format', choices=['md', 'csv'], default='md',
                      help='Output format (md or csv)')
    parser.add_argument('--output', '-o', help='Output file path')
    parser.add_argument('--category', choices=['action', 'filter', 'shortcode', 'hook'],
                      help='Filter results by category')

    args = parser.parse_args()

    if not args.output:
        args.output = f'wp_hooks_analysis.{args.format}'

    # project_name = os.path.dirname(os.path.abspath(__file__))
    project_name = os.path.basename(os.path.dirname(os.path.abspath(__file__)))

    analyzer = WPHooksAnalyzer()
    results = analyzer.scan_directory(args.directory)

    if args.category:
        results = [r for r in results if r['category'] == args.category]

    if args.format == 'md':
        analyzer.export_markdown(results, args.output, project_name)
    else:
        analyzer.export_csv(results, args.output, project_name)

    # Generate summary
    categories = defaultdict(int)
    functions = defaultdict(int)
    for result in results:
        categories[result['category']] += 1
        functions[result['function']] += 1

    print(f"\nAnalysis complete for {project_name}!")
    print(f"Results saved to {args.output}")
    print(f"\nFound {len(results)} total occurrences:")
    for category, count in categories.items():
        print(f"\n{category.title()}s ({count} total):")
        category_functions = {func: count for func, count in functions.items()
                            if any(func in pattern for pattern in analyzer.patterns[category])}
        for func, count in sorted(category_functions.items()):
            print(f"  - {func}: {count}")

if __name__ == "__main__":
    main()
