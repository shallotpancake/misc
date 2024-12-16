#!/bin/bash

# Check if at least one argument is provided
if [ $# -lt 1 ]; then
    echo "Usage: $0 <alias_name> <alias>"
    exit 1
fi

echo "alias $1='$2'" >> ~/.bashrc

# Access arguments
echo "Script name: $0"
echo "Alias: $1"
echo "Command: $2"
