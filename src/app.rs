use crate::*;
use sqlx::SqlitePool;

pub struct App {
    database: SqlitePool,
    host: core::net::IpAddr,
    port: u16,
}
// use axum_login::{
//     AuthManagerLayerBuilder, login_required,
//     tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer},
// };

impl App {
    pub async fn new(
        database_url: impl AsRef<str>,
        host: core::net::IpAddr,
        port: u16,
    ) -> Result<Self> {
        let database = crate::database::connect(database_url)
            .await
            .change_context(Error)
            .attach_printable("Failed to connect to database")?;
        Ok(App {
            database,
            host,
            port,
        })
    }

    pub async fn serve(self) -> Result<()> {
        tracing::info!("Starting server at http://{}:{}", self.host, self.port);
        let app = routes::routes()
            .layer(axum::Extension(self.database.clone()))
            .layer(axum::middleware::from_fn(routes::handler_405))
            .fallback(routes::handler_404);
        let listener = tokio::net::TcpListener::bind((self.host, self.port))
            .await
            .change_context(Error)
            .attach_printable("Failed to create listener")?;
        axum::serve(listener, app.into_make_service())
            .await
            .change_context(Error)?;
        Ok(())
    }

    // pub async fn serve(self) -> Result<()> {
    //     use tower_sessions_sqlx_store::SqliteStore;
    //     let session_store = SqliteStore::new(self.database.clone());
    //     session_store.migrate().await.change_context(Error)?;
    //
    //     let deletion_task = tokio::task::spawn(
    //         session_store
    //             .clone()
    //             .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    //     );
    //
    //     // Generate a cryptographic key to sign the session cookie.
    //     let key = Key::generate();
    //
    //     let session_layer = SessionManagerLayer::new(session_store)
    //         .with_secure(false)
    //         .with_expiry(Expiry::OnInactivity(Duration::days(1)))
    //         .with_signed(key);
    //
    //     // Auth service.
    //     //
    //     // This combines the session layer with our backend to establish the auth
    //     // service which will provide the auth session as a request extension.
    //     let backend = Backend::new(self.db);
    //     let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
    //
    //     let app = protected::router()
    //         .route_layer(login_required!(Backend, login_url = "/login"))
    //         .merge(auth::router())
    //         .layer(MessagesManagerLayer)
    //         .layer(auth_layer);
    //
    //     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    //
    //     // Ensure we use a shutdown signal to abort the deletion task.
    //     axum::serve(listener, app.into_make_service())
    //         .with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle()))
    //         .await?;
    //
    //     deletion_task.await??;
    //
    //     Ok(())
    // }
}

// async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
//     let ctrl_c = async {
//         signal::ctrl_c()
//             .await
//             .expect("failed to install Ctrl+C handler");
//     };
//
//     #[cfg(unix)]
//     let terminate = async {
//         signal::unix::signal(signal::unix::SignalKind::terminate())
//             .expect("failed to install signal handler")
//             .recv()
//             .await;
//     };
//
//     #[cfg(not(unix))]
//     let terminate = std::future::pending::<()>();
//
//     tokio::select! {
//         _ = ctrl_c => { deletion_task_abort_handle.abort() },
//         _ = terminate => { deletion_task_abort_handle.abort() },
//     }
// }
