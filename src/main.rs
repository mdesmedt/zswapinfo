//! Show the current state of zswap.
//!
//! Sources:
//! - `/sys/module/zswap/parameters/*`: settings, readable by anyone.
//! - `/proc/meminfo`: `Zswap` (compressed pool size) and `Zswapped` (raw size
//!   of the pages in it).
//!
//! Everything is readable without root.

use std::fs;
use std::path::Path;

const PARAMS: &str = "/sys/module/zswap/parameters";
const MIB: f64 = 1024.0 * 1024.0;

/// A zswap module parameter.
fn param(name: &str) -> Option<String> {
    fs::read_to_string(Path::new(PARAMS).join(name))
        .ok()
        .map(|s| s.trim().to_string())
}

/// A /proc/meminfo field, in bytes.
fn meminfo(text: &str, key: &str) -> Option<u64> {
    text.lines().find_map(|line| {
        let rest = line.strip_prefix(key)?.strip_prefix(':')?;
        let kb: u64 = rest.trim().trim_end_matches("kB").trim().parse().ok()?;
        Some(kb * 1024)
    })
}

/// Bytes as MiB, or "?" when unknown.
fn mib(bytes: Option<u64>) -> String {
    bytes.map_or("?".to_string(), |b| format!("{:.1} MiB", b as f64 / MIB))
}

/// `part` as a percentage of `whole`, or "?" when unknown or `whole` is 0.
fn pct(part: Option<u64>, whole: Option<u64>) -> String {
    match (part, whole) {
        (Some(p), Some(w)) if w > 0 => format!("{:.1}%", p as f64 * 100.0 / w as f64),
        _ => "?".to_string(),
    }
}

/// A row with a left-aligned value, for settings.
fn row(label: &str, value: &str) {
    println!("  {label:<22}{value}");
}

/// A row with a right-aligned value and an optional note, for numbers.
fn num_row(label: &str, value: &str, note: &str) {
    let line = format!("  {label:<22}{value:>14}   {note}");
    println!("{}", line.trim_end());
}

fn main() {
    if param("enabled").is_none() {
        eprintln!("zswap is not available on this kernel ({PARAMS} missing)");
        std::process::exit(1);
    }
    let meminfo_text = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mi = |key| meminfo(&meminfo_text, key);

    let mem_total = mi("MemTotal");
    let swap_total = mi("SwapTotal");
    let swap_used = swap_total
        .zip(mi("SwapFree"))
        .map(|(t, f)| t.saturating_sub(f));
    // Both are missing on kernels older than 5.19.
    let pool = mi("Zswap");
    let stored = mi("Zswapped");
    let max_pool_percent = param("max_pool_percent").and_then(|v| v.parse::<u64>().ok());
    let pool_limit = mem_total.zip(max_pool_percent).map(|(m, p)| m * p / 100);

    let setting = |name| param(name).unwrap_or_else(|| "?".to_string());
    println!("Settings");
    row("enabled", &setting("enabled"));
    row("compressor", &setting("compressor"));
    row("max_pool_percent", &setting("max_pool_percent"));
    row("accept_threshold", &setting("accept_threshold_percent"));
    row("shrinker_enabled", &setting("shrinker_enabled"));

    let limit_pct = max_pool_percent.map_or("?".to_string(), |p| format!("{p}%"));
    // An empty pool has no ratio; leave the value blank.
    let ratio = match (stored, pool) {
        (Some(_), Some(0)) => String::new(),
        (Some(s), Some(p)) => format!("{:.2}", s as f64 / p as f64),
        _ => "?".to_string(),
    };
    let saved = stored.zip(pool).map(|(s, p)| s.saturating_sub(p));

    println!();
    println!("RAM (zswap pool)");
    num_row(
        "limit",
        &mib(pool_limit),
        &format!("{limit_pct} of {} RAM", mib(mem_total)),
    );
    num_row(
        "used (compressed)",
        &mib(pool),
        &format!("{} of limit", pct(pool, pool_limit)),
    );
    num_row("used (raw)", &mib(stored), "");
    num_row("compression ratio", &ratio, "");
    num_row("saved", &mib(saved), "");

    println!();
    println!("Swap device(s)");
    num_row("total", &mib(swap_total), "");
    num_row(
        "used",
        &mib(swap_used),
        &format!("{} of total", pct(swap_used, swap_total)),
    );
    // Every page in zswap holds a swap slot, so it counts as used swap.
    num_row(
        "used by zswap",
        &mib(stored),
        &format!("{} of used", pct(stored, swap_used)),
    );
}

#[cfg(test)]
mod tests {
    use super::meminfo;

    const MEMINFO: &str = "\
MemTotal:       16264804 kB
SwapCached:        12345 kB
SwapTotal:       8388604 kB
SwapFree:        6686652 kB
Zswap:            516412 kB
Zswapped:        1394176 kB
";

    /// Keys sharing a prefix (`Zswap`/`Zswapped`, `Swap*`) must not match
    /// each other's lines.
    #[test]
    fn meminfo_matches_exact_key() {
        assert_eq!(meminfo(MEMINFO, "MemTotal"), Some(16264804 * 1024));
        assert_eq!(meminfo(MEMINFO, "SwapTotal"), Some(8388604 * 1024));
        assert_eq!(meminfo(MEMINFO, "SwapFree"), Some(6686652 * 1024));
        assert_eq!(meminfo(MEMINFO, "Zswap"), Some(516412 * 1024));
        assert_eq!(meminfo(MEMINFO, "Zswapped"), Some(1394176 * 1024));
        assert_eq!(meminfo(MEMINFO, "Swap"), None);

        let reversed = "Zswapped:  2048 kB\nZswap:  1024 kB\n";
        assert_eq!(meminfo(reversed, "Zswap"), Some(1024 * 1024));
    }

    /// Kernels before 5.19 have no `Zswap`/`Zswapped` lines.
    #[test]
    fn meminfo_missing_key() {
        let text = "MemTotal:  1000 kB\nSwapTotal:  0 kB\n";
        assert_eq!(meminfo(text, "Zswap"), None);
        assert_eq!(meminfo(text, "Zswapped"), None);
    }
}
