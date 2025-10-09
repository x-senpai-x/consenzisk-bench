#!/bin/bash

OPERATION=$1

LOG_FILE="logs/execution_$OPERATION.log"
OUTPUT_FILE="summaries/summary_$OPERATION.md"

# Table Header
echo '| Operation | Test Case | Execution Time |' > $OUTPUT_FILE
echo '|-----------|-----------|----------------|' >> $OUTPUT_FILE

awk '
BEGIN {
    op = "";
    test_case = "";
    execution_time = 0;
}

/\[.*\] Test case:/ {
    op = $4;
    gsub(/[\[\]]/, "", op)
    test_case = $NF;
}

/process_rom\(\) steps=/ {
    # Extract duration from the line: process_rom() steps=... duration=X.XXXX ...
    for (i = 1; i <= NF; i++) {
        if ($i ~ /^duration=/) {
            execution_time = substr($i, 10);  # Remove "duration=" prefix
            break;
        }
    }
}

/----- Cycle Tracker End -----/ {
    printf "%s | %s | %s |\n", op, test_case, execution_time >> "'$OUTPUT_FILE'"

    # Re-initialize for next log
    op = "";
    test_case = "";
    execution_time = 0;
}
' $LOG_FILE
