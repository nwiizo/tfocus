resource "local_file" "example" {
  content  = "Hello, This is a test file created by Terraform!"
  filename = "${path.module}/example.txt"
}
