use colored::*;
#[cfg(feature = "napi")]
use napi_derive::napi;

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

// ---------------------------------------------------------------------------
// napi-rs wrappers (JavaScript interop)
// ---------------------------------------------------------------------------

/// napi-exported wrapper of [`error`].
///
/// napi-rs does not support `&str` parameters, so the JS-facing version
/// accepts an owned `String`.
#[cfg(feature = "napi")]
#[cfg_attr(feature = "napi", napi)]
pub fn log_error(message: String, e: bool) {
    error(&message, e);
}

/// napi-exported wrapper of [`info`].
#[cfg(feature = "napi")]
#[cfg_attr(feature = "napi", napi)]
pub fn log_info(message: String) {
    info(&message);
}

/// napi-exported wrapper of [`warning`].
#[cfg(feature = "napi")]
#[cfg_attr(feature = "napi", napi)]
pub fn log_warning(message: String) {
    warning(&message);
}
