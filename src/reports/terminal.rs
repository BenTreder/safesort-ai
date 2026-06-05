use super::ScanReport;
use std::fmt::Write;

/// Render a scan report to the terminal with premium formatting.
pub fn render(report: &ScanReport) -> String {
    let mut out = String::new();

    let w = |out: &mut String, args: std::fmt::Arguments| {
        write!(out, "{args}").unwrap();
    };

    // Header
    w(
        &mut out,
        format_args!("\n  ╔═══════════════════════════════════════════════════╗\n"),
    );
    w(
        &mut out,
        format_args!("  ║        SafeSort AI — Safety-First Folder Organizer ║\n"),
    );
    w(
        &mut out,
        format_args!("  ╚═══════════════════════════════════════════════════╝\n\n"),
    );

    // Scan target
    w(
        &mut out,
        format_args!("  Scan target: {}\n\n", report.scan_target),
    );

    // Safety summary
    w(&mut out, format_args!("  Safety summary:\n\n"));
    w(
        &mut out,
        format_args!("    🔒 {:>12}  {:>6}\n", "LOCKED", report.summary.locked),
    );
    w(
        &mut out,
        format_args!("    ⚠️  {:>12}  {:>6}\n", "REVIEW", report.summary.review),
    );
    w(
        &mut out,
        format_args!(
            "    ✅ {:>12}  {:>6}\n",
            "SAFE CANDIDATES", report.summary.safe_candidate
        ),
    );
    w(&mut out, format_args!("    {:─>30}\n", ""));
    w(
        &mut out,
        format_args!("    {:>18}  {:>6}\n", "TOTAL", report.summary.total),
    );
    if report.summary.skipped > 0 {
        w(
            &mut out,
            format_args!(
                "    ⊘  {:>12}  {:>6}  (--exclude)\n",
                "SKIPPED", report.summary.skipped
            ),
        );
    }
    w(&mut out, format_args!("\n"));

    // Impact summary
    w(&mut out, format_args!("  Impact summary:\n\n"));
    w(
        &mut out,
        format_args!(
            "    🔴 {:>12}  {:>6}\n",
            "CRITICAL", report.summary.impact_critical
        ),
    );
    w(
        &mut out,
        format_args!("    🟠 {:>12}  {:>6}\n", "HIGH", report.summary.impact_high),
    );
    w(
        &mut out,
        format_args!(
            "    ⚠️  {:>12}  {:>6}\n",
            "MEDIUM", report.summary.impact_medium
        ),
    );
    w(
        &mut out,
        format_args!("    🟢 {:>12}  {:>6}\n", "LOW", report.summary.impact_low),
    );
    w(
        &mut out,
        format_args!("    ✅ {:>12}  {:>6}\n", "NONE", report.summary.impact_none),
    );
    w(&mut out, format_args!("\n"));

    // Profile
    if let Some(ref profile) = report.profile {
        w(&mut out, format_args!("  Detected profile:\n\n"));

        let sorted = profile.sorted_scores();
        for (name, score) in sorted.iter().filter(|(_, s)| s.score > 0.0) {
            let conf_indicator = match score.confidence.as_str() {
                "high" => "●●●",
                "medium" => "●●○",
                "low" => "●○○",
                "baseline" => "○○○",
                _ => "   ",
            };
            w(
                &mut out,
                format_args!(
                    "    {:<30} {:>8}  {}\n",
                    name,
                    format!("({})", score.score),
                    conf_indicator
                ),
            );
        }
        w(&mut out, format_args!("\n"));
    }

    // Protected examples with impact inline
    w(&mut out, format_args!("  Protected examples:\n\n"));

    let levels: &[(&str, &str)] = &[("LOCKED", "🔒"), ("REVIEW", "⚠️ "), ("SAFE", "✅")];

    for (level, icon) in levels {
        let examples = report.get_examples(level, 3);
        for item in examples {
            let reason = item
                .reasons
                .first()
                .map(|s| s.as_str())
                .unwrap_or("No specific reason");

            let impact_icon = match item.impact_level.as_str() {
                "CRITICAL" => "🔴",
                "HIGH" => "🟠",
                "MEDIUM" => "⚠️ ",
                "LOW" => "🟢",
                _ => "  ",
            };

            w(&mut out, format_args!("    {}  {:<8}\n", icon, level));
            w(&mut out, format_args!("       {}\n", item.path));
            w(
                &mut out,
                format_args!(
                    "       Impact: {} {}    Reason: {}\n\n",
                    impact_icon, item.impact_level, reason
                ),
            );
        }
    }

    // Footer
    w(&mut out, format_args!("  Nothing was moved.\n\n"));

    out
}
