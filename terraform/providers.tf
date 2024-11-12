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
