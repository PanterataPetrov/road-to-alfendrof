; Builds the Windows installer: build the game first (`cargo build --release --bin t2f`), then
; `ISCC.exe installer\road-to-alfendrof.iss`. The installer is written to installer\Output.
; CI passes /DAppVersion=<Cargo.toml version>; manual builds retain the default below.

#define AppName "Road to Alfendrof"
#ifndef AppVersion
#define AppVersion "0.1.7"
#endif
#define AppExe "RoadToAlfendrof.exe"

[Setup]
AppId={{6F1B8F55-3C0D-4E5A-9C36-6E8D5C1A7B42}
AppName={#AppName}
AppVersion={#AppVersion}
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
OutputDir=Output
OutputBaseFilename=RoadToAlfendrof-Setup
SetupIconFile=emblem.ico
UninstallDisplayIcon={app}\emblem.ico
Compression=lzma2/ultra
SolidCompression=yes
; installs for the current user without asking for admin rights unless they choose otherwise
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Shortcuts:"

[Files]
Source: "..\target\release\t2f.exe"; DestDir: "{app}"; DestName: "{#AppExe}"; Flags: ignoreversion
Source: "..\assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "emblem.ico"; DestDir: "{app}"; Flags: ignoreversion
; Microsoft's shader compiler; the game picks it up from its own folder on DirectX 12 (run get_dxc.ps1 once to fetch it)
Source: "dxc\dxcompiler.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "dxc\dxil.dll"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExe}"; WorkingDir: "{app}"; IconFilename: "{app}\emblem.ico"
Name: "{group}\Uninstall {#AppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; WorkingDir: "{app}"; IconFilename: "{app}\emblem.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExe}"; Description: "Play {#AppName}"; WorkingDir: "{app}"; Flags: nowait postinstall skipifsilent
; an update started by the game itself (it passes /relaunch=1) starts the game again when done
Filename: "{app}\{#AppExe}"; WorkingDir: "{app}"; Flags: nowait runasoriginaluser; Check: Relaunch

[Code]
function Relaunch: Boolean;
begin
  Result := ExpandConstant('{param:relaunch|0}') = '1';
end;
