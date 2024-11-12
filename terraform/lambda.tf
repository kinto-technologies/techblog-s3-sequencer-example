resource "aws_lambda_function" "this" {
  architectures    = ["arm64"]
  function_name    = "${var.resource_prefix}-lambda"
  role             = aws_iam_role.lambda.arn
  memory_size      = "1024"
  package_type     = "Zip"
  timeout          = 60
  source_code_hash = data.archive_file.this.output_base64sha256
  runtime          = "provided.al2023"
  handler          = "bootstrap"
  publish          = false
  skip_destroy     = false
  filename         = data.archive_file.this.output_path
  ephemeral_storage {
    size = 512
  }
  tracing_config {
    mode = "PassThrough"
  }
  environment {
    variables = {
      DYNAMODB_TABLE_NAME = aws_dynamodb_table.basic-dynamodb-table.name
      OUTPUT_BUCKET_NAME  = aws_s3_bucket.output.bucket
    }
  }
}

resource "null_resource" "rust_build" {
  triggers = {
    code_diff = sha512(join("", [
      for file in fileset(var.rust_src_path, "**/*.rs")
      : filesha256("${var.rust_src_path}/${file}")
    ]))
  }

  provisioner "local-exec" {
    working_dir = var.rust_src_path
    command     = "cargo lambda build --release --arm64"
  }
}

data "archive_file" "this" {
  type        = "zip"
  source_file = "${var.rust_src_path}/target/lambda/s3-sequencer-example/bootstrap"
  output_path = var.lambda_zip_local_path

  depends_on = [
    null_resource.rust_build
  ]
}

resource "aws_lambda_permission" "allow_bucket" {
  statement_id  = "${var.resource_prefix}-lambda-s3-permission"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.this.arn
  principal     = "s3.amazonaws.com"
  source_arn    = aws_s3_bucket.input.arn
}

resource "aws_s3_bucket_notification" "this" {
  bucket = aws_s3_bucket.input.bucket

  lambda_function {
    id                  = "${var.resource_prefix}-lambda"
    lambda_function_arn = aws_lambda_function.this.arn
    events              = ["s3:ObjectCreated:*", "s3:ObjectRemoved:Delete"]
  }

  depends_on = [aws_lambda_permission.allow_bucket]
}
