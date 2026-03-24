pub mod report_worker;
pub mod thumbnail_worker;

use crate::app_state::AppState;

pub async fn start_workers(state: AppState, rabbitmq_url: String) -> anyhow::Result<()> {
    if rabbitmq_url.trim().is_empty() {
        tracing::warn!("RabbitMQ URL empty, skip worker startup");
        return Ok(());
    }

    thumbnail_worker::start(state.clone(), rabbitmq_url.clone()).await?;
    report_worker::start(state, rabbitmq_url).await?;
    Ok(())
}
