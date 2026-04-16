mod html;
mod css;
mod style;
mod layout;
mod paint;

use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(name = "crowser", version, about = "A browser engine from scratch")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Phase 1: Tokenize an HTML file
    Tokenize {
        file: String,
    },
    /// Phase 2: Parse an HTML file into a DOM tree
    Parse {
        file: String,
    },
    /// Phase 3: Tokenize a CSS file
    #[command(name = "tokenize-css")]
    TokenizeCss {
        file: String,
    },
    /// Phase 4: Parse a CSS file into a stylesheet
    #[command(name = "parse-css")]
    ParseCss {
        file: String,
    },
    /// Phase 5: Style resolution
    Style {
        file: String,
    },
    /// Phase 6: Layout
    Layout {
        file: String,
    },
    /// Phase 7: Paint to PNG
    Paint {
        file: String,
        #[arg(short)]
        o: String,
    },
    /// Phase 8: Open in window
    Open {
        file: String,
    },
}

fn read_file(path: &str) -> String {
    if !std::path::Path::new(path).exists() {
        eprintln!("Error: file not found: {}", path);
        process::exit(2);
    }
    match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error: could not read file: {}", e);
            process::exit(2);
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Tokenize { file } => {
            let input = read_file(&file);
            let mut tokenizer = html::tokenizer::Tokenizer::new(&input);
            let tokens = tokenizer.tokenize();
            let json = serde_json::to_string_pretty(&tokens).unwrap();
            println!("{}", json);
        }
        Commands::Parse { file } => {
            let input = read_file(&file);
            let mut tokenizer = html::tokenizer::Tokenizer::new(&input);
            let tokens = tokenizer.tokenize();
            let mut parser = html::parser::Parser::new(tokens);
            let dom = parser.parse();
            let json = serde_json::to_string_pretty(&dom).unwrap();
            println!("{}", json);
        }
        Commands::TokenizeCss { file } => {
            let input = read_file(&file);
            let mut tokenizer = css::tokenizer::CssTokenizer::new(&input);
            let tokens = tokenizer.tokenize();
            let json = serde_json::to_string_pretty(&tokens).unwrap();
            println!("{}", json);
        }
        Commands::ParseCss { file } => {
            let input = read_file(&file);
            let mut tokenizer = css::tokenizer::CssTokenizer::new(&input);
            let tokens = tokenizer.tokenize();
            let mut parser = css::parser::CssParser::new(tokens);
            let stylesheet = parser.parse_stylesheet();
            let json = serde_json::to_string_pretty(&stylesheet).unwrap();
            println!("{}", json);
        }
        Commands::Style { file } => {
            let input = read_file(&file);
            // Parse HTML
            let mut html_tokenizer = html::tokenizer::Tokenizer::new(&input);
            let html_tokens = html_tokenizer.tokenize();
            let mut html_parser = html::parser::Parser::new(html_tokens);
            let dom = html_parser.parse();
            // Extract CSS from <style> elements
            let css_text = style::extract_css(&dom);
            // Parse CSS
            let mut css_tokenizer = css::tokenizer::CssTokenizer::new(&css_text);
            let css_tokens = css_tokenizer.tokenize();
            let mut css_parser = css::parser::CssParser::new(css_tokens);
            let stylesheet = css_parser.parse_stylesheet();
            // Resolve styles
            let styled = style::resolve_styles(&dom, &stylesheet, None);
            if let Some(styled_tree) = styled {
                let json = serde_json::to_string_pretty(&styled_tree).unwrap();
                println!("{}", json);
            }
        }
        Commands::Layout { file } => {
            let input = read_file(&file);
            // Parse HTML
            let mut html_tokenizer = html::tokenizer::Tokenizer::new(&input);
            let html_tokens = html_tokenizer.tokenize();
            let mut html_parser = html::parser::Parser::new(html_tokens);
            let dom = html_parser.parse();
            // Extract CSS
            let css_text = style::extract_css(&dom);
            // Parse CSS
            let mut css_tokenizer = css::tokenizer::CssTokenizer::new(&css_text);
            let css_tokens = css_tokenizer.tokenize();
            let mut css_parser = css::parser::CssParser::new(css_tokens);
            let stylesheet = css_parser.parse_stylesheet();
            // Resolve styles
            let styled = style::resolve_styles(&dom, &stylesheet, None);
            if let Some(styled_tree) = styled {
                // Compute layout
                if let Some(layout_tree) = layout::layout(&styled_tree, 800.0, 600.0) {
                    let json = serde_json::to_string_pretty(&layout_tree).unwrap();
                    println!("{}", json);
                }
            }
        }
        Commands::Paint { file, o } => {
            let input = read_file(&file);
            // Parse HTML
            let mut html_tokenizer = html::tokenizer::Tokenizer::new(&input);
            let html_tokens = html_tokenizer.tokenize();
            let mut html_parser = html::parser::Parser::new(html_tokens);
            let dom = html_parser.parse();
            // Extract CSS
            let css_text = style::extract_css(&dom);
            // Parse CSS
            let mut css_tokenizer = css::tokenizer::CssTokenizer::new(&css_text);
            let css_tokens = css_tokenizer.tokenize();
            let mut css_parser = css::parser::CssParser::new(css_tokens);
            let stylesheet = css_parser.parse_stylesheet();
            // Resolve styles
            let styled = style::resolve_styles(&dom, &stylesheet, None);
            if let Some(styled_tree) = styled {
                // Compute layout
                if let Some(layout_tree) = layout::layout(&styled_tree, 800.0, 600.0) {
                    // Find the body styled node for painting
                    let body_styled = layout::find_body(&styled_tree).unwrap_or(&styled_tree);
                    // Paint
                    let mut canvas = paint::Canvas::new(800, 600);
                    paint::paint(&layout_tree, body_styled, &mut canvas);
                    if let Err(e) = canvas.save_png(&o) {
                        eprintln!("Error writing PNG: {}", e);
                        process::exit(1);
                    }
                }
            }
        }
        Commands::Open { file } => {
            let _input = read_file(&file);
            eprintln!("not implemented");
            process::exit(1);
        }
    }
}
