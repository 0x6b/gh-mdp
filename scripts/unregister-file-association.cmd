@echo off
setlocal

reg delete "HKCU\Software\Classes\.md\OpenWithProgids" /v "gh-mdp.Markdown" /f >nul 2>&1
reg delete "HKCU\Software\Classes\gh-mdp.Markdown" /f >nul 2>&1
reg delete "HKCU\Software\RegisteredApplications" /v "gh-mdp" /f >nul 2>&1
reg delete "HKCU\Software\gh-mdp" /f >nul 2>&1

echo Removed the gh-mdp Markdown file association.
