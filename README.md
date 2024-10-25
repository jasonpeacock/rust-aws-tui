# AWS Lambda Logs Viewer

A terminal-based user interface (TUI) application for viewing AWS Lambda function logs across multiple profiles and regions.

## Features

- 🔍 Browse and filter Lambda functions across AWS profiles
- ⚡ Quick access to recent logs with predefined time ranges
- 📅 Custom date range selection for detailed log analysis
- 🔎 Real-time log filtering and search
- 💨 Fast navigation with keyboard shortcuts
- 📦 Function list caching for improved performance

## Prerequisites

- Rust (latest stable version)
- AWS credentials configured in `~/.aws/credentials`
- AWS config with profiles in `~/.aws/config`

## Installation

1. Clone the repository:

```shell
git clone https://github.com/resola-ai/rust-aws-tui
cd rust-aws-tui
```

2. Build and install:

```
cargo install --path .
```

## Usage

- Update config.toml with your AWS profiles and regions.

### Basic Navigation

- Use `↑`/`↓` or `k`/`j` to navigate through lists
- `Tab` to switch between panels
- `Enter` to select/confirm
- `Esc` to go back/cancel
- `q` to quit the application

### Function Selection

1. Select an AWS profile from the list
2. Choose a region
3. Browse or search through the Lambda functions list
4. Press `Enter` to view logs for the selected function

### Time Range Selection

- Choose from predefined ranges:
  - Last 15 minutes
  - Last hour
  - Last 3 hours
  - Last 24 hours
- Or select "Custom Range" to specify exact dates and times

### Log Viewing

- Use `↑`/`↓` to scroll through logs
- Type to search/filter logs in real-time
- `Ctrl+C` to copy selected log entry
- `f` to toggle full-screen mode

## Configuration

### AWS Credentials

Ensure your AWS credentials are properly configured:

# ~/.aws/credentials
[default]
aws_access_key_id = YOUR_ACCESS_KEY
aws_secret_access_key = YOUR_SECRET_KEY

[other-profile]
aws_access_key_id = OTHER_ACCESS_KEY
aws_secret_access_key = OTHER_SECRET_KEY

# ~/.aws/config
[profile default]
region = us-west-2

[profile other-profile]
region = eu-west-1

### Cache Configuration

The application caches function lists to improve performance. Cache files are stored in:
- Linux/macOS: `~/.cache/aws-lambda-logs-viewer/`
- Windows: `%LOCALAPPDATA%\aws-lambda-logs-viewer\`

To clear the cache, delete the cache directory or use the `--clear-cache` flag when launching the application.

## Troubleshooting

### Common Issues

1. **No AWS profiles found**
   - Verify AWS credentials file exists
   - Check file permissions
   - Ensure proper file format

2. **Cannot fetch Lambda functions**
   - Verify AWS credentials are valid
   - Check IAM permissions
   - Ensure network connectivity

3. **Slow function loading**
   - Consider using the cache feature
   - Check network latency
   - Verify AWS API rate limits

### Required IAM Permissions

Minimum IAM policy required:

{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "lambda:ListFunctions",
                "logs:GetLogEvents",
                "logs:FilterLogEvents",
                "logs:DescribeLogStreams",
                "logs:DescribeLogGroups"
            ],
            "Resource": "*"
        }
    ]
}

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
