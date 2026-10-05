use anyhow::{Context, Result};
use futures_util::StreamExt;
use tauri::{
    AppHandle,
    async_runtime::{JoinHandle, spawn},
};
use zbus::{Connection, Proxy};

pub(crate) struct WakeMonitor(JoinHandle<()>);

impl WakeMonitor {
    #[expect(
        clippy::unnecessary_wraps,
        reason = "Common constructor across desktop platforms"
    )]
    pub(crate) fn new(app: &AppHandle) -> Result<Self> {
        let app = app.clone();
        Ok(Self(spawn(async move {
            if let Err(error) = watch_resume(&app).await {
                tracing::warn!("System wake notifications unavailable: {error:#}");
            }
        })))
    }
}

impl Drop for WakeMonitor {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn watch_resume(app: &AppHandle) -> Result<()> {
    let connection = Connection::system()
        .await
        .context("Cannot connect to system D-Bus")?;
    let proxy = Proxy::new(
        &connection,
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
    )
    .await?;
    let mut signals = proxy.receive_signal("PrepareForSleep").await?;
    while let Some(signal) = signals.next().await {
        let (sleeping,): (bool,) = signal.body().deserialize()?;
        if !sleeping {
            super::notify_resume(app);
        }
    }
    anyhow::bail!("logind wake notification stream ended")
}
