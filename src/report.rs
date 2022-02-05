use ariadne::{Config, Label, Report, ReportKind, sources};
use crate::dead_code::DeadCode;
use rnix::{TextSize, types::TypedNode};

// assumes results to be sorted by occurrence in file
pub fn print(file: String, content: &str, results: &[DeadCode]) {
    let first_result_range = results[0].binding.name.node().text_range();
    let mut builder = Report::build(
        ReportKind::Warning,
        file.clone(),
        first_result_range.start().into()
    )
        .with_config(
            Config::default()
                .with_compact(true)
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
        builder = builder.with_label(Label::new((file.clone(), start_char..end_char))
            .with_message(format!("{}", result))
            .with_color(result.scope.color())
            .with_order(order as i32)
        );
    }

    // print
    builder.finish()
        .print(sources(vec![
            (file, content)
        ]))
        .unwrap();
}
