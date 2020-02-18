# Image resize via RUST vips bindings

## Requirements
To run this project locally you need to have locally installed libvips, libglib2.0 and libgtk2.0
and libvips with *at least version 8.8.4*.

## Usage
To start project just run `cargo run` - the project will be hosted on `localhost:3030/`

Service accepts POST requests on root route in next format:
```json
{
	"url": "your-url-to-image"
}
```

* Currently it converts only to .png format