pub fn init() -> anyhow::Result<()> {
    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        // Add blanket level filter -
        .level(log::LevelFilter::Error)
        // - and per-module overrides
        .level_for("docsore", log::LevelFilter::Debug)
        // Output to stdout, files, and other Dispatch configurations
        .chain(std::io::stdout())
        // Apply globally
        .apply()?;
    Ok(())
}
