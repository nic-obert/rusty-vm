
pub struct SourceToken<'a> {
    /// Line index.
    pub line: usize,
    /// Column index.
    pub column: usize,
    /// The actual string from the source code.
    pub string: &'a str,
}
