use std::env;
use ariadne::{Config, Label, Report, ReportKind, sources};
use crate::dead_code::DeadCode;
use rnix::{TextSize, types::TypedNode};

#[cfg(feature = "json-out")]
use serde_json::json;

// assumes results to be sorted by occurrence in file
pub fn print(file: String, content: &str, results: &[DeadCode]) {
    let no_color = env::var("NO_COLOR").is_ok();

    let first_result_range = results[0].binding.name.node().text_range();
    let mut builder = Report::build(
        ReportKind::Warning,
        file.clone(),
        first_result_range.start().into()
    )
        .with_config(
            Config::default()
                .with_compact(true)
                .with_color(!no_color)
        )
        .with_message("Unused declarations were found.");

    // advance into content to convert byte offsets into char offsets
    let mut content_bytes = 0;
    let mut content_chars = 0usize;
    let mut char_bytes = content.chars()
        .map(|c| usize::from(TextSize::of(c)));
    // reverse order to avoid overlapping lanes
    let mut order = results.len();
    for result in results {
        order -= 1;

        let range = result.binding.name.node().text_range();
        let start_byte = usize::from(range.start());
        while content_bytes < start_byte {
            content_bytes += char_bytes.next().unwrap();
            content_chars += 1;
        }
        let start_char = content_chars;
        let end_byte = usize::from(range.end());
        while content_bytes < end_byte {
            content_bytes += char_bytes.next().unwrap();
            content_chars += 1;
        }
        let end_char = content_chars;

        // add report label
        let mut label = Label::new((file.clone(), start_char..end_char))
            .with_message(format!("{}", result))
            .with_order(order as i32);
        if !no_color {
            label = label.with_color(result.scope.color());
        }
        builder = builder.with_label(label);
    }

    // print
    builder.finish()
        .print(sources(vec![
            (file, content)
        ]))
        .unwrap();
}

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
            let range = result.binding.name.node().text_range();
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
                "message": format!("{}", result),
                "line": line_number,
                "column": start - line_offset + 1,
                "endColumn": usize::from(range.end()) - line_offset + 1,
            })
        }).collect::<serde_json::Value>(),
    });
    println!("{}", json);
}
