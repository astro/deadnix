//! Dead code reporting in human-readable and JSON formats

use crate::dead_code::DeadCode;
use ariadne::{sources, Config, Label, Report, ReportKind};
use rowan::ast::AstNode;
use std::{collections::BTreeMap, env, ops::Range};

#[cfg(feature = "json-out")]
use serde_json::json;

/// Build a report and print it to stdout
///
/// assumes results to be sorted by occurrence in file
pub fn print(file: String, content: &str, results: &[DeadCode]) {
    let no_color = env::var("NO_COLOR").is_ok();

    // advance into content to convert byte offsets into char offsets
    let mut content_bytes = 0;
    let mut content_chars = 0usize;
    let mut chars = content.chars();
    let mut result_ranges_by_line = BTreeMap::<usize, Vec<(&DeadCode, Range<usize>)>>::new();
    let mut line = 1;
    for result in results {
        let range = result.binding.name.syntax().text_range();
        let start_byte = usize::from(range.start());
        while content_bytes < start_byte {
            let b = chars.next().unwrap();
            if b == '\n' {
                line += 1;
            }
            content_bytes += b.len_utf8();
            content_chars += 1;
        }
        let start_char = content_chars;
        let end_byte = usize::from(range.end());
        while content_bytes < end_byte {
            let b = chars.next().unwrap();
            if b == '\n' {
                line += 1;
            }
            content_bytes += b.len_utf8();
            content_chars += 1;
        }
        let end_char = content_chars;

        result_ranges_by_line
            .entry(line)
            .or_default()
            .push((result, start_char..end_char));
    }

    let mut builder = Report::build(
        ReportKind::Warning,
        (
            file.clone(),
            result_ranges_by_line.first_key_value().unwrap().1[0]
                .1
                .start
                .into()
                ..result_ranges_by_line
                    .last_key_value()
                    .unwrap()
                    .1
                    .last()
                    .unwrap()
                    .1
                    .end
                    .into(),
        ),
    )
    .with_config(Config::default().with_compact(true).with_color(!no_color))
    .with_message("Unused declarations were found.");
    let mut order = 0;
    for (_line, ranges) in result_ranges_by_line.into_iter() {
        for (result, range) in ranges.into_iter().rev() {
            // add report label
            let mut label = Label::new((file.clone(), range))
                .with_message(format!("{result}"))
                .with_order(order as i32);
            order += 1;
            if !no_color {
                label = label.with_color(result.scope.color());
            }
            builder = builder.with_label(label);
        }
    }
    // print
    builder
        .finish()
        .print(sources(vec![(file.clone(), content)]))
        .unwrap();
}

/// Print dead code to stdout in JSON
#[cfg(feature = "json-out")]
pub fn print_json(file: &str, content: &str, results: &[DeadCode]) {
    let mut offset = 0;
    let mut offsets = vec![offset];
    while let Some(next) = content[offset..].find('\n') {
        offset += next + 1;
        offsets.push(offset);
    }

    let json = json!({
        "file": file,
        "results": results.iter().map(|result| {
            let range = result.binding.name.syntax().text_range();
            let start = usize::from(range.start());
            let mut line_number = 0;
            let mut line_offset = 0;
            for &offset in &offsets {
                if start < offset {
                    break;
                }
                line_number += 1;
                line_offset = offset;
            }
            json!({
                "message": format!("{result}"),
                "line": line_number,
                "column": start - line_offset + 1,
                "endColumn": usize::from(range.end()) - line_offset + 1,
            })
        }).collect::<serde_json::Value>(),
    });
    println!("{json}");
}
