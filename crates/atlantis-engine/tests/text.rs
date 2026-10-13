use atlantis_engine::text::ReportWriter;

const REPORTS: &[&str] = &[
    include_str!("fixtures/text/standard-turn-0-report-3.txt"),
    include_str!("fixtures/text/neworigins-turn-8-report-3.txt"),
];

// 0-based, end-exclusive physical-line ranges pinned independently of the writer.
const REFERENCE_GROUPS: &[&[(usize, usize)]] = &[
    &[
        (19, 25),
        (26, 31),
        (32, 47),
        (50, 52),
        (63, 65),
        (66, 70),
        (84, 86),
        (87, 91),
    ],
    &[
        (26, 28),
        (32, 34),
        (39, 41),
        (52, 56),
        (56, 60),
        (60, 64),
        (64, 68),
        (68, 72),
        (72, 76),
        (78, 82),
        (92, 96),
        (96, 100),
        (100, 104),
        (104, 108),
        (108, 112),
        (112, 116),
        (118, 122),
        (128, 132),
        (132, 136),
        (136, 140),
        (140, 144),
        (144, 148),
        (148, 152),
        (154, 158),
        (170, 172),
        (172, 174),
        (174, 176),
        (176, 178),
        (178, 180),
        (183, 191),
        (201, 203),
        (203, 210),
        (221, 233),
        (233, 236),
        (236, 239),
        (239, 242),
        (242, 245),
        (245, 248),
    ],
];

#[test]
fn reference_reports_reflow_to_the_exact_cxx_text() {
    for (fixture_index, expected) in REPORTS.iter().enumerate() {
        let mut report = ReportWriter::new();
        let reference_groups = REFERENCE_GROUPS[fixture_index];
        let lines: Vec<_> = expected
            .strip_suffix('\n')
            .unwrap_or(expected)
            .split('\n')
            .collect();
        let mut index = 0;
        let mut group_index = 0;
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
            let group_end = match reference_groups.get(group_index) {
                Some(&(start, end)) if start == index => {
                    group_index += 1;
                    end
                }
                _ => index + 1,
            };
            assert!(group_end <= lines.len());
            let group = &lines[index..group_end];
            let input = unwrapped_input(group, comment, indent).unwrap_or_else(|| {
                panic!("reference group {fixture_index}:{index}..{group_end} has no matching input")
            });
            report.clear_tab();
            for _ in 0..indent / 2 {
                report.add_tab();
            }
            report.put_str(&input, comment);
            index = group_end;
        }
        assert_eq!(report.as_bytes(), expected.as_bytes());
        assert_eq!(group_index, reference_groups.len());
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
            return writer.as_bytes() == expected.as_bytes();
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
