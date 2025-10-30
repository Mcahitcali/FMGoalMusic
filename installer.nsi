; FM Goal Musics - Windows Installer Script
; Requires NSIS (Nullsoft Scriptable Install System) to compile

!define APPNAME "FM Goal Musics"
!define VERSION "1.0.0"
!define PUBLISHER "FM Goal Musics"
!define DESCRIPTION "Goal celebration music player for Football Manager"

; Modern UI settings
!include "MUI2.nsh"

; General settings
Name "${APPNAME}"
OutFile "FM-Goal-Musics-Installer.exe"
InstallDir "$PROGRAMFILES\${APPNAME}"
InstallDirRegKey HKLM "Software\${APPNAME}" "InstallPath"
RequestExecutionLevel admin

; Interface settings
!define MUI_ABORTWARNING
!define MUI_ICON "assets\icon.ico"
!define MUI_UNICON "assets\icon.ico"

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE.txt"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

; Languages
!insertmacro MUI_LANGUAGE "English"

; Installer sections
Section "MainSection" SEC01
    SetOutPath "$INSTDIR"
    
    ; Create installation directory
    CreateDirectory "$INSTDIR"
    
    ; Display installation progress
    DetailPrint "Installing ${APPNAME}..."
    
    ; Check if Rust is installed
    DetailPrint "Checking for Rust installation..."
    nsExec::Exec 'cmd /c "rustc --version"'
    Pop $0
    StrCmp $0 0 rust_found rust_not_found
    
rust_not_found:
    DetailPrint "Rust not found. Installing Rust..."
    
    ; Download Rust installer
    DetailPrint "Downloading Rust installer..."
    inetc::get "https://win.rustup.rs/x86_64" "$TEMP\rustup-init.exe" /END
    Pop $0
    StrCmp $0 "OK" rust_download_ok rust_download_failed
    
rust_download_failed:
    MessageBox MB_OK "Failed to download Rust installer. Please check your internet connection."
    Abort
    
rust_download_ok:
    ; Install Rust silently
    DetailPrint "Installing Rust (this may take several minutes)..."
    nsExec::ExecToLog '"$TEMP\rustup-init.exe" -y --default-toolchain stable'
    Pop $0
    StrCmp $0 0 rust_install_ok rust_install_failed
    
rust_install_failed:
    MessageBox MB_OK "Rust installation failed. Please try running the installer as administrator."
    Abort
    
rust_install_ok:
    ; Clean up Rust installer
    Delete "$TEMP\rustup-init.exe"
    
rust_found:
    DetailPrint "Rust is installed and ready."
    
    ; Copy source files
    DetailPrint "Copying application files..."
    File /r "src\"
    File "Cargo.toml"
    File "Cargo.lock"
    File "build_windows.bat"
    File /r "assets\"
    
    ; Build the application
    DetailPrint "Building ${APPNAME} (this may take 10-15 minutes)..."
    nsExec::ExecToLog 'cmd /c "cd $INSTDIR && build_windows.bat"'
    Pop $0
    StrCmp $0 0 build_ok build_failed
    
build_failed:
    MessageBox MB_OK "Build failed. Please check the build log for details."
    Abort
    
build_ok:
    DetailPrint "Build completed successfully!"
    
    ; Create shortcuts
    CreateDirectory "$SMPROGRAMS\${APPNAME}"
    CreateShortCut "$SMPROGRAMS\${APPNAME}\${APPNAME}.lnk" "$INSTDIR\build\windows\fm-goal-musics-gui.exe"
    CreateShortCut "$DESKTOP\${APPNAME}.lnk" "$INSTDIR\build\windows\fm-goal-musics-gui.exe"
    
    ; Create uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"
    
    ; Registry entries
    WriteRegStr HKLM "Software\${APPNAME}" "InstallPath" "$INSTDIR"
    WriteRegStr HKLM "Software\${APPNAME}" "Version" "${VERSION}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayName" "${APPNAME}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "UninstallString" "$INSTDIR\Uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayVersion" "${VERSION}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "Publisher" "${PUBLISHER}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "Description" "${DESCRIPTION}"
    
    DetailPrint "Installation completed successfully!"
SectionEnd

; Uninstaller section
Section "Uninstall"
    ; Remove shortcuts
    Delete "$SMPROGRAMS\${APPNAME}\${APPNAME}.lnk"
    Delete "$DESKTOP\${APPNAME}.lnk"
    RMDir "$SMPROGRAMS\${APPNAME}"
    
    ; Remove files and directories
    RMDir /r "$INSTDIR"
    
    ; Remove registry entries
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}"
    DeleteRegKey HKLM "Software\${APPNAME}"
    
    MessageBox MB_OK "${APPNAME} has been uninstalled successfully."
SectionEnd

; Functions
Function .onInit
    ; Check if running on Windows
    IfWinExists $0
    FunctionEnd
    
    ; Display welcome message
    MessageBox MB_OK "Welcome to ${APPNAME} Installer$\r$\n$\r$\nThis installer will:$\r$\n• Install Rust (if needed)$\r$\n• Build the application$\r$\n• Create desktop shortcuts$\r$\n$\r$\nThe installation may take 15-20 minutes."
FunctionEnd

Function .onInstSuccess
    MessageBox MB_OK "${APPNAME} has been installed successfully!$\r$\n$\r$\nYou can now run the application from:$\r$\n• Desktop shortcut$\r$\n• Start Menu$\r$\n• Or directly from: $INSTDIR\build\windows\fm-goal-musics-gui.exe"
FunctionEnd
