use crate::image_task::ImageTask;
use aws_lambda_events::event::s3::S3Event;
use lambda_runtime::{tracing::info, Error, LambdaEvent};

pub async fn function_handler(event: LambdaEvent<S3Event>) -> Result<(), Error> {
    // S3イベントをImageTaskに変換する
    let tasks: Vec<_> = event
        .payload
        .records
        .into_iter()
        .map(ImageTask::try_from)
        .collect::<Result<Vec<_>, _>>()?;

    // futures::future::join_allで実行するタスクを作成する
    let execute_tasks = tasks.iter().map(|task| task.execute());

    // join_allで全てのタスクを実行・待機する
    // 実行結果をretに格納する
    let ret = futures::future::join_all(execute_tasks).await;

    // 実行結果をログに出力する
    for (t, r) in tasks.iter().zip(&ret) {
        info!("object_key: {}, Result: {:?}", t.object_key, r);
    }

    // エラーがある場合はエラーを返す
    if ret.iter().any(|r| r.is_err()) {
        return Err("Some tasks failed".into());
    }

    // 正常終了
    Ok(())
}
