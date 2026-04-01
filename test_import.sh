#!/bin/bash
cd /d/Project/Helios/omni-lang/compiler
echo "Testing fixed omnc with import..."
timeout 5 ./target/release/omnc /d/Project/Helios/test_import_hang.omni 2>&1 &
wait $!
exit_code=$?
echo "Exit code: $exit_code"
if [ $exit_code -eq 124 ]; then
    echo "TIMEOUT - Process hung (exit 124)"
else
    echo "Process completed normally"
fi
