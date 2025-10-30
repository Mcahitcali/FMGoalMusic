[Setup]
AppName=FM Goal Musics
AppVersion=1.0.0
AppPublisher=FM Goal Musics
AppPublisherURL=
AppSupportURL=
AppUpdatesURL=
DefaultDirName={pf}\FM Goal Musics
DefaultGroupName=FM Goal Musics
AllowNoIcons=yes
LicenseFile=LICENSE.txt
OutputDir=.
OutputBaseFilename=FM-Goal-Musics-Installer
SetupIconFile=assets\icon.ico
Compression=lzma
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "src\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "Cargo.toml"; DestDir: "{app}"; Flags: ignoreversion
Source: "Cargo.lock"; DestDir: "{app}"; Flags: ignoreversion
Source: "build_windows.bat"; DestDir: "{app}"; Flags: ignoreversion
Source: "assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "LICENSE.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\FM Goal Musics"; Filename: "{app}\build\windows\fm-goal-musics-gui.exe"
Name: "{commondesktop}\FM Goal Musics"; Filename: "{app}\build\windows\fm-goal-musics-gui.exe"; Tasks: desktopicon

[Run]
Filename: "{tmp}\rustup-init.exe"; Parameters: "-y --default-toolchain stable"; StatusMsg: "Installing Rust (this may take several minutes)..."; Check: NotRustInstalled
Filename: "{app}\build_windows.bat"; WorkingDir: "{app}"; StatusMsg: "Building FM Goal Musics (this may take 10-15 minutes)..."; Flags: runhidden
Filename: "{app}\build\windows\fm-goal-musics-gui.exe"; Description: "Launch FM Goal Musics"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{app}"

[Code]
function NotRustInstalled: Boolean;
var
  ResultCode: Integer;
begin
  // Check if Rust is installed
  Result := not Exec('cmd.exe', '/c rustc --version', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  if Result then
  begin
    // Download Rust installer
    if not MsgBox('Rust is required for this application. Would you like to install it now?', mbConfirmation, MB_YESNO) = IDYES then
    begin
      Result := False;
    end else begin
      // Download to temp folder
      if not DownloadPage('https://win.rustup.rs/x86_64', ExpandConstant('{tmp}\rustup-init.exe')) then
      begin
        MsgBox('Failed to download Rust installer. Please check your internet connection.', mbError, MB_OK);
        Result := False;
      end;
    end;
  end;
end;

procedure DownloadPage(URL, FileName: string);
var
  ResultCode: Integer;
begin
  // Simple download using PowerShell
  Exec('powershell.exe', ExpandConstant('-Command "Invoke-WebRequest -Uri ''{#URL}'' -OutFile ''{#FileName}''"'), '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
end;
