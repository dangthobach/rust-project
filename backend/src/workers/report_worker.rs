use chrono::Datelike;
use futures_util::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, ExchangeDeclareOptions, QueueBindOptions, QueueDeclareOptions},
    types::FieldTable,
    Connection, ConnectionProperties, ExchangeKind,
};
use serde::Deserialize;

use crate::app_state::AppState;
use crate::models::{Client, Task, User};

#[derive(Debug, Deserialize)]
struct ReportJob {
    requested_by: String,
    format: Option<String>,
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
            "report.export".into(),
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    ch.queue_bind(
        q.name().as_str().into(),
        "crm.jobs".into(),
        "report.export.*".into(),
        QueueBindOptions::default(),
        FieldTable::default(),
    )
    .await?;

    let consumer = ch
        .basic_consume(
            q.name().as_str().into(),
            "crm-report-worker".into(),
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    tokio::spawn(async move {
        futures_util::pin_mut!(consumer);
        while let Some(delivery) = consumer.next().await {
            match delivery {
                Ok(d) => {
                    let rk = d.routing_key.as_str().to_string();
                    if let Err(e) = process_job(state.clone(), &rk, &d.data).await {
                        tracing::error!("report job failed: {}", e);
                    }
                    let _ = d.ack(BasicAckOptions::default()).await;
                }
                Err(e) => tracing::error!("report consumer error: {}", e),
            }
        }
    });

    Ok(())
}

async fn process_job(state: AppState, routing_key: &str, payload: &[u8]) -> anyhow::Result<()> {
    let job: ReportJob = serde_json::from_slice(payload)?;
    let format = job.format.unwrap_or_else(|| "csv".to_string()).to_ascii_lowercase();
    let pool = state.pool();

    let (content_type, bytes, kind) = match routing_key {
        "report.export.clients" => {
            let rows: Vec<Client> = sqlx::query_as("SELECT * FROM clients ORDER BY created_at DESC")
                .fetch_all(pool)
                .await?;
            if format == "json" {
                ("application/json".to_string(), serde_json::to_vec_pretty(&rows)?, "clients")
            } else {
                let mut csv = String::from("id,name,email,phone,company,status,created_at\n");
                for r in rows {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{}\n",
                        r.id,
                        r.name.replace(',', " "),
                        r.email.unwrap_or_default().replace(',', " "),
                        r.phone.unwrap_or_default().replace(',', " "),
                        r.company.unwrap_or_default().replace(',', " "),
                        r.status,
                        r.created_at
                    ));
                }
                ("text/csv".to_string(), csv.into_bytes(), "clients")
            }
        }
        "report.export.tasks" => {
            let rows: Vec<Task> = sqlx::query_as("SELECT * FROM tasks ORDER BY created_at DESC")
                .fetch_all(pool)
                .await?;
            if format == "json" {
                ("application/json".to_string(), serde_json::to_vec_pretty(&rows)?, "tasks")
            } else {
                let mut csv = String::from("id,title,status,priority,created_at\n");
                for r in rows {
                    csv.push_str(&format!(
                        "{},{},{},{},{}\n",
                        r.id,
                        r.title.replace(',', " "),
                        r.status,
                        r.priority,
                        r.created_at
                    ));
                }
                ("text/csv".to_string(), csv.into_bytes(), "tasks")
            }
        }
        "report.export.users" => {
            let rows: Vec<User> = sqlx::query_as("SELECT * FROM users ORDER BY created_at DESC")
                .fetch_all(pool)
                .await?;
            if format == "json" {
                ("application/json".to_string(), serde_json::to_vec_pretty(&rows)?, "users")
            } else {
                let mut csv = String::from("id,email,full_name,role,created_at\n");
                for r in rows {
                    csv.push_str(&format!(
                        "{},{},{},{},{}\n",
                        r.id,
                        r.email.replace(',', " "),
                        r.full_name.replace(',', " "),
                        r.role,
                        r.created_at
                    ));
                }
                ("text/csv".to_string(), csv.into_bytes(), "users")
            }
        }
        _ => {
            let summary = serde_json::json!({
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "clients": sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM clients").fetch_one(pool).await?,
                "tasks": sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tasks").fetch_one(pool).await?,
                "users": sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users").fetch_one(pool).await?,
            });
            ("application/json".to_string(), serde_json::to_vec_pretty(&summary)?, "dashboard")
        }
    };

    let now = chrono::Utc::now();
    let tenant = state.config().default_tenant_id.clone();
    let ext = if content_type == "application/json" { "json" } else { "csv" };
    let key = format!(
        "{}/{}/reports/{:04}/{:02}/{:02}/{}_{}.{}",
        tenant,
        job.requested_by,
        now.year(),
        now.month(),
        now.day(),
        kind,
        now.timestamp(),
        ext
    );
    let uri = state.object_storage().put_object(&key, &bytes, &content_type).await?;
    let signed = state.object_storage().presign_get_url(&uri, 3600).await?;
    let link = signed.unwrap_or(uri);

    let email_job = serde_json::json!({
        "job": "email.send_link",
        "to_user_id": job.requested_by,
        "subject": format!("Your {} report is ready", kind),
        "download_url": link
    });
    let _ = state
        .rabbitmq_publisher
        .publish("crm.jobs", "email.send_link", &email_job.to_string())
        .await;

    Ok(())
}
