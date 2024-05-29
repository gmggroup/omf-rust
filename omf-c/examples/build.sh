#!/usr/bin/sh
# Run 'cargo build --all --release' before running this script.
set -e
mkdir -p build
cd build
cmake ..
cmake --build .
export PATH="../../../target/release;$PATH"
./pyramid > pyramid.txt
echo "Run pyramid"
diff -u --strip-trailing-cr pyramid.txt ../pyramid_output.txt
./metadata > metadata.txt
echo "Run metadata"
diff -u --strip-trailing-cr metadata.txt ../metadata_output.txt
./geometries > geometries.txt
echo "Run geometries"
diff -u --strip-trailing-cr geometries.txt ../geometries_output.txt
./attributes > attributes.txt
echo "Run attributes"
diff -u --strip-trailing-cr attributes.txt ../attributes_output.txt
./textures > textures.txt
echo "Run textures"
diff -u --strip-trailing-cr textures.txt ../textures_output.txt
echo "All OK"
