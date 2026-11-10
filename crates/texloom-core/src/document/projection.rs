//! Monaco sees LF text without the leading UTF-8 BOM. Canonical buffers keep
//! their exact bytes; each edit is mapped back, never a whole-file serialization.
use super::{EditError, EditResult, Replacement};
use super::position::{EditorUtf16Offset, SourceByteOffset};

pub struct EditorProjection<'a> {
    source: &'a str,
    text: String,
    boundaries: Vec<Option<usize>>,
}
impl<'a> EditorProjection<'a> {
    pub fn new(source: &'a str) -> Self {
        let start = if source.starts_with('\u{feff}') { 3 } else { 0 };
        let mut text = String::new();
        let mut boundaries = vec![Some(start)];
        let mut chars = source
            .get(start..)
            .unwrap_or_default()
            .char_indices()
            .peekable();
        while let Some((offset, c)) = chars.next() {
            let mut end = start + offset + c.len_utf8();
            if c == '\r' {
                if chars.peek().is_some_and(|(_, c)| *c == '\n') {
                    chars.next();
                    end += 1;
                }
                text.push('\n');
            } else {
                text.push(c);
                if c.len_utf16() == 2 {
                    boundaries.push(None);
                }
            }
            boundaries.push(Some(end));
        }
        Self {
            source,
            text,
            boundaries,
        }
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn source_position(&self, offset: EditorUtf16Offset) -> EditResult<SourceByteOffset> {
        SourceByteOffset::checked(self.source, self.source_offset(offset.get())?)
    }
    pub fn editor_position(&self, offset: SourceByteOffset) -> EditResult<EditorUtf16Offset> {
        self.editor_offset(offset.get()).map(EditorUtf16Offset::new)
    }
    pub fn source_offset(&self, utf16: usize) -> EditResult<usize> {
        self.boundaries
            .get(utf16)
            .copied()
            .flatten()
            .ok_or(EditError::InvalidEditorOffset)
    }
    pub fn editor_offset(&self, byte: usize) -> EditResult<usize> {
        self.boundaries
            .iter()
            .position(|p| *p == Some(byte))
            .ok_or(EditError::InvalidBoundary)
    }
    pub fn replacement(&self, start: usize, end: usize, text: &str) -> EditResult<Replacement> {
        if start > end {
            return Err(EditError::InvalidRange);
        }
        let range = self.source_offset(start)?..self.source_offset(end)?;
        let expected = self
            .source
            .get(range.clone())
            .ok_or(EditError::InvalidRange)?
            .to_string();
        let eol = line_ending_at(self.source, range.start);
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        Ok(Replacement {
            range,
            expected,
            text: if eol == "\n" {
                normalized
            } else {
                normalized.replace('\n', eol)
            },
        })
    }
}
fn line_ending_at(source: &str, position: usize) -> &'static str {
    // Prefer the current line's terminator. At EOF borrow the previous line.
    let suffix = source.get(position..).unwrap_or_default();
    for (i, c) in suffix.char_indices() {
        if c == '\r' {
            return if suffix.as_bytes().get(i + 1) == Some(&b'\n') {
                "\r\n"
            } else {
                "\r"
            };
        }
        if c == '\n' {
            return "\n";
        }
    }
    let prefix = source.get(..position).unwrap_or_default();
    for (i, c) in prefix.char_indices().rev() {
        if c == '\n' {
            return if i > 0 && prefix.as_bytes().get(i - 1) == Some(&b'\r') {
                "\r\n"
            } else {
                "\n"
            };
        }
        if c == '\r' {
            return "\r";
        }
    }
    "\n"
}
