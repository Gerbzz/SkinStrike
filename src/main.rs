use std::path::Path;

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let result = match args.as_slice() {
        [command, input, output] if command == "inspect" => {
            skinstrike::inspect_png(Path::new(input)).and_then(|report| {
                skinstrike::write_new_report(&report, Path::new(output))?;
                Ok(format!("{}x{} PNG inspected", report.width, report.height))
            })
        }
        [command, base, project] if command == "new" => {
            skinstrike::document::new_document(Path::new(base), Path::new(project))
                .and_then(|document| {
                    skinstrike::document::save_document(&document, Path::new(project), false)?;
                    Ok(format!("new finish document: {}", Path::new(project).display()))
                })
        }
        [command, project, name, red, green, blue, opacity] if command == "add-color" => {
            let parse = |value: &std::ffi::OsString| value.to_string_lossy().parse::<u8>()
                .map_err(|_| "color channels and opacity must be 0-255".to_string());
            (|| {
                let mut document = skinstrike::document::load_document(Path::new(project))?;
                document.add_color_layer(&name.to_string_lossy(),
                    [parse(red)?, parse(green)?, parse(blue)?], parse(opacity)?)?;
                skinstrike::document::save_document(&document, Path::new(project), true)?;
                Ok(format!("{} color layers", document.layers.len()))
            })()
        }
        [command, project, output] if command == "export" => {
            skinstrike::document::load_document(Path::new(project)).and_then(|document| {
                skinstrike::document::export_png(&document, Path::new(project), Path::new(output))?;
                Ok(format!("exported {}", Path::new(output).display()))
            })
        }
        _ => Err("usage: skinstrike inspect <finish.png> <new-report.json> | new <base.png> <new-project.json> | add-color <project.json> <name> <r> <g> <b> <opacity> | export <project.json> <new.png>".into()),
    };
    match result {
        Ok(message) => println!("{message}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
