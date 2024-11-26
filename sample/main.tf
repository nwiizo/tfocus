# main.tf
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = "ap-northeast-1"
}

# VPCとサブネットの定義
resource "aws_vpc" "main" {
  cidr_block = "10.0.0.0/16"
  
  tags = {
    Name = "main-vpc"
  }
}

resource "aws_subnet" "public" {
  vpc_id     = aws_vpc.main.id
  cidr_block = "10.0.1.0/24"
  
  tags = {
    Name = "public-subnet"
  }
}

# network.tf
resource "aws_internet_gateway" "main" {
  vpc_id = aws_vpc.main.id
  
  tags = {
    Name = "main-igw"
  }
}

resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id
  
  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.main.id
  }
  
  tags = {
    Name = "public-rt"
  }
}

# compute.tf
resource "aws_instance" "web" {
  ami           = "ami-0d3bbfd074edd7acb"  # Amazon Linux 2023
  instance_type = "t3.micro"
  subnet_id     = aws_subnet.public.id
  
  tags = {
    Name = "web-server"
  }
}

resource "aws_security_group" "web" {
  name        = "web-sg"
  description = "Security group for web server"
  vpc_id      = aws_vpc.main.id
  
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }
  
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# storage.tf
resource "aws_s3_bucket" "logs" {
  bucket = "my-app-logs-bucket"
  
  tags = {
    Environment = "dev"
  }
}

# modules.tf
module "vpc" {
  source = "./modules/vpc"
  
  vpc_cidr = "172.16.0.0/16"
  environment = "staging"
}

module "ec2" {
  source = "./modules/ec2"
  
  instance_type = "t3.small"
  subnet_id     = module.vpc.public_subnet_id
}

# ./modules/vpc/main.tf
variable "vpc_cidr" {
  type = string
}

variable "environment" {
  type = string
}

resource "aws_vpc" "module_vpc" {
  cidr_block = var.vpc_cidr
  
  tags = {
    Name        = "${var.environment}-vpc"
    Environment = var.environment
  }
}

resource "aws_subnet" "module_public" {
  vpc_id     = aws_vpc.module_vpc.id
  cidr_block = cidrsubnet(var.vpc_cidr, 8, 1)
  
  tags = {
    Name = "${var.environment}-public-subnet"
  }
}

output "public_subnet_id" {
  value = aws_subnet.module_public.id
}

# ./modules/ec2/main.tf
variable "instance_type" {
  type = string
}

variable "subnet_id" {
  type = string
}

resource "aws_instance" "module_instance" {
  ami           = "ami-0d3bbfd074edd7acb"
  instance_type = var.instance_type
  subnet_id     = var.subnet_id
  
  tags = {
    Name = "module-instance"
  }
}
