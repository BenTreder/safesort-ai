use super::item::ScanItem;
use crate::error::Result;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Walk the filesystem from `root`, returning items up to `max_depth` levels deep.
pub fn walk(root: &PathBuf, max_depth: usize) -> Result<Vec<ScanItem>> {
    let root_depth = root.components().count();
    let mut items = Vec::new();

    for entry in WalkDir::new(root)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|_e| {
            // The entry itself still gets classified as LOCKED if encountered.
            true
        })
    {
        match entry {
            Ok(e) => {
                let item = ScanItem::from_entry(&e, root_depth);
                items.push(item);
            }
            Err(e) => {
                // Permission denied or other walk error — skip gracefully.
                tracing::debug!("Walker skip: {e}");
            }
        }
    }

    Ok(items)
}
