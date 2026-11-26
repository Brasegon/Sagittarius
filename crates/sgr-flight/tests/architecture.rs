//! Dependency rules of this crate that the compiler cannot check.
//!
//! `sgr-flight` depends on `sgr-protocol` for its equipment drivers only
//! (`docs/decisions/0003-pilotes-d-equipements.md`):
//!
//! - outside `src/drivers.rs` and `src/drivers/`, no code names `sgr_protocol`: the subsystems
//!   do not see message formats;
//! - inside, only its `equipment` module is used, always written `sgr_protocol::equipment`.
//!
//! These tests read the source files of the crate as text. Comment lines do not count.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Name of the protocol crate, as code writes it.
const PROTOCOL: &str = "sgr_protocol";

/// What must follow the name of the protocol crate in a driver.
const ALLOWED_MODULE: &str = "::equipment";

/// One line of code that names the protocol crate.
struct ProtocolUse {
    /// Path of the file, relative to `src/`.
    file: PathBuf,
    /// Line number, starting at 1.
    line: usize,
    /// The line itself, without its indentation.
    text: String,
}

impl ProtocolUse {
    /// Whether the line belongs to the equipment drivers.
    fn is_in_drivers(&self) -> bool {
        self.file.starts_with("drivers") || self.file == Path::new("drivers.rs")
    }

    /// Whether every mention of the protocol crate on the line goes through its `equipment`
    /// module.
    fn only_names_equipment(&self) -> bool {
        self.text
            .match_indices(PROTOCOL)
            .all(|(start, _)| self.text[start + PROTOCOL.len()..].starts_with(ALLOWED_MODULE))
    }
}

/// Adds to `found` every `.rs` file under `dir`, at any depth.
fn rust_files(dir: &Path, found: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            rust_files(&path, found)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
    Ok(())
}

/// Every line of code of `src/` that names the protocol crate.
fn protocol_uses() -> io::Result<Vec<ProtocolUse>> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files)?;
    files.sort();

    let mut uses = Vec::new();
    for file in files {
        let content = fs::read_to_string(&file)?;
        let relative = file.strip_prefix(&src).unwrap_or(&file);

        for (index, line) in content.lines().enumerate() {
            let text = line.trim();
            if text.starts_with("//") || !text.contains(PROTOCOL) {
                continue;
            }
            uses.push(ProtocolUse {
                file: relative.to_path_buf(),
                line: index + 1,
                text: text.to_string(),
            });
        }
    }
    Ok(uses)
}

/// One offending line per row, for the message of a failed test.
fn report(offenders: &[&ProtocolUse]) -> String {
    let mut rows = String::new();
    for offender in offenders {
        rows.push_str(&format!(
            "  src/{}:{}: {}\n",
            offender.file.display(),
            offender.line,
            offender.text
        ));
    }
    rows
}

#[test]
fn only_the_drivers_name_the_protocol_crate() -> io::Result<()> {
    let uses = protocol_uses()?;
    let offenders: Vec<&ProtocolUse> = uses.iter().filter(|found| !found.is_in_drivers()).collect();

    assert!(
        offenders.is_empty(),
        "`{PROTOCOL}` is named outside the drivers. Import the item from its driver instead, \
         which re-exports it:\n{}",
        report(&offenders)
    );
    Ok(())
}

#[test]
fn drivers_only_use_the_equipment_module() -> io::Result<()> {
    let uses = protocol_uses()?;
    let offenders: Vec<&ProtocolUse> = uses
        .iter()
        .filter(|found| found.is_in_drivers() && !found.only_names_equipment())
        .collect();

    assert!(
        offenders.is_empty(),
        "a driver names `{PROTOCOL}` without going through `{PROTOCOL}{ALLOWED_MODULE}`:\n{}",
        report(&offenders)
    );
    Ok(())
}

#[test]
fn scan_finds_the_pcdu_driver() -> io::Result<()> {
    // Guards the two tests above: if the scan found nothing, they would pass for no reason.
    let uses = protocol_uses()?;

    assert!(
        uses.iter()
            .any(|found| found.file == Path::new("drivers/pcdu.rs")),
        "the scan of `src/` did not find the PCDU driver, which uses `{PROTOCOL}`"
    );
    Ok(())
}
