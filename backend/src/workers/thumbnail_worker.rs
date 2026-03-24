use std::io::Cursor;

use chrono::Datelike;
use futures_util::StreamExt;
use image::ImageFormat;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions},
    types::FieldTable,
    Connection, ConnectionProperties, ExchangeKind,
};
use serde::Deserialize;
use tokio::fs;

use crate::app_state::AppState;
use crate::models::File;

#[derive(Debug, Deserialize)]
struct ThumbnailJob {
    file_id: String,
}

pub async fn start(state: AppState, rabbitmq_url: String) -> anyhow::Result<()> {
    let conn = Connection::connect(&rabbitmq_url, ConnectionProperties::default()).await?;
    let ch = conn.create_channel().await?;

    ch.exchange_declare(
        "crm.jobs".into(),
        ExchangeKind::Topic,
        ExchangeDeclareOptions::default(),
        FieldTable::default(),
    )
    .await?;

    let q = ch
        .queue_declare(
            "thumbnail.generate".into(),
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    ch.queue_bind(
        q.name().as_str().into(),
        "crm.jobs".into(),
        "thumbnail.generate".into(),
        QueueBindOptions::default(),
        FieldTable::default(),
    )
    .await?;

    let consumer = ch
        .basic_consume(
            q.name().as_str().into(),
            "crm-thumb-worker".into(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    tokio::spawn(async move {
        futures_util::pin_mut!(consumer);
        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(d) => {
                    if let Err(e) = process_job(state.clone(), &d.data).await {
                        tracing::error!("thumbnail job failed: {}", e);
                    }
                    let _ = d.ack(BasicAckOptions::default()).await;
                }
                Err(e) => tracing::error!("thumbnail consumer error: {}", e),
            }
        }
    });

    Ok(())
}

async fn process_job(state: AppState, payload: &[u8]) -> anyhow::Result<()> {
    let job: ThumbnailJob = serde_json::from_slice(payload)?;
    let pool = state.pool();
    let file: File = sqlx::query_as("SELECT * FROM files WHERE id = ?1")
        .bind(&job.file_id)
        .fetch_one(pool)
        .await?;

    // Real image processing path (local path source).
    if file.file_path.starts_with("rustfs://") {
        anyhow::bail!("thumbnail generation from rustfs source is not implemented yet");
    }
    let data = fs::read(&file.file_path).await?;
    let img = image::load_from_memory(&data)?;
    let thumb = img.thumbnail(320, 320);
    let mut buf = Vec::new();
    thumb.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)?;

    let now = chrono::Utc::now();
    let tenant = state.config().default_tenant_id.clone();
    let owner = file.uploaded_by.unwrap_or_else(|| "system".to_string());
    let key = format!(
        "{}/{}/thumbnails/{:04}/{:02}/{:02}/{}.png",
        tenant,
        owner,
        now.year(),
        now.month(),
        now.day(),
        file.id
    );
    let thumbnail_uri = state.object_storage().put_object(&key, &buf, "image/png").await?;

    sqlx::query("UPDATE files SET thumbnail_path = ?1 WHERE id = ?2")
        .bind(&thumbnail_uri)
        .bind(&file.id)
        .execute(pool)
        .await?;

    Ok(())
}
