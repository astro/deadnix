use rnix::types::TypedNode;
use crate::dead_code::DeadCode;

pub struct LineReport {
    line_start: usize,
    line_number: usize,
    line: String,
    results: Vec<DeadCode>,
}

pub struct Report {
    file_path: String,
    line_reports: Vec<LineReport>,
}

impl Report {
    /// Create a report grouped by line
    ///
    /// Assumes results are pre-sorted by order of appearance.
    pub fn new(file_path: String, content: &str, results: Vec<DeadCode>) -> Self {
        let mut lines = Vec::new();
        let mut offset = 0;
        while let Some(next) = content[offset..].find('\n') {
            let line = &content[offset..offset + next];
            lines.push((offset, line));
            offset += next + 1;
        }
        lines.push((offset, &content[offset..]));

        let mut last_line = 0;
        let mut line_reports = Vec::new();
        for result in results {
            let range = result.binding.name.node().text_range();
            let start = usize::from(range.start());
            let line_number = lines.iter().filter(|(offset, _)| *offset <= start).count();
            if line_number != last_line {
                last_line = line_number;
                line_reports.push(LineReport {
                    line_start: lines[line_number - 1].0,
                    line_number,
                    line: lines[line_number - 1].1.to_string(),
                    results: Vec::new(),
                });
            }
            let line_results = &mut line_reports.last_mut().unwrap().results;
            line_results.push(result);
        }

        Report {
            file_path,
            line_reports,
        }
    }

    pub fn print(&self) {
        for LineReport { line_start, line_number, line, results } in self.line_reports.iter() {
            // file location
            println!("{}:{}:", self.file_path, line_number);
            // line
            println!("> {}", line);

            // underscores ^^^^^^^^^
            let mut pos = *line_start;
            print!("> ");
            for result in results.iter() {
                let range = result.binding.name.node().text_range();
                let start = usize::from(range.start());
                let end = usize::from(range.end());
                print!("{0: <1$}{2:^<3$}", "", start - pos, "", end - start);
                pos = end;
            }
            println!("");

            let mut bars = String::new();
            let mut pos = *line_start;
            for result in results.iter() {
                let range = result.binding.name.node().text_range();
                let start = usize::from(range.start());
                bars = format!("{}{1: <2$}|", bars, "", start - pos);
                pos = start + 1;
            }
            println!("> {}", bars);

            // messages
            for result in results.iter().rev() {
                let range = result.binding.name.node().text_range();
                let start = usize::from(range.start());
                println!("> {}{}", &bars[..start - line_start], result);
            }
        }
    }
}
