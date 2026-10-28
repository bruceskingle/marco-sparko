@echo off
rem Brings both repos up to date on the latest release branch and downloads the installer for this PC from the latest
rem try or build, as release.sh sync and release.sh download do, see docs/Release.md. Run it from PowerShell, or a
rem Command Prompt, in any directory, e.g. .\scripts\sync-and-download.cmd
rem
rem It runs release.sh in Git Bash, which comes with Git for Windows, and needs gh installed and signed in.
setlocal

rem Git Bash, not the bash.exe of WSL, which is also on the PATH if WSL is installed. Where Git for Windows usually is,
rem or else from where git.exe is, e.g. C:\Program Files\Git\cmd\git.exe
set "BASH=%ProgramFiles%\Git\bin\bash.exe"
if exist "%BASH%" goto :found
for %%G in (git.exe) do set "GIT=%%~$PATH:G"
if not defined GIT goto :missing
for %%D in ("%GIT%\..\..") do set "BASH=%%~fD\bin\bash.exe"
if exist "%BASH%" goto :found
:missing
echo ERROR: Git Bash was not found, install Git for Windows
exit /b 1
:found

rem With / in place of \, which bash needs to find the repos from the script's path.
set "SCRIPT=%~dp0release.sh"
set "SCRIPT=%SCRIPT:\=/%"

rem Separately, so that download runs the version of release.sh which sync may have just pulled.
"%BASH%" "%SCRIPT%" sync || exit /b 1
"%BASH%" "%SCRIPT%" download || exit /b 1
