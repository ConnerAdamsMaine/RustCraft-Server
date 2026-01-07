#!/bin/env bash
#

JAVA_FILE=""
MINECRAFT_VERSION="1.21.7"
JAVA_FILE_FALLBACK="server_$MINECRAFT_VERSION.jar"

# This points to 1.21.7 server jar (at time of creation, the $MINECRAFT_VERSION used)
SERVER_JAR_URL="https://piston-data.mojang.com/v1/objects/05e4b48fbc01f0385adb74bcff9751d34552486c/server.jar"

function check_jar_exists() {
  if [[ ! -f "$JAVA_FILE" ]]; then
    printf "Downloading Minecraft server jar...\n"
    curl -o "$JAVA_FILE_FALLBACK" -L "$SERVER_JAR_URL"
    if [[ $? -ne 0 ]]; then
      printf "Failed to download the Minecraft server jar.\n"
      exit 1
    fi
    JAVA_FILE="$JAVA_FILE_FALLBACK"
  fi
}

function dep_check() {
  check=$(command -v biome)
  if [[ ! $check ]]; then
    printf "biome is not installed. Please install biome to use this script.\n"
    return 1
  fi
}

function extract_jar() {
  cmd="java -DbundlerMainClass='net.minecraft.data.Main' -jar $JAVA_FILE -all"
  eval "$cmd"
  return $?
}

function biome_fmt() {
  cmd="biome format --write --files-max-size=50000000 --json-formatter-enabled=true --json-formatter-indent-style=tab --json-formatter-indent-width=2 --json-formatter-line-ending=lf"
  eval "$cmd"
  return $?
}

function cleanup() {
  dirs=(
    "generated"
    "libraries"
    "logs"
    "versions"
    "$JAVA_FILE_FALLBACK"
  )

  for dir in "${dirs[@]}"; do
    if [[ -e $dir ]]; then
      rm -rf "$dir"
    fi
  done

}

function help_msg() {
  # use a 'heredoc'

  msg=$(
    cat <<'EOF'
Usage: extract_format.sh [options] [path_to_minecraft_server_jar]

Parameters:
  path_to_minecraft_server_jar          Path to the Minecraft server jar file. If not provided, the script will download the default server jar for version 1.21.7.
  c | clean | cleanup                   Cleans up generated files and directories created during extraction and formatting.

Returns:
  0   Success
  1   Failure (e.g., missing dependencies, download failure, extraction/formatting errors)

EOF
  )

  printf "%s" "$msg"
  return 0
}

function main() {
  # if the first param is either "c" "clean" or "cleanup", run cleanup and exit
  clean_args=(
    "c"
    "clean"
    "cleanup"
    # permuations via "-X" and "--X" also
  )
  if [[ " ${clean_args[*]} " == *" $1 "* || " ${clean_args[*]} " == *" --$1 "* ]]; then
    cleanup
    exit 0
  fi
  if [[ $1 == "-h" || $1 == "--help" ]]; then
    help_msg
    exit 0
  fi

  JAVA_FILE="$1"

  check_jar_exists || exit 1
  dep_check || exit 1

  extract_jar
  biome_fmt

  if [[ $? -ne 0 ]]; then
    printf "An error occurred during extraction or formatting.\n"
    exit 1
  fi
  return 0
}

main "$@"
