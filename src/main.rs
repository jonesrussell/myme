use anyhow::Result;

fn main() -> Result<()> {
    // Initialize core
    myme_core::init()?;

    // Create and initialize application
    let mut app = myme_core::App::new()?;
    app.initialize()?;

    tracing::info!("MyMe application started");

    println!("MyMe - Personal Productivity & Dev Hub");
    println!("Architecture initialized successfully!");
    println!("\nConfiguration:");
    println!("  Config directory: {}", app.config().config_dir.display());

    // Graceful shutdown
    app.shutdown()?;

    Ok(())
}
