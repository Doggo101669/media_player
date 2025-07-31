#!/bin/bash
ls -la ../bin
echo "This is the contents of the bin folder, are you sure you want to remove them?"
read -n 1 key
if [ "$key" = "y" ]; then
    echo "Continuing program."
    rm ../bin/* -r
    cargo build -r
    cargo build -r --target x86_64-pc-windows-gnu
    cp ./target/release/player ../bin
    cp ./target/x86_64-pc-windows-gnu/release/player.exe ../bin
    cp ./res ../bin -r
else
    echo "Exiting program..."
    exit
fi
