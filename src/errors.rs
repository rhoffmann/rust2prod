pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}", e)?;
    let mut source = e.source();
    while let Some(e) = source {
        writeln!(f, "Caused by: {}", e)?;
        source = e.source();
    }
    Ok(())
}
