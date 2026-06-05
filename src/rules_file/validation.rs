/// Path components or substrings that make a staging destination unsafe.
const UNSAFE_DESTINATION_PATTERNS: &[&str] = &[
    "/etc",
    "/usr",
    "/var",
    "/boot",
    "/run",
    "/proc",
    "/sys",
    "/dev",
    "public_html",
    "htdocs",
    "webroot",
    "live-site",
    "live_site",
    // bare "www" only when it appears as a path component, not as part of e.g. ~/Workspace
];

/// Return true if `dest` is safe to use as a recommendation destination.
///
/// A destination is safe if it does not match any known system or live-site
/// path patterns. This is checked case-insensitively. The destination is
/// never created or written to — this is display-only validation.
pub fn is_safe_destination(dest: &str) -> bool {
    let lower = dest.to_lowercase();
    for pat in UNSAFE_DESTINATION_PATTERNS {
        if lower.contains(pat) {
            return false;
        }
    }
    // Reject bare /www/ path component (not ~/www inside a workspace)
    if lower.starts_with("/www") || lower.contains("/www/") {
        return false;
    }
    // Reject absolute system roots unless tilde-prefixed
    if !dest.starts_with('~') && !dest.starts_with('.') {
        let abs = std::path::Path::new(dest);
        if abs.is_absolute() {
            // Allow only if it's clearly under a home-like dir
            let s = abs.to_string_lossy().to_lowercase();
            let is_home_like =
                s.contains("/home/") || s.contains("/users/") || s.contains("/root/");
            if !is_home_like {
                return false;
            }
        }
    }
    true
}

/// Human-readable rejection reason for an unsafe destination.
pub fn rejection_reason(dest: &str) -> String {
    let lower = dest.to_lowercase();
    for pat in UNSAFE_DESTINATION_PATTERNS {
        if lower.contains(pat) {
            return format!(
                "Unsafe destination '{}': matches restricted path pattern '{}'",
                dest, pat
            );
        }
    }
    format!(
        "Unsafe destination '{}': absolute system path — use ~/... notation instead",
        dest
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_destinations_pass() {
        assert!(is_safe_destination(
            "~/Workspace/03_Websites/BenTreder.com/Incoming Assets"
        ));
        assert!(is_safe_destination("~/Downloads/Sorted"));
        assert!(is_safe_destination("./staging"));
    }

    #[test]
    fn unsafe_system_paths_rejected() {
        assert!(!is_safe_destination("/etc/nginx"));
        assert!(!is_safe_destination("/usr/local/bin"));
        assert!(!is_safe_destination("/var/www/html"));
        assert!(!is_safe_destination("/boot"));
    }

    #[test]
    fn live_site_paths_rejected() {
        assert!(!is_safe_destination("/var/www/public_html"));
        assert!(!is_safe_destination("~/servers/htdocs"));
        assert!(!is_safe_destination("~/sites/live-site/uploads"));
        assert!(!is_safe_destination("/srv/webroot"));
    }

    #[test]
    fn bare_www_path_rejected() {
        assert!(!is_safe_destination("/www/uploads"));
    }
}
