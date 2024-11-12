use crate::{lock::S3Lock, s3_sequencer::S3Sequencer};
use aws_lambda_events::event::s3::S3EventRecord;
use lambda_runtime::Error;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum TaskType {
    Grayscale,
    Delete,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageTask {
    pub bucket_name: String,
    #[serde(rename = "id")]
    pub object_key: String,
    pub sequencer: S3Sequencer,
    pub task_type: TaskType,
    pub processing: bool,
}

impl ImageTask {
    pub async fn execute(&self) -> Result<(), Error> {
        // ロックを取得する
        let lock = S3Lock::new(self).await?;

        // タスクの種類に応じて処理を行う
        match self.task_type {
            TaskType::Grayscale => {
                // 画像をグレースケールに変換し、出力バケットに保存する
                let body = lock.read_input_object().await?;
                let format = image::ImageFormat::from_path(&self.object_key)?;
                let img = image::load_from_memory_with_format(&body, format)?;
                let img = img.grayscale();
                let mut buf = Vec::new();
                img.write_to(&mut Cursor::new(&mut buf), format)?;
                lock.write_output_object(buf).await?;
            }
            // 画像を出力用バケットから削除する
            TaskType::Delete => lock.delete_output_object().await?,
        }

        // ロックを解放する
        lock.free().await?;

        Ok(())
    }
}

impl TryFrom<S3EventRecord> for ImageTask {
    type Error = anyhow::Error;

    fn try_from(record: S3EventRecord) -> Result<Self, Self::Error> {
        let record_name = record
            .event_name
            .ok_or(anyhow::anyhow!("Event name is not found."))?;

        let task_type = if record_name.starts_with("ObjectCreated") {
            TaskType::Grayscale
        } else if record_name.starts_with("ObjectRemoved") {
            TaskType::Delete
        } else {
            return Err(anyhow::anyhow!("Unknown event name: {}", record_name));
        };

        let bucket_name = record
            .s3
            .bucket
            .name
            .ok_or(anyhow::anyhow!("Bucket name is not found."))?;

        let object_key = record
            .s3
            .object
            .key
            .ok_or(anyhow::anyhow!("Object key is not found."))?;

        let sequencer = record
            .s3
            .object
            .sequencer
            .ok_or(anyhow::anyhow!("Sequencer is not found."))?;

        let sequencer = S3Sequencer::new(&bucket_name, &object_key, &sequencer);

        Ok(Self {
            bucket_name,
            object_key,
            sequencer,
            task_type,
            processing: true,
        })
    }
}
