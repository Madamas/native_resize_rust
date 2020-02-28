# Image resize via native Rust image processing capabilities

## Usage
To start project just run `cargo run` - the project will be hosted on `localhost:3030/`

Service accepts GET requests on root route in next format:
```curl
http://localhost:3030/thumbnail?url=url-to-image&width=180
```

* Currently it converts only to .png format