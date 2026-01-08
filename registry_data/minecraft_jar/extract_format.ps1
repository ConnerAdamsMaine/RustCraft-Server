#!/bin/env powershell

<#
Direct port of the provided bash script to PowerShell.
Preserves the original structure, including separate functions and manual exit code checking.
Uses Invoke-Expression to mimic the bash 'eval' behavior.
#>

param(
    [string]$MINECRAFT_VERSION = "1.21.7",
    [string]$JAVA_FILE = "",

    # This points to 1.21.7 server jar (at time of creation, the $MINECRAFT_VERSION used)
    [string]$SERVER_JAR_URL = "https://piston-data.mojang.com/v1/objects/05e4b48fbc01f0385adb74bcff9751d34552486c/server.jar"
)

[string]$script:MINECRAFT_VERSION = "1.21.7";
[string]$script:JAVA_FILE_FALLBACK = "server_$script:MINECRAFT_VERSION.jar";

[string]$script:SERVER_JAR_URL = "https://piston-data.mojang.com/v1/objects/05e4b48fbc01f0385adb74bcff9751d34552486c/server.jar";

# Use provided JAVA_FILE or default to fallback
[string]$script:JAVA_FILE = if ($JAVA_FILE) { $JAVA_FILE } else { $script:JAVA_FILE_FALLBACK };

function check_jar_exists {
    if (-not (Test-Path $script:JAVA_FILE)) {
        Write-Host "Downloading Minecraft server jar...";

        try {
            Invoke-WebRequest -Uri $script:SERVER_JAR_URL -OutFile $script:JAVA_FILE_FALLBACK;
        } catch {
            Write-Host "Failed to download the Minecraft server jar.";
            exit 1;
        }

        if (-not (Test-Path $script:JAVA_FILE_FALLBACK)) {
            Write-Host "Failed to download the Minecraft server jar.";
            exit 1;
        };

        # Switch to the downloaded fallback for the rest of the script
        $script:JAVA_FILE = $script:JAVA_FILE_FALLBACK;
    };
}
function dep_check {
    $check = Get-Command biome -ErrorAction SilentlyContinue;
    if (-not $check) {
        Write-Host "biomejs is not installed. Please install biomejs to use this script.";
        Write-Host "Visit 'https://biomejs.dev/' for installation instructions.";
        return 1;
    };
    return 0;
}

function extract_jar {
    $cmd = "java -DbundlerMainClass='net.minecraft.data.Main' -jar '$script:JAVA_FILE' -all";
    Invoke-Expression $cmd;
    return $LASTEXITCODE;
}

function biome_fmt {
    $cmd = "biome format --write --files-max-size=50000000 --json-formatter-enabled=true --json-formatter-indent-style=tab --json-formatter-indent-width=2 --json-formatter-line-ending=lf";
    Invoke-Expression $cmd;
    return $LASTEXITCODE;
}

function cleanup {
  $dirs=@(
    "generated",
    "libraries",
    "logs",
    "versions",
    "$script:JAVA_FILE_FALLBACK"
  );

  foreach ($dir in $dirs) {
    if (Test-Path $dir) {
      Remove-Item -Recurse -Force $dir;
    };
    continue;
  };

  return 0;
}

function Main {
    $clean_args = $args | Where-Object { $_ -eq "clean" -or $_ -eq "--clean" -or $_ -eq "-c" };
    if ($clean_args.Count -gt 0) {
      cleanup | Out-Null;
      Write-Host "Cleanup completed.";
      exit 0;
    };

    check_jar_exists;
    if ($LASTEXITCODE -ne 0) { exit 1; }

    dep_check;
    if ($LASTEXITCODE -ne 0) { exit 1; }

    extract_jar;
    if ($LASTEXITCODE -ne 0) { 
        Write-Host "An error occurred during extraction.";
        exit 1;
    }

    biome_fmt;
    if ($LASTEXITCODE -ne 0) { 
        Write-Host "An error occurred during formatting.";
        exit 1;
    }

    return 0
}

Main
