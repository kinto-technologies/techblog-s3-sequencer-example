resource "aws_dynamodb_table" "basic-dynamodb-table" {
  name         = "${var.resource_prefix}-dynamodb-table"
  billing_mode = "PAY_PER_REQUEST"

  hash_key = "id"

  attribute {
    name = "id"
    type = "S"
  }

  tags = {
    Name = "${var.resource_prefix}-dynamodb-table"
  }
}
