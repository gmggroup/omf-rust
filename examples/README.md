# Examples

## Running the Examples

For the examples with code, you can run them with:

```
cargo run --release --example <example-name>
```


## Available Examples

| Path | Code? | Description |
|---|---|---|
| `bunny/` | ✓ | Reads a mesh from a Wavefront OBJ file, writes it to an OMF file, then re-reads the OMF file. |
| `pyramid/` | ✓ | Writes a small pyramid mesh into an OMF file, then re-reads the file. |
| `bunny_blocks/` | ✓ | Reads an octree sub-blocked model from a CSV file, write it to OMF, reads that back, and writes some of the centroids to another CSV. |
