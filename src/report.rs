use ariadne::{Label, Report, ReportKind, sources};
use crate::dead_code::DeadCode;
use rnix::types::TypedNode;

pub fn print(file: String, content: &str, results: &[DeadCode]) {
    let first_result_range = results[0].binding.name.node().text_range();
    let mut builder = Report::build(
        ReportKind::Warning,
        file.clone(),
        first_result_range.start().into()
    )
        .with_message("Unused declarations were found.");

    // reverse order to avoid overlapping lanes
    let mut order = results.len();
    for result in results {
        let range = result.binding.name.node().text_range();
        builder = builder.with_label(Label::new((file.clone(), range.start().into()..range.end().into()))
            .with_message(format!("{}", result))
            .with_color(result.scope.color())
            .with_order(order as i32)
        );

        order -= 1;
    }

    builder.finish()
        .print(sources(vec![
            (file, content)
        ]))
        .unwrap()
}
