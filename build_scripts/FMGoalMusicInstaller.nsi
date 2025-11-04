!include "MUI2.nsh"

!define BASE "build\windows"  ; PowerShell script'inin oluşturduğu build klasörü

!define APP_NAME "FM Goal Musics"

!define APP_EXE "fm-goal-musics-gui.exe"

!define INSTALL_DIR "$PROGRAMFILES\${APP_NAME}"

!define STARTMENU_FOLDER "${APP_NAME}"

!define LICENSE_FILE "LICENSE.txt"

; ---- MOD SEÇ ----

!define STRICT ; bu satır açık: strict mod. Yoruma al → relaxed moda geç.

SetCompressor lzma

Unicode true

SetDatablockOptimize off

Name "${APP_NAME}"

OutFile "FMGoalMusicInstaller.exe"

InstallDir "${INSTALL_DIR}"

RequestExecutionLevel admin

ShowInstDetails show

!insertmacro MUI_PAGE_WELCOME

!insertmacro MUI_PAGE_LICENSE "${LICENSE_FILE}"

!insertmacro MUI_PAGE_DIRECTORY

!insertmacro MUI_PAGE_INSTFILES

!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

Section "Install ${APP_NAME}"

DetailPrint "Installing to: ${INSTALL_DIR}"

; EXE (zorunlu)

SetOutPath "${INSTALL_DIR}"

DetailPrint "Extract EXE"

File "${BASE}\${APP_EXE}"

; CONFIG

SetOutPath "${INSTALL_DIR}\config"

DetailPrint "Extract config"

!ifdef STRICT

File /r "${BASE}\config\*.*"

!else

File /nonfatal /r "${BASE}\config\*.*"

!endif

; ASSETS

SetOutPath "${INSTALL_DIR}\assets"

DetailPrint "Extract assets"

!ifdef STRICT

File /r "${BASE}\assets\*.*"

!else

File /nonfatal /r "${BASE}\assets\*.*"

!endif

; TESSDATA (bundle)

SetOutPath "${INSTALL_DIR}\tessdata"

DetailPrint "Extract tessdata (bundled)"

!ifdef STRICT

File /r "${BASE}\tessdata\*.*"

!else

File /nonfatal /r "${BASE}\tessdata\*.*"

!endif

; TESSDATA fallback (system)

DetailPrint "Extract tessdata (system fallback)"

!ifdef STRICT

File /r "${BASE}\tessdata\*.*"

!else

File /nonfatal /r "${BASE}\tessdata\*.*"

!endif

WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "TESSDATA_PREFIX" "$INSTDIR\tessdata"

CreateDirectory "$SMPROGRAMS\${STARTMENU_FOLDER}"

CreateShortCut "$SMPROGRAMS\${STARTMENU_FOLDER}\${APP_NAME}.lnk" "${INSTALL_DIR}\${APP_EXE}" "" "${INSTALL_DIR}\${APP_EXE}" 0

CreateShortCut "$DESKTOP\${APP_NAME}.lnk" "${INSTALL_DIR}\${APP_EXE}" "" "${INSTALL_DIR}\${APP_EXE}" 0

WriteUninstaller "$INSTDIR\Uninstall.exe"

WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "DisplayName" "${APP_NAME}"

WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}" "UninstallString" "$INSTDIR\Uninstall.exe"

DetailPrint "Install finished."

SectionEnd

Section "Uninstall"

Delete "$DESKTOP\${APP_NAME}.lnk"

Delete "$SMPROGRAMS\${STARTMENU_FOLDER}\${APP_NAME}.lnk"

RMDir "$SMPROGRAMS\${STARTMENU_FOLDER}"

DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"

DeleteRegValue HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "TESSDATA_PREFIX"

RMDir /r "${INSTALL_DIR}"

SectionEnd

