# Image resize via RUST vips bindings

## Requirements
To run this project locally you need to have locally installed libvips, libglib2.0 and libgtk2.0
and libvips with *at least version 8.8.4*.

## Usage
To start project just run `cargo run` - the project will be hosted on `localhost:3030/`

Service accepts GET requests on root route in next format:
```curl
http://localhost:3030/thumbnail?url=url-to-image&width=180
```

* Currently it converts only to .png format