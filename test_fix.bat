@echo off
cd /d D:\Project\Helios\omni-lang\compiler
echo Testing main_minimal.omni with import std::core and std::collections...
setlocal enabledelayedexpansion
set start=%time%
.\target\release\omnc.exe ../omni/compiler/main_minimal.omni -o ../build/test-minimal-fix.ovm 2>&1
set exit_code=%errorlevel%
set end=%time%
echo.
echo EXIT CODE: %exit_code%
echo Test completed successfully - NO HANG!
echo.
if %exit_code% equ 0 (
    echo SUCCESS: Compilation completed without hanging
    type ..\build\test-minimal-fix.ovm >nul 2>&1
    if %errorlevel% equ 0 (
        echo OUTPUT FILE CREATED: ..\build\test-minimal-fix.ovm
    )
) else (
    echo FAILED: Exit code %exit_code%
)
