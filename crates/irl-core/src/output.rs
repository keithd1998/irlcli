use std::io::{self, IsTerminal};

use colored::Colorize;
use serde::Serialize;
use tabled::settings::Style;
use tabled::{Table, Tabled};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

impl OutputFormat {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "table" => Some(Self::Table),
            "json" => Some(Self::Json),
            "csv" => Some(Self::Csv),
            _ => None,
        }
    }
}

pub struct OutputConfig {
    pub format: OutputFormat,
    pub colour: bool,
    pub quiet: bool,
}

impl OutputConfig {
    pub fn new(format: OutputFormat, colour: bool, quiet: bool) -> Self {
        let is_tty = io::stdout().is_terminal();
        let no_color_env = std::env::var("NO_COLOR").is_ok();

        let effective_colour = colour && is_tty && !no_color_env;

        if !effective_colour {
            colored::control::set_override(false);
        }

        Self {
            format,
            colour: effective_colour,
            quiet,
        }
    }

    pub fn print_header(&self, text: &str) {
        if self.quiet {
            return;
        }
        if self.colour {
            eprintln!("{}", text.bold().cyan());
        } else {
            eprintln!("{}", text);
        }
    }

    pub fn print_info(&self, text: &str) {
        if self.quiet {
            return;
        }
        if self.colour {
            eprintln!("{}", text.dimmed());
        } else {
            eprintln!("{}", text);
        }
    }

    pub fn print_error(&self, text: &str) {
        if self.colour {
            eprintln!("{} {}", "Error:".red().bold(), text);
        } else {
            eprintln!("Error: {}", text);
        }
    }

    pub fn render_table<T: Tabled>(&self, items: &[T]) {
        if items.is_empty() {
            if !self.quiet {
                eprintln!("No results found.");
            }
            return;
        }
        let mut table = Table::new(items);
        table.with(Style::rounded());
        println!("{}", table);
    }

    pub fn render_json<T: Serialize>(&self, data: &T) -> Result<(), io::Error> {
        let json = serde_json::to_string_pretty(data).map_err(|e| io::Error::other(e))?;
        println!("{}", json);
        Ok(())
    }

    pub fn render_csv<T: Serialize>(&self, items: &[T]) -> Result<(), io::Error> {
        let mut writer = csv::Writer::from_writer(io::stdout());
        for item in items {
            writer.serialize(item).map_err(|e| io::Error::other(e))?;
        }
        writer.flush()?;
        Ok(())
    }

    pub fn render<T: Tabled + Serialize>(&self, items: &[T]) -> Result<(), io::Error> {
        match self.format {
            OutputFormat::Table => {
                self.render_table(items);
                Ok(())
            }
            OutputFormat::Json => self.render_json(&items),
            OutputFormat::Csv => self.render_csv(items),
        }
    }

    pub fn render_single<T: Serialize>(&self, item: &T) -> Result<(), io::Error> {
        match self.format {
            OutputFormat::Table | OutputFormat::Csv => {
                // For single items, JSON is a reasonable fallback
                self.render_json(item)
            }
            OutputFormat::Json => self.render_json(item),
        }
    }
}

pub fn coming_soon(module: &str) {
    let is_tty = io::stdout().is_terminal();
    if is_tty && std::env::var("NO_COLOR").is_err() {
        eprintln!(
            "{} The {} module is coming soon!",
            "ℹ".blue().bold(),
            module.yellow().bold()
        );
    } else {
        eprintln!("The {} module is coming soon!", module);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(
            OutputFormat::from_str_opt("table"),
            Some(OutputFormat::Table)
        );
        assert_eq!(OutputFormat::from_str_opt("json"), Some(OutputFormat::Json));
        assert_eq!(OutputFormat::from_str_opt("csv"), Some(OutputFormat::Csv));
        assert_eq!(OutputFormat::from_str_opt("JSON"), Some(OutputFormat::Json));
        assert_eq!(OutputFormat::from_str_opt("invalid"), None);
    }

    #[derive(Tabled, Serialize)]
    struct TestRow {
        name: String,
        value: i32,
    }

    #[test]
    fn test_render_json() {
        let config = OutputConfig {
            format: OutputFormat::Json,
            colour: false,
            quiet: false,
        };
        let items = vec![
            TestRow {
                name: "a".to_string(),
                value: 1,
            },
            TestRow {
                name: "b".to_string(),
                value: 2,
            },
        ];
        // Just verify it doesn't panic
        let result = config.render_json(&items);
        assert!(result.is_ok());
    }

    #[test]
    fn test_coming_soon_no_panic() {
        // Ensure it doesn't panic
        coming_soon("test-module");
    }
}
