; Builds the Windows installer: build the game first (`cargo build --release --bin t2f`), then
; `ISCC.exe installer\road-to-alfendrof.iss`. The installer is written to installer\Output.

#define AppName "Road to Alfendrof"
#define AppVersion "0.1.1"
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

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExe}"; WorkingDir: "{app}"; IconFilename: "{app}\emblem.ico"
Name: "{group}\Uninstall {#AppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; WorkingDir: "{app}"; IconFilename: "{app}\emblem.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExe}"; Description: "Play {#AppName}"; WorkingDir: "{app}"; Flags: nowait postinstall skipifsilent
