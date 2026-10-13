use super::trunc;

/// In-memory counterpart of `Areport`.
#[derive(Default)]
pub struct ReportWriter {
    output: Vec<u8>,
    tabs: usize,
}

impl ReportWriter {
    /// C++: `fileio.cpp` `Areport::Areport`/`Open` initialize tabs to zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// C++: `fileio.cpp` `Areport::AddTab`.
    pub fn add_tab(&mut self) {
        self.tabs += 1;
    }

    /// C++: `fileio.cpp` `Areport::DropTab`.
    pub fn drop_tab(&mut self) {
        self.tabs = self.tabs.saturating_sub(1);
    }

    /// C++: `fileio.cpp` `Areport::ClearTab`.
    pub fn clear_tab(&mut self) {
        self.tabs = 0;
    }

    /// C++: `fileio.cpp` `Areport::PutStr` (lines 278-298).
    pub fn put_str(&mut self, text: &str, comment: bool) {
        let mut current = self.indentation();
        current.extend_from_slice(text.as_bytes());
        loop {
            let remainder = trunc(&mut current, 70, 30);
            if comment {
                self.output.push(b';');
            }
            self.output.extend_from_slice(&current);
            self.output.push(b'\n');
            let Some(remainder) = remainder else {
                break;
            };
            current = b"  ".to_vec();
            current.extend_from_slice(&self.indentation());
            current.extend_from_slice(&remainder);
        }
    }

    /// C++: `fileio.cpp` `Areport::PutNoFormat`.
    pub fn put_no_format(&mut self, text: &str) {
        self.output.extend_from_slice(text.as_bytes());
        self.output.push(b'\n');
    }

    /// C++: `fileio.cpp` `Areport::EndLine`.
    pub fn end_line(&mut self) {
        self.output.push(b'\n');
    }

    /// Return the bytes written so far.
    pub fn as_bytes(&self) -> &[u8] {
        &self.output
    }

    fn indentation(&self) -> Vec<u8> {
        b"  ".repeat(self.tabs)
    }
}

/// In-memory counterpart of `Arules`.
#[derive(Default)]
pub struct RulesWriter {
    output: Vec<u8>,
    tabs: usize,
    wrap_tabs: usize,
}

impl RulesWriter {
    /// C++: `fileio.cpp` `Arules::Arules`/`Open` initialize both tab levels to zero.
    pub fn new() -> Self {
        Self::default()
    }

    /// C++: `fileio.cpp` `Arules::AddTab`.
    pub fn add_tab(&mut self) {
        self.tabs += 1;
    }

    /// C++: `fileio.cpp` `Arules::DropTab`.
    pub fn drop_tab(&mut self) {
        self.tabs = self.tabs.saturating_sub(1);
    }

    /// C++: `fileio.cpp` `Arules::ClearTab`.
    pub fn clear_tab(&mut self) {
        self.tabs = 0;
    }

    /// C++: `fileio.cpp` `Arules::AddWrapTab`.
    pub fn add_wrap_tab(&mut self) {
        self.wrap_tabs += 1;
    }

    /// C++: `fileio.cpp` `Arules::DropWrapTab`.
    pub fn drop_wrap_tab(&mut self) {
        self.wrap_tabs = self.wrap_tabs.saturating_sub(1);
    }

    /// C++: `fileio.cpp` `Arules::ClearWrapTab`.
    pub fn clear_wrap_tab(&mut self) {
        self.wrap_tabs = 0;
    }

    /// C++: `fileio.cpp` `Arules::PutStr` (lines 367-382).
    pub fn put_str(&mut self, text: &str) {
        let mut current = self.indentation(self.tabs);
        current.extend_from_slice(text.as_bytes());
        loop {
            let remainder = trunc(&mut current, 78, 70);
            self.output.extend_from_slice(&current);
            self.output.push(b'\n');
            let Some(remainder) = remainder else {
                break;
            };
            current = self.indentation(self.tabs);
            current.extend_from_slice(&remainder);
        }
    }

    /// C++: `fileio.cpp` `Arules::WrapStr` (lines 384-395).
    pub fn wrap_str(&mut self, text: &str) {
        let mut current = self.indentation(self.wrap_tabs);
        current.extend_from_slice(text.as_bytes());
        loop {
            let remainder = trunc(&mut current, 70, 30);
            self.output.extend_from_slice(&current);
            self.output.push(b'\n');
            let Some(remainder) = remainder else {
                break;
            };
            current = b"  ".to_vec();
            current.extend_from_slice(&self.indentation(self.wrap_tabs));
            current.extend_from_slice(&remainder);
        }
    }

    /// C++: `fileio.cpp` `Arules::PutNoFormat`.
    pub fn put_no_format(&mut self, text: &str) {
        self.output.extend_from_slice(text.as_bytes());
        self.output.push(b'\n');
    }

    /// C++: `fileio.cpp` `Arules::EndLine`.
    pub fn end_line(&mut self) {
        self.output.push(b'\n');
    }

    /// C++: `fileio.cpp` `Arules::Enclose`.
    pub fn enclose(&mut self, open: bool, tag: &str) {
        if open {
            self.put_str(&format!("<{tag}>"));
            self.add_tab();
        } else {
            self.drop_tab();
            self.put_str(&format!("</{tag}>"));
        }
    }

    /// C++: `fileio.cpp` `Arules::TagText`.
    pub fn tag_text(&mut self, tag: &str, text: &str) {
        self.enclose(true, tag);
        self.put_str(text);
        self.enclose(false, tag);
    }

    /// C++: `fileio.cpp` `Arules::ClassTagText`.
    pub fn class_tag_text(&mut self, tag: &str, class: &str, text: &str) {
        self.enclose(true, &format!("{tag} class=\"{class}\""));
        self.put_str(text);
        self.enclose(false, tag);
    }

    /// C++: `fileio.cpp` `Arules::Paragraph`.
    pub fn paragraph(&mut self, text: &str) {
        self.tag_text("p", text);
    }

    /// C++: `fileio.cpp` `Arules::CommandExample`.
    pub fn command_example(&mut self, header: &str, example: &str) {
        self.paragraph(header);
        self.paragraph("");
        self.enclose(true, "pre");
        self.put_no_format(example);
        self.enclose(false, "pre");
    }

    /// C++: `fileio.cpp` `Arules::Link`.
    pub fn link(href: &str, text: &str) -> String {
        format!("<a href=\"{href}\">{text}</a>")
    }

    /// C++: `fileio.cpp` `Arules::LinkRef`.
    pub fn link_ref(&mut self, name: &str) {
        self.put_str(&format!("<a name=\"{name}\"></a>"));
    }

    /// Return the bytes written so far.
    pub fn as_bytes(&self) -> &[u8] {
        &self.output
    }

    fn indentation(&self, tabs: usize) -> Vec<u8> {
        b"  ".repeat(tabs)
    }
}

#[cfg(test)]
mod tests {
    use super::{ReportWriter, RulesWriter};

    fn output_text(bytes: &[u8]) -> &str {
        std::str::from_utf8(bytes).expect("test output is valid UTF-8")
    }

    // C++: fileio.cpp Areport::PutStr, lines 278-298.
    #[test]
    fn report_writer_indents_and_comments_every_wrapped_line() {
        let mut report = ReportWriter::new();
        report.add_tab();
        report.put_str("nested", false);
        report.drop_tab();

        let text = "word ".repeat(20);
        report.put_str(&text, true);
        let lines: Vec<_> = output_text(report.as_bytes()).lines().collect();
        assert_eq!(lines[0], "  nested");
        assert_eq!(lines[1], format!(";{}word", "word ".repeat(13)));
        assert_eq!(lines[2], format!(";  {}", "word ".repeat(6)));
        assert!(lines.iter().all(|line| line.len() <= 71));
    }

    // C++: fileio.cpp Areport::PutStr and astring.cpp AString::Trunc.
    #[test]
    fn report_writer_keeps_short_embedded_newlines_verbatim() {
        let mut report = ReportWriter::new();
        report.put_str("first\nsecond", true);
        assert_eq!(report.as_bytes(), b";first\nsecond\n");
    }

    // C++: fileio.cpp Areport::PutStr and astring.cpp AString::Trunc, which count bytes.
    #[test]
    fn report_writer_wraps_at_byte_width_even_inside_utf8() {
        let mut report = ReportWriter::new();
        report.put_str(&format!("{}éz", "x".repeat(69)), false);

        let mut expected = vec![b'x'; 69];
        expected.extend_from_slice(&[0xc3, b'\n', b' ', b' ', 0xa9, b'z', b'\n']);
        assert_eq!(report.as_bytes(), expected);
    }

    // C++: fileio.cpp Areport::PutStr/PutNoFormat/EndLine, lines 278-307.
    #[test]
    fn report_writer_handles_no_format_and_empty_lines() {
        let mut report = ReportWriter::new();
        report.put_no_format("exact");
        report.end_line();
        report.add_tab();
        report.drop_tab();
        report.drop_tab();
        report.put_str("plain", false);
        assert_eq!(report.as_bytes(), b"exact\n\nplain\n");
    }

    // C++: fileio.cpp Arules::PutStr/WrapStr, lines 367-395.
    #[test]
    fn rules_writer_uses_distinct_78_and_70_column_wrapping() {
        let mut rules = RulesWriter::new();
        rules.put_str(&"x".repeat(79));
        rules.wrap_str(&"y".repeat(71));
        let expected = format!("{}\nx\n{}\n  y\n", "x".repeat(78), "y".repeat(70));
        assert_eq!(rules.as_bytes(), expected.as_bytes());
    }

    // C++: fileio.cpp Arules::PutStr/WrapStr, lines 367-395.
    #[test]
    fn rules_writer_uses_distinct_backtrack_windows_and_tab_levels() {
        let mut rules = RulesWriter::new();
        rules.add_tab();
        rules.put_str("plain");
        rules.add_wrap_tab();
        rules.wrap_str("wrapped");
        assert_eq!(rules.as_bytes(), b"  plain\n  wrapped\n");

        let mut put = RulesWriter::new();
        let put_text = format!("{} {}", "x".repeat(20), "y".repeat(64));
        put.put_str(&put_text);
        let put_expected = format!("{}\n{}\n", "x".repeat(20), "y".repeat(64));
        assert_eq!(put.as_bytes(), put_expected.as_bytes());

        let mut wrap = RulesWriter::new();
        let wrap_text = format!("{} {}", "x".repeat(20), "y".repeat(54));
        wrap.wrap_str(&wrap_text);
        let wrap_expected = format!(
            "{} {}\n  {}\n",
            "x".repeat(20),
            "y".repeat(49),
            "y".repeat(5)
        );
        assert_eq!(wrap.as_bytes(), wrap_expected.as_bytes());
    }

    // C++: fileio.cpp Arules::Enclose/TagText/Paragraph/CommandExample/Link/LinkRef.
    #[test]
    fn rules_writer_emits_html_helpers_verbatim() {
        let mut rules = RulesWriter::new();
        rules.paragraph("Text");
        rules.link_ref("begin");
        assert!(output_text(rules.as_bytes()).contains("<p>\n  Text\n</p>\n"));
        assert!(output_text(rules.as_bytes()).contains("<a name=\"begin\"></a>\n"));
        assert_eq!(RulesWriter::link("url", "text"), "<a href=\"url\">text</a>");
    }

    // C++: fileio.cpp Arules::ClassTagText/CommandExample, lines 430-462.
    #[test]
    fn rules_writer_emits_class_tags_and_command_examples() {
        let mut rules = RulesWriter::new();
        rules.class_tag_text("div", "notice", "Body");
        rules.command_example("Example", "atlantis run");
        assert_eq!(
            output_text(rules.as_bytes()),
            "<div class=\"notice\">\n  Body\n</div>\n\
             <p>\n  Example\n</p>\n\
             <p>\n  \n</p>\n\
             <pre>\natlantis run\n</pre>\n"
        );
    }
}
