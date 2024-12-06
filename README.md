# kinto-technologies/techblog-s3-sequencer-example

This repository contains the code for the blog post [データ競合を気にしながらS3イベントを処理してみた 〜Rust・Lambda・DynamoDBを添えて〜](https://blog.kinto-technologies.com/posts/2024-12-06-データ競合を気にしながらS3イベントを処理してみた/)

# Requirements
  - Rust
    - https://www.rust-lang.org/tools/install
  - Cargo Lambda
    - https://www.cargo-lambda.info/guide/installation.html
  - Terraform
    - https://developer.hashicorp.com/terraform/tutorials/aws-get-started/install-cli

# Getting Started

- Clone the repository
  ```bash
  git clone https://github.com/kinto-technologies/techblog-s3-sequencer-example.git
  ```
- Change directory to the repository
  ```bash
  cd techblog-s3-sequencer-example/terraform
  ```

- Edit the `providers.tf` file and update the `region`, `profile` and `bucket` values with your desired values.
  ```terraform
  provider "aws" {
    region  = "** INPUT HERE **"
    profile = "** INPUT HERE **"
  }

  terraform {
    required_providers {
      aws = {
        source  = "hashicorp/aws"
        version = "~> 5.75.1"
      }
    }

    backend "s3" {
      bucket  = "** INPUT HERE **"
      key     = "terraform.tfstate"
      region  = "** INPUT HERE **"
      profile = "** INPUT HERE **"
    }
  }
  ```
- Edit the `variables.tf` file and update the `resource_prefix` value with your desired prefix.
  ```terraform
  variable "rust_src_path" {
    default = "../lambda"
  }

  variable "lambda_zip_local_path" {
    default = "lambda.zip"
  }

  variable "resource_prefix" {
    default = "** INPUT HERE **"
  }
  ```
- Run the following commands to create the resources
  ```bash
  terraform init
  terraform apply
  ```
