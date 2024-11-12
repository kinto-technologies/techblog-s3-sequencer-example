resource "aws_s3_bucket" "input" {
  bucket = "${var.resource_prefix}-input-bucket"
}

resource "aws_s3_bucket" "output" {
  bucket = "${var.resource_prefix}-output-bucket"
}
