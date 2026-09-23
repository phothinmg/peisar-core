use colored::*;

/// Print a formatted error message to stderr.
///
/// When `e` is `true` the process exits with code 1 after printing.
pub fn error(message: &str, e: bool) {
    eprintln!("{}", "");
    eprintln!("  [{}]", "Peisar Error".magenta().bold());
    eprintln!("   {}", message);
    if e {
        std::process::exit(1)
    }
}

/// Print a formatted info message to stderr.
pub fn info(message: &str) {
    eprintln!("  [{}]", "Peisar Info".green().bold());
    eprintln!("   {}", message);
}

/// Print a formatted warning message to stderr.
pub fn warning(message: &str) {
    eprintln!("  [{}]", "Peisar Warning".yellow().bold());
    eprintln!("   {}", message);
}
