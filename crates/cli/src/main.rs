use clap::{Parser, Subcommand};

/// wkhtmltopdf – convert HTML to PDF or image.
#[derive(Parser)]
#[command(name = "wkhtmltopdf", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Convert HTML to PDF.
    Pdf {
        /// Input HTML file or URL.
        input: String,
        /// Output PDF path.
        output: String,
    },
    /// Convert HTML to an image.
    Image {
        /// Input HTML file or URL.
        input: String,
        /// Output image path.
        output: String,
        /// Image format (png, jpg, …). Defaults to png.
        #[arg(long, default_value = "png")]
        format: String,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Pdf { input, output } => {
            eprintln!("PDF conversion: {} -> {}", input, output);
            eprintln!("PDF rendering not yet implemented.");
            std::process::exit(1);
        }
        Command::Image {
            input,
            output,
            format,
        } => {
            eprintln!("Image conversion: {} -> {} ({})", input, output, format);
            eprintln!("Image rendering not yet implemented.");
            std::process::exit(1);
        }
    }
}
