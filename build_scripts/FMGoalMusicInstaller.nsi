; ==========================================
; FM Goal Musics – Universal Installer (for GitHub Actions & local)
; ==========================================
OutFile "..\build\windows\FMGoalMusicInstaller.exe"
InstallDir "$PROGRAMFILES\FM Goal Musics"
RequestExecutionLevel admin
ShowInstDetails show
ShowUninstDetails show

!include "MUI2.nsh"

Name "FM Goal Musics"
BrandingText "FM Goal Musics Installer"

Var StartMenuFolder

!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

Section "MainSection" SEC01
  SetOutPath "$INSTDIR"
  File /r "..\build\windows\*.*"
  CreateShortCut "$DESKTOP\FM Goal Musics.lnk" "$INSTDIR\fm-goal-musics-gui.exe"
  CreateShortCut "$SMPROGRAMS\FM Goal Musics\FM Goal Musics.lnk" "$INSTDIR\fm-goal-musics-gui.exe"
SectionEnd

Section "Uninstall"
  Delete "$DESKTOP\FM Goal Musics.lnk"
  Delete "$SMPROGRAMS\FM Goal Musics\FM Goal Musics.lnk"
  RMDir /r "$INSTDIR"
  RMDir "$SMPROGRAMS\FM Goal Musics"
SectionEnd
