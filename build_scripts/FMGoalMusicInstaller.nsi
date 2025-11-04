!define APP_NAME "FM Goal Musics"
!define COMPANY  "FM Goal Musics"
!define VERSION  "1.0.0"

!verbose 4
OutFile "build_scripts\FMGoalMusicInstaller.exe"
InstallDir "$PROGRAMFILES\${APP_NAME}"
RequestExecutionLevel admin
SetCompress auto

Page directory
Page instfiles
UninstPage uninstConfirm
UninstPage instfiles

Section "Install"
  SetShellVarContext all
  SetOutPath "$INSTDIR"
  File /r "build\windows\*.*"

  ; Point Tesseract to $INSTDIR\tessdata (bundled by the build script)
  WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "TESSDATA_PREFIX" "$INSTDIR"
  System::Call 'USER32::SendMessageTimeout(p 0xffff, i ${WM_SETTINGCHANGE}, i 0, t "Environment", i 0, i 5000, *i .r0)'

  CreateShortCut "$SMPROGRAMS\${APP_NAME}.lnk" "$INSTDIR\fm-goal-musics-gui.exe"
  CreateShortCut "$DESKTOP\${APP_NAME}.lnk"     "$INSTDIR\fm-goal-musics-gui.exe"
SectionEnd

Section "Uninstall"
  Delete "$SMPROGRAMS\${APP_NAME}.lnk"
  Delete "$DESKTOP\${APP_NAME}.lnk"
  RMDir /r "$INSTDIR"
  ; Optional: clear TESSDATA_PREFIX
  ; WriteRegStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "TESSDATA_PREFIX" ""
  System::Call 'USER32::SendMessageTimeout(p 0xffff, i ${WM_SETTINGCHANGE}, i 0, t "Environment", i 0, i 5000, *i .r0)'
SectionEnd
