use std::{env::args, fs};

mod usage;
mod dead_code;


fn main() {
    for path in args().skip(1) {
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Error reading file: {}", err);
                continue;
            }
        };

        let ast = rnix::parse(&content);
        let mut failed = false;
        for error in ast.errors() {
            eprintln!("Parse error: {}", error);
            failed = true;
        }
        if failed {
            continue;
        }

        let results = crate::dead_code::find_dead_code(ast.node());
        if results.len() == 0 {
            return;
        }

        let mut lines = Vec::new();
        let mut offset = 0;
        while let Some(next) = content[offset..].find('\n') {
            let line = &content[offset..offset + next];
            lines.push((offset, line));
            offset += next + 1;
        }
        lines.push((offset, &content[offset..]));

        let mut last_line = 0;
        let mut result_by_lines = Vec::new();
        for result in results {
            let range = result.node.text_range();
            let start = usize::from(range.start());
            let line_number = lines.iter().filter(|(offset, _)| *offset <= start).count();
            if line_number != last_line {
                last_line = line_number;
                result_by_lines.push((line_number, Vec::new()));
            }
            let result_by_lines_len = result_by_lines.len();
            let line_results = &mut result_by_lines[result_by_lines_len - 1].1;
            line_results.push(result);
        }
        for (line_number, results) in result_by_lines.iter_mut() {
            // file location
            println!("{}:{}:", path, line_number);
            // line
            println!("> {}", lines[*line_number - 1].1);
            results.sort_unstable_by_key(|result| result.node.text_range().start());

            // underscores ^^^^^^^^^
            let line_start = lines[*line_number - 1].0;
            let mut pos = line_start;
            print!("> ");
            for result in results.iter() {
                let range = result.node.text_range();
                let start = usize::from(range.start());
                let end = usize::from(range.end());
                print!("{0: <1$}{2:^<3$}", "", start - pos, "", end - start);
                pos = end;
            }
            println!("");

            let mut bars = String::new();
            let mut pos = line_start;
            for result in results.iter() {
                let range = result.node.text_range();
                let start = usize::from(range.start());
                bars = format!("{}{1: <2$}|", bars, "", start - pos);
                pos = start + 1;
            }
            println!("> {}", bars);

            // messages
            for result in results.iter().rev() {
                let range = result.node.text_range();
                let start = usize::from(range.start());
                println!("> {}Unused {}", &bars[..start - line_start], result);
            }
        }
    }
}
