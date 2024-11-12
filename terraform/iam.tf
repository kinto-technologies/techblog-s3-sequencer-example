resource "aws_iam_role_policy" "allow_access_s3" {
  name = "${var.resource_prefix}-lambda-s3-access"
  role = aws_iam_role.lambda.name
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = [
          "s3:GetObject",
        ]
        Effect   = "Allow"
        Resource = "${aws_s3_bucket.input.arn}/*"
      },
      {
        Action = [
          "s3:ListBucket",
        ]
        Effect   = "Allow"
        Resource = aws_s3_bucket.input.arn
      },
      {
        Action = [
          "s3:PutObject",
          "s3:DeleteObject",
        ]
        Effect   = "Allow"
        Resource = "${aws_s3_bucket.output.arn}/*"
      },
      {
        Action = [
          "s3:ListBucket",
        ]
        Effect   = "Allow"
        Resource = aws_s3_bucket.output.arn
      },
      {
        Action = [
          "dynamodb:DeleteItem",
          "dynamodb:GetItem",
          "dynamodb:PutItem",
          "dynamodb:Query",
          "dynamodb:Scan",
          "dynamodb:UpdateItem",
        ]
        Effect   = "Allow"
        Resource = aws_dynamodb_table.basic-dynamodb-table.arn
      },
    ]
  })
}
resource "aws_iam_role" "lambda" {
  name = "${var.resource_prefix}-lambda-role"
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
      },
    ]
  })
}

resource "aws_iam_role_policy" "lambda_execution" {
  name = "${var.resource_prefix}-lambda-execution-policy"
  role = aws_iam_role.lambda.name
  policy = jsonencode(
    {
      Statement = [
        {
          Effect   = "Allow"
          Action   = "logs:CreateLogGroup"
          Resource = "${aws_cloudwatch_log_group.this.arn}:*",
        },
        {
          Action = [
            "logs:CreateLogStream",
            "logs:PutLogEvents",
          ]
          Effect   = "Allow"
          Resource = "${aws_cloudwatch_log_group.this.arn}:*",
        },
      ]
      Version = "2012-10-17"
    }
  )
}
