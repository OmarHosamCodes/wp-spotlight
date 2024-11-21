use clap::{Parser, ValueEnum};
use csv::WriterBuilder;
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, ValueEnum)]
enum Category {
    Action,
    Filter,
    Shortcode,
    Hook,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::Action => write!(f, "action"),
            Category::Filter => write!(f, "filter"),
            Category::Shortcode => write!(f, "shortcode"),
            Category::Hook => write!(f, "hook"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = "WordPress Hook Analyzer", long_about = None)]
struct Args {
    /// Directory to scan (defaults to current directory)
    #[arg(default_value = ".")]
    directory: String,

    /// Output format (md or csv)
    #[arg(long, default_value = "md")]
    format: OutputFormat,

    /// Output file path
    #[arg(short, long)]
    output: Option<String>,

    /// Filter results by category
    #[arg(long)]
    category: Option<Category>,
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Md,
    Csv,
}

#[derive(Debug, Serialize)]
struct MatchResult {
    category: String,
    function: String,
    function_type: String,
    hook_name: String,
    file_path: String,
    line_number: usize,
    original_line: String,
    highlighted_line: String,
}

struct WPHooksAnalyzer {
    patterns: HashMap<String, Vec<Regex>>,
    function_types: HashMap<String, String>,
}

impl WPHooksAnalyzer {
    fn new() -> Self {
        let mut patterns: HashMap<String, Vec<Regex>> = HashMap::new();
        let action_patterns = vec![
            r#"add_action\(['"][\w-]+['"]"#,
            r#"do_action\(['"][\w-]+['"]"#,
            r#"has_action\(['"][\w-]+['"]"#,
            r#"remove_action\(['"][\w-]+['"]"#,
            r#"remove_all_actions\(['"][\w-]+['"]"#,
            r#"did_action\(['"][\w-]+['"]"#,
            r#"do_action_ref_array\(['"][\w-]+['"]"#,
        ];
        let filter_patterns = vec![
            r#"add_filter\(['"][\w-]+['"]"#,
            r#"apply_filters\(['"][\w-]+['"]"#,
            r#"has_filter\(['"][\w-]+['"]"#,
            r#"remove_filter\(['"][\w-]+['"]"#,
            r#"remove_all_filters\(['"][\w-]+['"]"#,
            r#"current_filter\("#,
            r#"apply_filters_ref_array\(['"][\w-]+['"]"#,
        ];
        let shortcode_patterns = vec![
            r#"add_shortcode\(['"][\w-]+['"]"#,
            r#"do_shortcode\(['"][\w-]+['"]"#,
            r#"has_shortcode\(['"][\w-]+['"]"#,
            r#"remove_shortcode\(['"][\w-]+['"]"#,
            r#"remove_all_shortcodes\("#,
            r#"shortcode_atts\("#,
        ];
        let hook_patterns = vec![r#"wp_hook\(['"][\w-]+['"]"#];

        patterns.insert(
            "action".to_string(),
            action_patterns
                .iter()
                .map(|p| Regex::new(p).unwrap())
                .collect(),
        );
        patterns.insert(
            "filter".to_string(),
            filter_patterns
                .iter()
                .map(|p| Regex::new(p).unwrap())
                .collect(),
        );
        patterns.insert(
            "shortcode".to_string(),
            shortcode_patterns
                .iter()
                .map(|p| Regex::new(p).unwrap())
                .collect(),
        );
        patterns.insert(
            "hook".to_string(),
            hook_patterns
                .iter()
                .map(|p| Regex::new(p).unwrap())
                .collect(),
        );

        let mut function_types = HashMap::new();
        function_types.insert("add_action".to_string(), "Action Registration".to_string());
        function_types.insert("do_action".to_string(), "Action Execution".to_string());
        function_types.insert("has_action".to_string(), "Action Check".to_string());
        function_types.insert("remove_action".to_string(), "Action Removal".to_string());
        function_types.insert(
            "remove_all_actions".to_string(),
            "All Actions Removal".to_string(),
        );
        function_types.insert(
            "did_action".to_string(),
            "Action Execution Check".to_string(),
        );
        function_types.insert(
            "do_action_ref_array".to_string(),
            "Action Execution (Reference)".to_string(),
        );
        function_types.insert("add_filter".to_string(), "Filter Registration".to_string());
        function_types.insert(
            "apply_filters".to_string(),
            "Filter Application".to_string(),
        );
        function_types.insert("has_filter".to_string(), "Filter Check".to_string());
        function_types.insert("remove_filter".to_string(), "Filter Removal".to_string());
        function_types.insert(
            "remove_all_filters".to_string(),
            "All Filters Removal".to_string(),
        );
        function_types.insert(
            "current_filter".to_string(),
            "Current Filter Check".to_string(),
        );
        function_types.insert(
            "apply_filters_ref_array".to_string(),
            "Filter Application (Reference)".to_string(),
        );
        function_types.insert(
            "add_shortcode".to_string(),
            "Shortcode Registration".to_string(),
        );
        function_types.insert(
            "do_shortcode".to_string(),
            "Shortcode Execution".to_string(),
        );
        function_types.insert("has_shortcode".to_string(), "Shortcode Check".to_string());
        function_types.insert(
            "remove_shortcode".to_string(),
            "Shortcode Removal".to_string(),
        );
        function_types.insert(
            "remove_all_shortcodes".to_string(),
            "All Shortcodes Removal".to_string(),
        );
        function_types.insert(
            "shortcode_atts".to_string(),
            "Shortcode Attributes".to_string(),
        );
        function_types.insert("wp_hook".to_string(), "Hook Creation".to_string());

        WPHooksAnalyzer {
            patterns,
            function_types,
        }
    }

    fn get_function_name(&self, match_text: &str) -> String {
        match_text.split('(').next().unwrap_or("").to_string()
    }

    fn find_matches(&self, file_path: &Path) -> io::Result<Vec<MatchResult>> {
        let content = fs::read_to_string(file_path)?;
        let mut results = Vec::new();
        let hook_name_regex = Regex::new(r#"['"][\w-]+['"]"#).unwrap();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            for (category, patterns) in &self.patterns {
                for pattern in patterns {
                    for cap in pattern.find_iter(line) {
                        let func_name = self.get_function_name(&cap.as_str());
                        let hook_name = hook_name_regex
                            .find(cap.as_str())
                            .map(|m| {
                                m.as_str()
                                    .trim_matches(|c| c == '\'' || c == '"')
                                    .to_string()
                            })
                            .unwrap_or_else(|| "N/A".to_string());

                        let highlighted_line =
                            line.replace(cap.as_str(), &format!("`{}`", cap.as_str()));

                        results.push(MatchResult {
                            category: category.clone(),
                            function: func_name.clone(),
                            function_type: self
                                .function_types
                                .get(&func_name)
                                .unwrap_or(&"Unknown".to_string())
                                .clone(),
                            hook_name,
                            line_number: line_num + 1,
                            file_path: file_path.to_string_lossy().to_string(),
                            original_line: line.to_string(),
                            highlighted_line,
                        });
                    }
                }
            }
        }
        Ok(results)
    }

    fn scan_directory(&self, directory: &str) -> io::Result<Vec<MatchResult>> {
        let mut results = Vec::new();
        for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().map_or(false, |ext| ext == "php") {
                if let Ok(mut file_results) = self.find_matches(entry.path()) {
                    results.append(&mut file_results);
                }
            }
        }
        Ok(results)
    }

    fn export_markdown(
        &self,
        results: &[MatchResult],
        output_file: &str,
        project_name: &str,
    ) -> io::Result<()> {
        let mut file = File::create(output_file)?;
        writeln!(file, "# {}\n", project_name)?;
        writeln!(file, "## WordPress Hooks Analysis\n")?;

        for category in &["action", "filter", "shortcode", "hook"] {
            let category_results: Vec<_> =
                results.iter().filter(|r| r.category == *category).collect();

            if !category_results.is_empty() {
                writeln!(
                    file,
                    "\n### {}s\n",
                    category[0..1].to_uppercase() + &category[1..]
                )?;

                let mut by_function: HashMap<String, Vec<&MatchResult>> = HashMap::new();
                for result in category_results {
                    by_function
                        .entry(result.function.clone())
                        .or_default()
                        .push(result);
                }

                for (func_name, func_results) in by_function.iter() {
                    writeln!(
                        file,
                        "#### {}\n",
                        self.function_types
                            .get(func_name)
                            .unwrap_or(&func_name.to_string())
                    )?;

                    for result in func_results {
                        writeln!(
                            file,
                            "- **File:** {}:{}",
                            result.file_path, result.line_number
                        )?;
                        if result.hook_name != "N/A" {
                            writeln!(file, "  - **Hook:** {}", result.hook_name)?;
                        }
                        writeln!(file, "  - **Line:** {}\n", result.highlighted_line)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn export_csv(
        &self,
        results: &[MatchResult],
        output_file: &str,
        project_name: &str,
    ) -> io::Result<()> {
        let mut writer = WriterBuilder::new().from_path(output_file)?;

        // Write headers only once
        writer.write_record(&[
            "project_name", // Added project name column
            "category",
            "function",
            "function_type",
            "hook_name",
            "file_path",
            "line_number",
            "original_line",
            "highlighted_line",
        ])?;

        // Write data rows with project name
        for result in results {
            writer.write_record(&[
                project_name,
                &result.category,
                &result.function,
                &result.function_type,
                &result.hook_name,
                &result.file_path,
                &result.line_number.to_string(),
                &result.original_line,
                &result.highlighted_line,
            ])?;
        }

        writer.flush()?;
        Ok(())
    }
}
fn main() -> io::Result<()> {
    let args = Args::parse();

    // Get absolute path of the directory
    let directory = if args.directory == "." {
        env::current_dir()?
    } else {
        Path::new(&args.directory).canonicalize()?
    };

    // Get directory name for both output file and project name
    let dir_name = directory
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("wp-spotlight")
        .to_string();

    // Generate default output filename based on directory name
    let output_file = args.output.unwrap_or_else(|| {
        format!(
            "{}-analysis.{}",
            dir_name,
            match args.format {
                OutputFormat::Md => "md",
                OutputFormat::Csv => "csv",
            }
        )
    });

    let project_name = dir_name;

    let analyzer = WPHooksAnalyzer::new();
    let mut results = analyzer.scan_directory(directory.to_str().unwrap_or("."))?;

    if let Some(category) = args.category {
        results.retain(|r| r.category == category.to_string());
    }

    match args.format {
        OutputFormat::Md => analyzer.export_markdown(&results, &output_file, &project_name)?,
        OutputFormat::Csv => analyzer.export_csv(&results, &output_file, &project_name)?,
    }

    // Generate summary with color output
    let mut categories = HashMap::new();
    let mut functions = HashMap::new();
    for result in &results {
        *categories.entry(result.category.clone()).or_insert(0) += 1;
        *functions.entry(result.function.clone()).or_insert(0) += 1;
    }

    println!("\n🔍 Analysis complete for {}!", project_name);
    println!("📁 Results saved to: {}", output_file);
    println!("\n📊 Found {} total occurrences:", results.len());

    for (category, count) in categories {
        println!(
            "\n{}s ({} total):",
            category[0..1].to_uppercase() + &category[1..],
            count
        );
        let category_patterns = analyzer.patterns.get(&category).unwrap();
        for (func, count) in &functions {
            if category_patterns
                .iter()
                .any(|p| p.to_string().contains(func))
            {
                println!("  - {}: {}", func, count);
            }
        }
    }

    Ok(())
}
