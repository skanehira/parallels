#!/bin/bash
# E2E testing script: simulates server log output

RESET="\033[0m"
BOLD="\033[1m"
DIM="\033[2m"

FG_GREEN="\033[32m"
FG_YELLOW="\033[33m"
FG_RED="\033[31m"
FG_CYAN="\033[36m"
FG_MAGENTA="\033[35m"
FG_WHITE="\033[37m"

METHODS=("GET" "POST" "PUT" "DELETE" "PATCH")
PATHS=("/api/users" "/api/products" "/api/orders" "/health" "/api/auth/login" "/api/search" "/static/main.js" "/static/style.css")
STATUS_CODES=(200 200 200 200 201 204 301 400 401 403 404 500)
USER_AGENTS=("Mozilla/5.0" "curl/7.88" "PostmanRuntime" "Python-requests")

random_ip() {
    echo "$((RANDOM % 256)).$((RANDOM % 256)).$((RANDOM % 256)).$((RANDOM % 256))"
}

random_element() {
    local arr=("$@")
    echo "${arr[$RANDOM % ${#arr[@]}]}"
}

format_status() {
    local code=$1
    if ((code >= 200 && code < 300)); then
        echo -e "${FG_GREEN}${code}${RESET}"
    elif ((code >= 300 && code < 400)); then
        echo -e "${FG_CYAN}${code}${RESET}"
    elif ((code >= 400 && code < 500)); then
        echo -e "${FG_YELLOW}${code}${RESET}"
    else
        echo -e "${BOLD}${FG_RED}${code}${RESET}"
    fi
}

format_method() {
    local method=$1
    case $method in
        GET)    echo -e "${FG_GREEN}${method}${RESET}" ;;
        POST)   echo -e "${FG_YELLOW}${method}${RESET}" ;;
        PUT)    echo -e "${FG_CYAN}${method}${RESET}" ;;
        DELETE) echo -e "${FG_RED}${method}${RESET}" ;;
        PATCH)  echo -e "${FG_MAGENTA}${method}${RESET}" ;;
    esac
}

output_access_log() {
    local ip=$(random_ip)
    local method=$(random_element "${METHODS[@]}")
    local path=$(random_element "${PATHS[@]}")
    local status=$(random_element "${STATUS_CODES[@]}")
    local response_time=$((RANDOM % 500 + 1))
    local timestamp=$(date +"%d/%b/%Y:%H:%M:%S %z")

    echo -e "${DIM}${ip}${RESET} - [${timestamp}] $(format_method $method) ${FG_WHITE}${path}${RESET} $(format_status $status) ${DIM}${response_time}ms${RESET}"
}

output_error_log() {
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    local errors=(
        "Connection refused to database"
        "Redis connection timeout"
        "Rate limit exceeded for IP"
        "Invalid JWT token"
        "Request body too large"
    )
    local error=$(random_element "${errors[@]}")
    echo -e "${BOLD}${FG_RED}[ERROR]${RESET} ${DIM}${timestamp}${RESET} ${error}" >&2
}

output_info_log() {
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    local infos=(
        "New connection established"
        "Cache hit for key: user_session"
        "Background job completed"
        "Health check passed"
        "Metrics exported successfully"
    )
    local info=$(random_element "${infos[@]}")
    echo -e "${FG_CYAN}[INFO]${RESET} ${DIM}${timestamp}${RESET} ${info}"
}

echo -e "${BOLD}${FG_MAGENTA}=== Server Log Simulator ===${RESET}"
echo -e "${DIM}Simulating HTTP server logs...${RESET}"
echo ""

while true; do
    case $((RANDOM % 10)) in
        0|1|2|3|4|5|6)
            output_access_log
            ;;
        7|8)
            output_info_log
            ;;
        9)
            output_error_log
            ;;
    esac

    sleep_time=$(awk "BEGIN {printf \"%.2f\", 0.2 + rand() * 0.8}")
    sleep "$sleep_time"
done
