@echo off
setlocal

set "EXE=%~dp0gh-mdp.exe"
set "HTML_ICON=%~dp0file-code.ico"
if not exist "%EXE%" (
  echo gh-mdp.exe must be in the same directory as this script.
  exit /b 1
)
if not exist "%HTML_ICON%" (
  echo file-code.ico must be in the same directory as this script.
  exit /b 1
)

reg add "HKCU\Software\Classes\gh-mdp.Markdown" /ve /d "Markdown Document" /f >nul
reg add "HKCU\Software\Classes\gh-mdp.Markdown\DefaultIcon" /ve /d "\"%EXE%\",0" /f >nul
reg add "HKCU\Software\Classes\gh-mdp.Markdown\shell\open\command" /ve /d "\"%EXE%\" --app \"%%1\"" /f >nul
reg add "HKCU\Software\Classes\.md\OpenWithProgids" /v "gh-mdp.Markdown" /t REG_NONE /d "" /f >nul
reg add "HKCU\Software\Classes\gh-mdp.HTML" /ve /d "HTML Document" /f >nul
reg add "HKCU\Software\Classes\gh-mdp.HTML\DefaultIcon" /ve /d "\"%HTML_ICON%\",0" /f >nul
reg add "HKCU\Software\Classes\gh-mdp.HTML\shell\open\command" /ve /d "\"%EXE%\" --app \"%%1\"" /f >nul
reg add "HKCU\Software\Classes\.html\OpenWithProgids" /v "gh-mdp.HTML" /t REG_NONE /d "" /f >nul
reg add "HKCU\Software\Classes\.htm\OpenWithProgids" /v "gh-mdp.HTML" /t REG_NONE /d "" /f >nul
reg add "HKCU\Software\Classes\.xhtml\OpenWithProgids" /v "gh-mdp.HTML" /t REG_NONE /d "" /f >nul
reg add "HKCU\Software\Classes\.xht\OpenWithProgids" /v "gh-mdp.HTML" /t REG_NONE /d "" /f >nul
reg add "HKCU\Software\gh-mdp\Capabilities" /v "ApplicationName" /d "gh-mdp" /f >nul
reg add "HKCU\Software\gh-mdp\Capabilities" /v "ApplicationDescription" /d "Markdown and HTML preview" /f >nul
reg add "HKCU\Software\gh-mdp\Capabilities\FileAssociations" /v ".md" /d "gh-mdp.Markdown" /f >nul
reg add "HKCU\Software\gh-mdp\Capabilities\FileAssociations" /v ".html" /d "gh-mdp.HTML" /f >nul
reg add "HKCU\Software\gh-mdp\Capabilities\FileAssociations" /v ".htm" /d "gh-mdp.HTML" /f >nul
reg add "HKCU\Software\gh-mdp\Capabilities\FileAssociations" /v ".xhtml" /d "gh-mdp.HTML" /f >nul
reg add "HKCU\Software\gh-mdp\Capabilities\FileAssociations" /v ".xht" /d "gh-mdp.HTML" /f >nul
reg add "HKCU\Software\RegisteredApplications" /v "gh-mdp" /d "Software\gh-mdp\Capabilities" /f >nul

echo Registered gh-mdp for Markdown and HTML files.
echo Select gh-mdp once from "Open with" or Windows Default Apps.
