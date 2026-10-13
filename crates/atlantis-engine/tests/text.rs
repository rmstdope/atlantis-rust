use atlantis_engine::text::ReportWriter;

const REPORTS: &[&str] = &[
    include_str!("fixtures/text/standard-turn-0-report-3.txt"),
    include_str!("fixtures/text/neworigins-turn-8-report-3.txt"),
];

#[test]
fn reference_reports_reflow_to_the_exact_cxx_text() {
    for expected in REPORTS {
        let mut report = ReportWriter::new();
        let lines: Vec<_> = expected
            .strip_suffix('\n')
            .unwrap_or(expected)
            .split('\n')
            .collect();
        let mut index = 0;
        let mut reflowed_groups = 0;
        while index < lines.len() {
            let line = lines[index];
            if line.is_empty() {
                report.end_line();
                index += 1;
                continue;
            }

            let comment = line.starts_with(';');
            let body = line_body(line, comment);
            let indent = leading_spaces(body) / 2 * 2;
            let mut group = vec![line];
            while index + group.len() < lines.len() {
                let next = lines[index + group.len()];
                if next.is_empty() || next.starts_with(';') != comment {
                    break;
                }
                let next_body = line_body(next, comment);
                if !next_body.starts_with(&" ".repeat(indent + 2)) {
                    break;
                }

                let mut candidate = group.clone();
                candidate.push(next);
                if unwrapped_input(&candidate, comment, indent).is_none() {
                    break;
                }
                group.push(next);
                reflowed_groups += 1;
            }

            let input = unwrapped_input(&group, comment, indent)
                .expect("each physical report line can be represented as a writer input");
            report.clear_tab();
            for _ in 0..indent / 2 {
                report.add_tab();
            }
            report.put_str(&input, comment);
            index += group.len();
        }
        assert_eq!(report.as_str(), *expected);
        assert!(
            reflowed_groups > 0,
            "fixture should exercise continuation wrapping"
        );
    }
}

fn line_body(line: &str, comment: bool) -> &str {
    if comment {
        line.strip_prefix(';').expect("comment marker was checked")
    } else {
        line
    }
}

fn leading_spaces(input: &str) -> usize {
    input.bytes().take_while(|byte| *byte == b' ').count()
}

fn unwrapped_input(lines: &[&str], comment: bool, indent: usize) -> Option<String> {
    let continuation_indent = " ".repeat(indent + 2);
    let first_body = line_body(lines[0], comment);
    let first = first_body.strip_prefix(&" ".repeat(indent))?;
    let parts: Option<Vec<_>> = lines[1..]
        .iter()
        .map(|line| line_body(line, comment).strip_prefix(&continuation_indent))
        .collect();
    let parts = parts?;
    let expected = lines
        .iter()
        .map(|line| format!("{line}\n"))
        .collect::<String>();

    fn search(
        parts: &[&str],
        lines: &[&str],
        comment: bool,
        indent: usize,
        index: usize,
        input: &mut String,
        expected: &str,
    ) -> bool {
        if index == parts.len() {
            let mut writer = ReportWriter::new();
            for _ in 0..indent / 2 {
                writer.add_tab();
            }
            writer.put_str(input, comment);
            return writer.as_str() == expected;
        }

        let previous = line_body(lines[index], comment);
        let separators: &[&str] = match previous.len() {
            70 => &["", " ", "\n", "\r"],
            41..=69 => &[" ", "\n", "\r"],
            _ => &["\n", "\r"],
        };
        for separator in separators {
            let original_len = input.len();
            input.push_str(separator);
            input.push_str(parts[index]);
            if search(parts, lines, comment, indent, index + 1, input, expected) {
                return true;
            }
            input.truncate(original_len);
        }
        false
    }

    let mut input = first.to_owned();
    search(&parts, lines, comment, indent, 0, &mut input, &expected).then_some(input)
}
