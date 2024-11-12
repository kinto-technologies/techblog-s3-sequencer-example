use crate::image_task::ImageTask;
use aws_sdk_dynamodb::{error::SdkError, operation::put_item::PutItemError, types::AttributeValue};
use aws_sdk_s3::primitives::ByteStream;
use lambda_runtime::{tracing::warn, Error};
use serde_dynamo::aws_sdk_dynamodb_1::{from_item, to_item};
use std::{thread, time::Duration, time::Instant};

pub struct S3Lock {
    dynamodb_client: aws_sdk_dynamodb::Client,
    table_name: String,

    s3_client: aws_sdk_s3::Client,
    input_bucket_name: String,
    input_object_key: String,
    output_bucket_name: String,
    output_object_key: String,
}

impl S3Lock {
    pub async fn new(task: &ImageTask) -> Result<Self, Error> {
        let table_name = std::env::var("DYNAMODB_TABLE_NAME").unwrap();
        let output_bucket_name = std::env::var("OUTPUT_BUCKET_NAME").unwrap();
        let require_lock_timeout = Duration::from_secs(
            std::env::var("REQUIRE_LOCK_TIMEOUT")
                .unwrap_or_else(|_| "30".to_string())
                .parse::<u64>()
                .unwrap(),
        );
        let interval_retry_time = Duration::from_secs(
            std::env::var("RETRY_INTERVAL")
                .unwrap_or_else(|_| "2".to_string())
                .parse::<u64>()
                .unwrap(),
        );

        let config = aws_config::load_defaults(aws_config::BehaviorVersion::v2024_03_28()).await;
        let s3_client = aws_sdk_s3::Client::new(&config);
        let dynamodb_client = aws_sdk_dynamodb::Client::new(&config);

        // ロックを取得する
        // 実行時間を計測する
        let start = Instant::now();
        loop {
            // 30秒以上ロックが取れない場合はタイムアウトする
            if start.elapsed() > require_lock_timeout {
                return Err("Failed to acquire lock, timeout".into());
            }

            // 強力な読み取り整合性を利用してDynamoDBからシーケンサを取得する
            let item = dynamodb_client
                .get_item()
                .table_name(table_name.clone())
                .key("id", AttributeValue::S(task.object_key.clone()))
                .consistent_read(true)
                .send()
                .await?;

            // 取得したアイテムが存在する場合はシーケンサを比較する
            if let Some(item) = item.item {
                let item: ImageTask = from_item(item)?;
                if task.sequencer <= item.sequencer {
                    // 自分自身が古いシーケンサの場合は処理する必要がないのでスキップする
                    return Err("Old sequencer".into());
                }

                // 自分自身が新しいシーケンサの場合は他の処理が終わるまで待機する
                if item.processing {
                    warn!(
                        "Waiting for other process to finish task, retrying, remaining time: {:?}",
                        require_lock_timeout - start.elapsed()
                    );
                    thread::sleep(interval_retry_time);
                    continue;
                }
            }

            // DynamoDBに条件付き書き込みでロックを取得する
            // その際にレコードが存在していたらprocessingフラグがfalseの場合のみ書き込む
            let resp = dynamodb_client
                .put_item()
                .table_name(table_name.clone())
                .set_item(Some(to_item(task).unwrap()))
                .condition_expression("attribute_not_exists(id) OR processing = :false")
                .expression_attribute_values(":false", AttributeValue::Bool(false))
                .send()
                .await;

            // 取得できたらループを抜け処理を続行する
            // 取得できなかった場合はロックが取れるまでリトライを繰り返す
            match resp {
                Ok(_) => break,
                Err(SdkError::ServiceError(e)) => match e.err() {
                    PutItemError::ConditionalCheckFailedException(_) => {
                        warn!(
                            "Failed to acquire lock, retrying, remaining time: {:?}",
                            require_lock_timeout - start.elapsed()
                        );
                        thread::sleep(Duration::from_secs(2));
                        continue;
                    }
                    _ => return Err(format!("{:?}", e).into()),
                },
                Err(e) => return Err(e.into()),
            }
        }

        Ok(Self {
            dynamodb_client,
            output_bucket_name,
            s3_client,
            table_name,
            input_bucket_name: task.bucket_name.clone(),
            input_object_key: task.object_key.clone(),
            output_object_key: task.object_key.clone(),
        })
    }

    pub async fn read_input_object(&self) -> Result<Vec<u8>, Error> {
        // S3オブジェクトを取得する
        let object = self
            .s3_client
            .get_object()
            .bucket(&self.input_bucket_name)
            .key(&self.input_object_key)
            .send()
            .await?;
        let body = object.body.collect().await?.to_vec();
        Ok(body)
    }

    pub async fn write_output_object(&self, buf: Vec<u8>) -> Result<(), Error> {
        // S3オブジェクトを保存する
        let byte_stream = ByteStream::from(buf);
        self.s3_client
            .put_object()
            .bucket(&self.output_bucket_name)
            .key(&self.output_object_key)
            .body(byte_stream)
            .send()
            .await?;
        Ok(())
    }

    pub async fn delete_output_object(&self) -> Result<(), Error> {
        // S3オブジェクトを削除する
        self.s3_client
            .delete_object()
            .bucket(&self.output_bucket_name)
            .key(&self.output_object_key)
            .send()
            .await?;
        Ok(())
    }

    pub async fn free(self) -> Result<(), Error> {
        // DynamoDBのロックを解放する
        // processingフラグのみを更新する
        self.dynamodb_client
            .update_item()
            .table_name(self.table_name)
            .key("id", AttributeValue::S(self.input_object_key))
            .update_expression("SET processing = :false")
            .expression_attribute_values(":false", AttributeValue::Bool(false))
            .send()
            .await?;
        Ok(())
    }
}
