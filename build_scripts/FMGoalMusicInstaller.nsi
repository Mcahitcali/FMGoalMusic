!define APP_NAME "FM Goal Musics"
!define COMPANY  "FM Goal Musics"
!define VERSION  "1.0.0"

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

  ; Ensure a writable config folder if you need one in ProgramData (optional)
  ; CreateDirectory "$PROGRAMDATA\${APP_NAME}"

  ; Add app dir to PATH (per-machine)
  ReadRegStr $0 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path"
  StrCpy $1 "$INSTDIR"
  StrCpy $2 "$INSTDIR\tessdata"
  ${IfThen} ${Errors} 0
  StrCpy $3 "$0;$1;$2"
  WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path" "$3"

  ; Set TESSDATA_PREFIX to bundled tessdata
  WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "TESSDATA_PREFIX" "$INSTDIR\tessdata"

  ; Broadcast environment change
  System::Call 'USER32::SendMessageTimeout(p 0xffff, i ${WM_SETTINGCHANGE}, i 0, t "Environment", i 0, i 5000, *i .r0)'

  ; Create shortcuts
  CreateShortCut "$SMPROGRAMS\${APP_NAME}.lnk" "$INSTDIR\fm-goal-musics-gui.exe"
  CreateShortCut "$DESKTOP\${APP_NAME}.lnk"     "$INSTDIR\fm-goal-musics-gui.exe"
SectionEnd

Section "Uninstall"
  Delete "$SMPROGRAMS\${APP_NAME}.lnk"
  Delete "$DESKTOP\${APP_NAME}.lnk"
  RMDir /r "$INSTDIR"

  ; Remove env vars we added (simple approach: do not attempt to surgically edit PATH; leave as-is if you prefer)
  ; WriteRegStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "TESSDATA_PREFIX" ""

  System::Call 'USER32::SendMessageTimeout(p 0xffff, i ${WM_SETTINGCHANGE}, i 0, t "Environment", i 0, i 5000, *i .r0)'
SectionEnd
