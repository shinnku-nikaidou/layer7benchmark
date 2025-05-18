
use log::{debug, info};
use tokio::{signal, sync::watch};

pub async fn handle_shutdown_signals(shutdown_tx: watch::Sender<bool>) {
    let ctrl_c = async {
        #[allow(clippy::expect_used)]
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        #[allow(clippy::expect_used)]
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install terminate signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    #[cfg(windows)]
    #[allow(clippy::expect_used)]
    let ctrl_shutdown = async {
        signal::windows::ctrl_shutdown()
            .expect("Failed to install Ctrl+Shutdown handler")
            .recv()
            .await;
    };

    #[cfg(not(windows))]
    let ctrl_shutdown = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal");
        }
        _ = terminate => {
            debug!("Received terminate signal");
        }
        _ = ctrl_shutdown => {
            debug!("Received shutdown signal");
        }
    }

    // Send shutdown signal to all tasks
    let _ = shutdown_tx.send(true);
}
