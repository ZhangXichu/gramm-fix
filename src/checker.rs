pub struct Correction {
    pub corrected: String,
    /// Byte offset and byte length of the corrected span within `corrected`
    pub span: (usize, usize),
    pub explanation: String,
}

/// Check `sentence` for grammar errors and return a correction if found.
/// TODO: integrate llama-3.3-70b via hypereal.tech API
pub fn check(_sentence: &str) -> Option<Correction> {
    None
}
