resource "aws_cloudwatch_log_group" "this" {
  name              = "/aws/lambda/${var.resource_prefix}-lambda"
  retention_in_days = 7
}
