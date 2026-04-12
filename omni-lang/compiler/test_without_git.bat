@echo off
REM Remove Git from PATH temporarily
set PATH=%PATH:C:\Program Files\Git\usr\bin;=%

cd /d D:\Project\Helios\omni-lang\compiler
cargo test --lib
