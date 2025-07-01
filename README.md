# LLM Router Service

LLM Router is a high-performance proxy service written in Rust that provides a unified interface for multiple Language Learning Model (LLM) backends. It dynamically discovers available models and routes requests to appropriate backends based on the requested model.

## Features

- Dynamic model discovery from multiple backends
- Automatic routing of requests based on model availability
- Health check endpoint
- Compatible with OpenAI-style API endpoints
- Configurable refresh intervals for model updates
- Docker support
- High performance thanks to Rust and Axum

## Requirements

- Rust 1.75 or higher
- Docker (optional, for containerized deployment)
- One or more LLM backends with OpenAI compatible API

## Quick Start

### Running Locally

1. Clone the repository:
2. Create a `config.yaml` file in the project root:
3. Build and run the service: `bash cargo build --release cargo run --release
` The service will be available at `http://localhost:8080`

### Running with Docker

1. Build the Docker image: `bash docker build -t llm-router .`
2. Run the container:
```bash
docker run -d -p 8080:8080 -v $(pwd)/config.yaml:/app/config.yaml -e RUST_LOG=info --name llm-router llm-router
```
## API Endpoints

- `GET /healthz` - Health check endpoint
- `GET /v1/models` - List available models
- `POST /v1/chat/completions` - Chat completion endpoint
- `POST /v1/completions` - Text completion endpoint

## Configuration

The service is configured via `config.yaml` file structure:
```yaml
refresh_interval: 300
backends:
- name: "local-llama"
  url: "http://localhost:8000"
- name: "ollama-server"
  url: "http://ollama:11434"
```

## Performance

The service is built with performance in mind:
- Async I/O with Tokio
- Efficient request routing
- Connection pooling
- Minimal overhead

## License

This project is licensed under the MIT License — see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Support

For support, please open an issue in the GitHub repository.
