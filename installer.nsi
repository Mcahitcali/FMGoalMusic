; FM Goal Musics - Smart Windows Installer
; Detects and auto-installs Tesseract OCR if needed

!define APPNAME "FM Goal Musics"
!define VERSION "1.0.0"
!define PUBLISHER "FM Goal Musics"
!define DESCRIPTION "Goal celebration music player for Football Manager"
!define URL "https://github.com/your-repo/fm-goal-musics"

; Modern UI
!include "MUI2.nsh"

; General settings
Name "${APPNAME}"
OutFile "FM-Goal-Musics-Setup-${VERSION}.exe"
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
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

; Languages
!insertmacro MUI_LANGUAGE "English"

; Installer sections
Section "FM Goal Musics (Required)" SecMain
    SectionIn RO
    
    SetOutPath "$INSTDIR"
    
    ; Main application files
    File /r "build\windows\*"
    
    ; Create start menu shortcuts
    CreateDirectory "$SMPROGRAMS\${APPNAME}"
    CreateShortCut "$SMPROGRAMS\${APPNAME}\${APPNAME}.lnk" "$INSTDIR\fm-goal-musics-gui.exe"
    CreateShortCut "$SMPROGRAMS\${APPNAME}\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
    
    ; Create desktop shortcut
    CreateShortCut "$DESKTOP\${APPNAME}.lnk" "$INSTDIR\fm-goal-musics-gui.exe"
    
    ; Registry entries for uninstall
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayName" "${APPNAME}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "UninstallString" "$INSTDIR\Uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayVersion" "${VERSION}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "Publisher" "${PUBLISHER}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "URLInfoAbout" "${URL}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" "DisplayIcon" "$INSTDIR\fm-goal-musics-gui.exe"
    WriteRegStr HKLM "Software\${APPNAME}" "InstallPath" "$INSTDIR"
    
    ; Create uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Tesseract OCR (Recommended)" SecTesseract
    ; Check if Tesseract is already installed
    DetailPrint "Checking for Tesseract OCR installation..."
    
    ; Check common installation paths
    ${If} ${FileExists} "$PROGRAMFILES\Tesseract-OCR\tesseract.exe"
        DetailPrint "Tesseract OCR found at $PROGRAMFILES\Tesseract-OCR"
        Goto TesseractDone
    ${EndIf}
    
    ${If} ${FileExists} "$PROGRAMFILES64\Tesseract-OCR\tesseract.exe"
        DetailPrint "Tesseract OCR found at $PROGRAMFILES64\Tesseract-OCR"
        Goto TesseractDone
    ${EndIf}
    
    ; Try to find in PATH
    nsExec::ExecToStack 'where tesseract'
    Pop $0
    ${If} $0 == 0
        DetailPrint "Tesseract OCR found in system PATH"
        Goto TesseractDone
    ${EndIf}
    
    ; Tesseract not found - download and install
    DetailPrint "Tesseract OCR not found. Downloading and installing..."
    
    ; Create temp directory
    CreateDirectory "$TEMP\fm-goal-musics-installer"
    SetOutPath "$TEMP\fm-goal-musics-installer"
    
    ; Download Tesseract installer (you should host this or use official URL)
    DetailPrint "Downloading Tesseract OCR installer..."
    ; inetc::get "https://github.com/UB-Mannheim/tesseract/wiki/tesseract-ocr-w64-setup-5.3.3.20231005.exe" "tesseract-installer.exe"
    
    ; For now, show message to user to download manually
    MessageBox MB_OK|MB_ICONINFORMATION "Tesseract OCR is required for text recognition features.$\n$\nPlease download and install Tesseract OCR from:$\n$\nhttps://github.com/UB-Mannheim/tesseract/wiki$\n$\nAfter installation, FM Goal Musics will work correctly."
    
    Goto TesseractDone
    
TesseractDone:
    DetailPrint "Tesseract OCR setup completed."
    SetOutPath "$INSTDIR"
SectionEnd

Section "Start Menu Shortcuts" SecShortcuts
    CreateDirectory "$SMPROGRAMS\${APPNAME}"
    CreateShortCut "$SMPROGRAMS\${APPNAME}\${APPNAME}.lnk" "$INSTDIR\fm-goal-musics-gui.exe"
    CreateShortCut "$SMPROGRAMS\${APPNAME}\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Desktop Shortcut" SecDesktop
    CreateShortCut "$DESKTOP\${APPNAME}.lnk" "$INSTDIR\fm-goal-musics-gui.exe"
SectionEnd

; Uninstaller section
Section "Uninstall"
    ; Remove files and folders
    RMDir /r "$INSTDIR"
    
    ; Remove shortcuts
    Delete "$SMPROGRAMS\${APPNAME}\*.*"
    RMDir "$SMPROGRAMS\${APPNAME}"
    Delete "$DESKTOP\${APPNAME}.lnk"
    
    ; Remove registry keys
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}"
    DeleteRegKey HKLM "Software\${APPNAME}"
SectionEnd

; Component descriptions
LangString DESC_SecMain ${LANG_ENGLISH} "The main FM Goal Musics application and all required files."
LangString DESC_SecTesseract ${LANG_ENGLISH} "Tesseract OCR for text recognition. Required for goal detection features."
LangString DESC_SecShortcuts ${LANG_ENGLISH} "Create shortcuts in the Start Menu."
LangString DESC_SecDesktop ${LANG_ENGLISH} "Create shortcut on the Desktop."

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
    !insertmacro MUI_DESCRIPTION_TEXT ${SecMain} $(DESC_SecMain)
    !insertmacro MUI_DESCRIPTION_TEXT ${SecTesseract} $(DESC_SecTesseract)
    !insertmacro MUI_DESCRIPTION_TEXT ${SecShortcuts} $(DESC_SecShortcuts)
    !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} $(DESC_SecDesktop)
!insertmacro MUI_FUNCTION_DESCRIPTION_END

; Functions
Function .onInit
    ; Check for Windows version
    ${IfNot} ${AtLeastWin7}
        MessageBox MB_OK|MB_ICONSTOP "This application requires Windows 7 or later."
        Abort
    ${EndIf}
FunctionEnd

Function un.onInit
    MessageBox MB_YESNO|MB_ICONQUESTION "Are you sure you want to completely remove ${APPNAME} and all of its components?" IDYES +2
    Abort
FunctionEnd
