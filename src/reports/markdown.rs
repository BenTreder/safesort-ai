use super::ScanReport;
use std::fmt::Write;

pub fn render(report: &ScanReport) -> String {
    let mut out = String::new();

    writeln!(out, "# SafeSort AI — Scan Report").unwrap();
    writeln!(out, "").unwrap();
    writeln!(out, "- **Scan target:** `{}`", report.scan_target).unwrap();
    writeln!(out, "- **Generated:** {}", report.generated_at).unwrap();
    writeln!(out, "").unwrap();

    // Summary
    writeln!(out, "## Safety Summary").unwrap();
    writeln!(out, "").unwrap();
    writeln!(out, "| Level | Count |").unwrap();
    writeln!(out, "|-------|-------|").unwrap();
    writeln!(out, "| 🔒 LOCKED | {} |", report.summary.locked).unwrap();
    writeln!(out, "| ⚠️  REVIEW | {} |", report.summary.review).unwrap();
    writeln!(
        out,
        "| ✅ SAFE CANDIDATES | {} |",
        report.summary.safe_candidate
    )
    .unwrap();
    writeln!(out, "| **Total** | **{}** |", report.summary.total).unwrap();
    writeln!(out, "").unwrap();

    // Profile
    if let Some(ref profile) = report.profile {
        writeln!(out, "## Detected Profile").unwrap();
        writeln!(out, "").unwrap();
        writeln!(out, "| Profile | Score | Confidence |").unwrap();
        writeln!(out, "|---------|-------|------------|").unwrap();

        let sorted = profile.sorted_scores();
        for (name, score) in sorted.iter().filter(|(_, s)| s.score > 0.0) {
            writeln!(
                out,
                "| {} | {:.1} | {} |",
                name, score.score, score.confidence
            )
            .unwrap();
        }
        writeln!(out, "").unwrap();
    }

    // Detailed items
    for (level, items) in &report.items {
        if items.is_empty() {
            continue;
        }
        let emoji = match level.as_str() {
            "LOCKED" => "🔒",
            "REVIEW" => "⚠️ ",
            _ => "✅",
        };
        writeln!(out, "## {emoji} {level} ({})", items.len()).unwrap();
        writeln!(out, "").unwrap();

        for item in items {
            writeln!(out, "- `{}` — score: {:.2}", item.path, item.score).unwrap();
            for reason in &item.reasons {
                writeln!(out, "  - {reason}").unwrap();
            }
        }
        writeln!(out, "").unwrap();
    }

    writeln!(out, "---").unwrap();
    writeln!(out, "").unwrap();
    writeln!(out, "*Nothing was moved. This is a read-only scan.*").unwrap();

    out
}
