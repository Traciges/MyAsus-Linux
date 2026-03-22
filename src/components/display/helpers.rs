/// Führt qdbus-qt6 mit Fallback auf qdbus aus.
/// Gibt Ok(()) oder Err(String) zurück.
pub(crate) async fn qdbus_ausfuehren(args: Vec<String>) -> Result<(), String> {
    let result = tokio::task::spawn_blocking(move || {
        let status = std::process::Command::new("qdbus-qt6")
            .args(&args)
            .status();
        match status {
            Ok(s) => Ok(("qdbus-qt6", s)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Fallback auf qdbus
                std::process::Command::new("qdbus")
                    .args(&args)
                    .status()
                    .map(|s| ("qdbus", s))
            }
            Err(e) => Err(e),
        }
    })
    .await;

    match result {
        Ok(Ok((_, status))) if status.success() => Ok(()),
        Ok(Ok((cmd, status))) => Err(format!(
            "{cmd} fehlgeschlagen mit Exit-Code: {}",
            status.code().unwrap_or(-1)
        )),
        Ok(Err(e)) => Err(format!("qdbus starten fehlgeschlagen: {e}")),
        Err(e) => Err(format!("spawn_blocking fehlgeschlagen: {e}")),
    }
}

/// Führt kwriteconfig6 mit den gegebenen Argumenten aus.
/// Gibt Ok(()) oder Err(String) zurück.
pub(crate) async fn kwriteconfig_ausfuehren(args: &[&str]) -> Result<(), String> {
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let result = tokio::task::spawn_blocking(move || {
        std::process::Command::new("kwriteconfig6")
            .args(&args)
            .status()
    })
    .await;

    match result {
        Ok(Ok(status)) if status.success() => Ok(()),
        Ok(Ok(status)) => Err(format!(
            "kwriteconfig6 fehlgeschlagen mit Exit-Code: {}",
            status.code().unwrap_or(-1)
        )),
        Ok(Err(e)) => Err(format!("kwriteconfig6 starten fehlgeschlagen: {e}")),
        Err(e) => Err(format!("spawn_blocking fehlgeschlagen: {e}")),
    }
}
