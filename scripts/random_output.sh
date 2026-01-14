#!/bin/bash
# E2E testing script: outputs random text with ANSI escape sequences

# ANSI color codes
RESET="\033[0m"
BOLD="\033[1m"
DIM="\033[2m"
ITALIC="\033[3m"
UNDERLINE="\033[4m"
BLINK="\033[5m"
REVERSE="\033[7m"

# Foreground colors
FG_BLACK="\033[30m"
FG_RED="\033[31m"
FG_GREEN="\033[32m"
FG_YELLOW="\033[33m"
FG_BLUE="\033[34m"
FG_MAGENTA="\033[35m"
FG_CYAN="\033[36m"
FG_WHITE="\033[37m"

# Bright foreground colors
FG_BRIGHT_RED="\033[91m"
FG_BRIGHT_GREEN="\033[92m"
FG_BRIGHT_YELLOW="\033[93m"
FG_BRIGHT_BLUE="\033[94m"
FG_BRIGHT_MAGENTA="\033[95m"
FG_BRIGHT_CYAN="\033[96m"

# Background colors
BG_RED="\033[41m"
BG_GREEN="\033[42m"
BG_YELLOW="\033[43m"
BG_BLUE="\033[44m"

# Arrays for random selection
COLORS=("$FG_RED" "$FG_GREEN" "$FG_YELLOW" "$FG_BLUE" "$FG_MAGENTA" "$FG_CYAN" "$FG_BRIGHT_RED" "$FG_BRIGHT_GREEN" "$FG_BRIGHT_YELLOW" "$FG_BRIGHT_BLUE" "$FG_BRIGHT_MAGENTA" "$FG_BRIGHT_CYAN")
STYLES=("$BOLD" "$DIM" "$ITALIC" "$UNDERLINE" "" "" "")
MESSAGES=(
    "Processing data..."
    "Compiling module"
    "Running tests"
    "Building artifacts"
    "Checking dependencies"
    "Analyzing code"
    "Optimizing output"
    "Generating report"
    "Validating input"
    "Synchronizing state"
)
WARNINGS=(
    "Deprecated API usage detected"
    "Performance could be improved"
    "Consider updating dependency"
    "Unused variable found"
    "Missing documentation"
)
ERRORS=(
    "Connection timeout"
    "Invalid configuration"
    "Resource not found"
    "Permission denied"
    "Unexpected token"
)
SUCCESS=(
    "Task completed successfully"
    "All tests passed"
    "Build finished"
    "Deployment successful"
    "Validation passed"
)

# Get random element from array
random_element() {
    local arr=("$@")
    echo "${arr[$RANDOM % ${#arr[@]}]}"
}

# Output functions
output_info() {
    local color=$(random_element "${COLORS[@]}")
    local style=$(random_element "${STYLES[@]}")
    local msg=$(random_element "${MESSAGES[@]}")
    local timestamp=$(date +"%H:%M:%S")
    echo -e "${FG_CYAN}[$timestamp]${RESET} ${style}${color}${msg}${RESET}"
}

output_success() {
    local msg=$(random_element "${SUCCESS[@]}")
    local timestamp=$(date +"%H:%M:%S")
    echo -e "${FG_CYAN}[$timestamp]${RESET} ${BOLD}${FG_GREEN}✓ ${msg}${RESET}"
}

output_warning() {
    local msg=$(random_element "${WARNINGS[@]}")
    local timestamp=$(date +"%H:%M:%S")
    echo -e "${FG_CYAN}[$timestamp]${RESET} ${BOLD}${FG_YELLOW}⚠ WARNING: ${msg}${RESET}" >&2
}

output_error() {
    local msg=$(random_element "${ERRORS[@]}")
    local timestamp=$(date +"%H:%M:%S")
    echo -e "${FG_CYAN}[$timestamp]${RESET} ${BOLD}${FG_RED}✗ ERROR: ${msg}${RESET}" >&2
}

output_progress() {
    local percent=$((RANDOM % 101))
    local bar_width=20
    local filled=$((percent * bar_width / 100))
    local empty=$((bar_width - filled))
    local bar="${FG_GREEN}$(printf '█%.0s' $(seq 1 $filled 2>/dev/null || echo))${FG_WHITE}$(printf '░%.0s' $(seq 1 $empty 2>/dev/null || echo))${RESET}"
    echo -e "${FG_CYAN}Progress:${RESET} [${bar}] ${BOLD}${percent}%${RESET}"
}

output_fancy() {
    local timestamp=$(date +"%H:%M:%S")
    echo -e "${BG_BLUE}${FG_WHITE}${BOLD} NOTICE ${RESET} ${FG_BRIGHT_CYAN}${UNDERLINE}Special announcement${RESET} at ${timestamp}"
}

output_multicolor() {
    echo -e "${FG_RED}R${FG_YELLOW}a${FG_GREEN}i${FG_CYAN}n${FG_BLUE}b${FG_MAGENTA}o${FG_RED}w${RESET} ${BOLD}text output${RESET}"
}

# Main loop
echo -e "${BOLD}${FG_BRIGHT_CYAN}=== Random Output Generator Started ===${RESET}"
echo -e "${DIM}Press Ctrl+C to stop${RESET}"
echo ""

counter=0
while true; do
    counter=$((counter + 1))

    # Random output type selection
    case $((RANDOM % 10)) in
        0|1|2|3)
            output_info
            ;;
        4|5)
            output_success
            ;;
        6)
            output_warning
            ;;
        7)
            output_error
            ;;
        8)
            output_progress
            ;;
        9)
            if ((RANDOM % 2 == 0)); then
                output_fancy
            else
                output_multicolor
            fi
            ;;
    esac

    # Random sleep between 0.3 and 1.5 seconds
    sleep_time=$(awk "BEGIN {printf \"%.1f\", 0.3 + rand() * 1.2}")
    sleep "$sleep_time"

    # Occasionally output a batch of messages
    if ((counter % 10 == 0)); then
        echo -e "\n${DIM}--- Batch output (${counter} messages sent) ---${RESET}"
        for i in {1..3}; do
            output_info
        done
        echo ""
    fi
done
