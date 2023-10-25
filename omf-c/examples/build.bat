@REM Run 'cargo build --release --all' before running this script.
@echo off
if not exist "build" mkdir build
cd build || ( exit /b 1 )
set CMAKE_BUILD_TYPE=Release
cmake .. || ( exit /b 1 )
cmake --build . --config Release || ( exit /b 1 )
set path=..\..\..\target\release;%path%

.\Release\pyramid.exe > pyramid.txt || exit /b
git diff --ignore-cr-at-eol --no-index --exit-code pyramid.txt ..\pyramid_output.txt || exit /b

.\Release\metadata.exe > metadata.txt || exit /b
git diff --ignore-cr-at-eol --no-index --exit-code metadata.txt ..\metadata_output.txt || exit /b

.\Release\geometries.exe > geometries.txt || exit /b
git diff --ignore-cr-at-eol --no-index --exit-code geometries.txt ..\geometries_output.txt || exit /b

.\Release\attributes.exe > attributes.txt || exit /b
git diff --ignore-cr-at-eol --no-index --exit-code attributes.txt ..\attributes_output.txt || exit /b

.\Release\textures.exe > textures.txt || exit /b
git diff --ignore-cr-at-eol --no-index --exit-code textures.txt ..\textures_output.txt || exit /b

echo All OK
