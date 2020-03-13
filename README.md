# fontgen
A utility to generate font atlas images and metadata

## Why?
* FreeType generates really nice looking text.
* Nobody likes to depend on large, unsafe C libraries when programming in Rust! This utility uses FreeType to pre-generate a font atlas for your app so it remains C-dependency-free!

## How?
* `cargo install fontgen`
* Read the documentation of `fontgen-export` to create a spec file in JSON.
* Use fontgen to create an atlas image and associated metadata. (`fontgen ./path-to-spec.json -o out`)
* Add `fontgen-export = "*"` to your Cargo.toml to share the metadata types used by `fontgen`.
