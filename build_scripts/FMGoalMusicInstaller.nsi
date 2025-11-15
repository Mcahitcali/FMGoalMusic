; ==========================================
; FM Goal Musics – Windows Installer
; ==========================================

!include "MUI2.nsh"

Name "FM Goal Musics"
OutFile "..\build\windows\FMGoalMusicInstaller.exe"
InstallDir "$PROGRAMFILES\FM Goal Musics"
RequestExecutionLevel admin
ShowInstDetails show
ShowUninstDetails show
BrandingText "FM Goal Musics Installer"

; Set custom icons (adjust path if needed)
!define MUI_ICON "..\assets\app.ico"
!define MUI_UNICON "..\assets\app.ico"

Var StartMenuFolder

!define MUI_ABORTWARNING

; Run app checkbox on finish page
!define MUI_FINISHPAGE_RUN "$INSTDIR\fm-goal-musics-gui.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Run FM Goal Musics"

;--------------------------------
; MUI Pages

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\license.txt"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_STARTMENU Application $StartMenuFolder
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

;--------------------------------
; Installer Section

Section "MainSection" SEC01
  SetOutPath "$INSTDIR"
  File /r "..\build\windows\*.*"

  ; Write uninstaller
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  ; Start Menu shortcuts
  !insertmacro MUI_STARTMENU_WRITE_BEGIN Application
    CreateDirectory "$SMPROGRAMS\$StartMenuFolder"
    CreateShortCut "$SMPROGRAMS\$StartMenuFolder\FM Goal Musics.lnk" "$INSTDIR\fm-goal-musics-gui.exe"
    CreateShortCut "$SMPROGRAMS\$StartMenuFolder\Uninstall FM Goal Musics.lnk" "$INSTDIR\Uninstall.exe"
  !insertmacro MUI_STARTMENU_WRITE_END

  ; Desktop shortcut
  CreateShortCut "$DESKTOP\FM Goal Musics.lnk" "$INSTDIR\fm-goal-musics-gui.exe"

  ; Add entry to Add/Remove Programs
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FM Goal Musics" "DisplayName" "FM Goal Musics"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FM Goal Musics" "UninstallString" "$INSTDIR\Uninstall.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FM Goal Musics" "InstallLocation" "$INSTDIR"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FM Goal Musics" "DisplayIcon" "$INSTDIR\fm-goal-musics-gui.exe"
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FM Goal Musics" "NoModify" 1
  WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FM Goal Musics" "NoRepair" 1
SectionEnd

;--------------------------------
; Uninstaller Section

Section "Uninstall"
  ; Remove shortcuts
  Delete "$DESKTOP\FM Goal Musics.lnk"

  !insertmacro MUI_STARTMENU_GETFOLDER Application $StartMenuFolder
  Delete "$SMPROGRAMS\$StartMenuFolder\FM Goal Musics.lnk"
  Delete "$SMPROGRAMS\$StartMenuFolder\Uninstall FM Goal Musics.lnk"
  RMDir "$SMPROGRAMS\$StartMenuFolder"

  ; Remove files and directory
  RMDir /r "$INSTDIR"

  ; Remove Add/Remove Programs entry
  DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\FM Goal Musics"
SectionEnd
