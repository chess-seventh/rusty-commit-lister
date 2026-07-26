//! OSC 52 terminal-escape [`ClipboardPort`] adapter (supersedes ADR-004's arboard).
//!
//! Copies by asking the terminal emulator itself to set the clipboard via the
//! OSC 52 escape sequence. This works on Wayland (Hyprland), X11, and — unlike a
//! system-library backend — over plain SSH and inside tmux (with
//! `set-clipboard on`), with zero system dependencies.
//!
//! Only copying is supported (we never read the clipboard back), which is all the
//! `Ctrl-Y/E/P/U` actions need.

use std::io::Write;

use crate::error::{Result, RustyCommitListerError};
use crate::ports::clipboard_port::ClipboardPort;
use crate::ports::config_port::Probe;

/// Standard base64 alphabet (RFC 4648).
const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encodes `input` as standard base64 with `=` padding.
///
/// Pure function - no I/O, no mutation. Kept dependency-free so the crate needs
/// no external base64 (and no `unsafe`).
fn base64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        let n = (u32::from(b0) << 16) | (u32::from(b1) << 8) | u32::from(b2);
        out.push(B64[((n >> 18) & 63) as usize] as char);
        out.push(B64[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            B64[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// Builds the OSC 52 clipboard-set escape sequence that copies `text` to the
/// `c` (clipboard) selection: `ESC ] 52 ; c ; <base64> BEL`.
///
/// Pure function - no I/O, no mutation.
pub fn osc52_sequence(text: &str) -> String {
    format!("\u{1b}]52;c;{}\u{7}", base64_encode(text.as_bytes()))
}

/// Clipboard adapter that copies via the OSC 52 terminal escape sequence.
#[derive(Default)]
pub struct Osc52ClipboardAdapter;

impl Osc52ClipboardAdapter {
    /// Create a new adapter (stateless).
    pub fn new() -> Self {
        Self
    }
}

impl Probe for Osc52ClipboardAdapter {
    /// OSC 52 needs no system backend — it writes to the terminal the app already
    /// owns — so the probe always succeeds. If the terminal ignores OSC 52 the
    /// copy is a harmless no-op.
    fn probe(&self) -> Result<()> {
        Ok(())
    }
}

impl ClipboardPort for Osc52ClipboardAdapter {
    fn write(&self, text: &str) -> Result<()> {
        let seq = osc52_sequence(text);
        let mut out = std::io::stdout();
        out.write_all(seq.as_bytes())
            .and_then(|()| out.flush())
            .map_err(|e| RustyCommitListerError::clipboard_unavailable(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::{base64_encode, osc52_sequence};

    /// Scenario: base64 matches known RFC 4648 vectors (incl. padding)
    #[test]
    fn base64_encode_matches_known_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    /// Scenario: the OSC 52 sequence wraps the base64 payload in ESC]52;c;…BEL
    #[test]
    fn osc52_sequence_wraps_base64_payload() {
        assert_eq!(osc52_sequence("foo"), "\u{1b}]52;c;Zm9v\u{7}");
    }

    /// Scenario: a Unicode payload round-trips its UTF-8 bytes through base64
    #[test]
    fn osc52_sequence_handles_unicode() {
        // "📅" is 4 UTF-8 bytes → 8 base64 chars (with padding).
        let seq = osc52_sequence("📅");
        assert!(seq.starts_with("\u{1b}]52;c;"));
        assert!(seq.ends_with('\u{7}'));
        assert_eq!(base64_encode("📅".as_bytes()), "8J+ThQ==");
    }
}
