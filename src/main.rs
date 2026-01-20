use anyhow::Result;

fn main() -> Result<()> {
    // Initialize core
    myme_core::init()?;

    // Create and initialize application
    let mut app = myme_core::App::new()?;
    app.initialize()?;

    tracing::info!("MyMe application started");
    tracing::info!("Todo API URL: {}", app.config().services.todo_api_url);

    // Note: In Phase 1, we're setting up the architecture
    // The Qt/QML application will need a separate C++ main that loads the QML
    // For now, we'll just demonstrate the Rust components work

    println!("MyMe - Personal Productivity & Dev Hub");
    println!("Architecture initialized successfully!");
    println!("\nConfiguration:");
    println!("  Todo API: {}", app.config().services.todo_api_url);
    println!("  Config directory: {}", app.config().config_dir.display());

    // Graceful shutdown
    app.shutdown()?;

    Ok(())
}
