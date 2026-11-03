; CADTBT360_V1.000: native Windows x64 installer.
#define MyAppName "CAD TBT 360"
#define MyAppVersion "1.000"
#define MyAppPublisher "Công ty CP CN TBT Việt Nam"
[Setup]
AppId={{0B643D1A-795E-4C0A-A36A-4AF2C03D1E6F}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={localappdata}\Programs\CADTBT360_V1.000
DefaultGroupName={#MyAppName}
UninstallDisplayIcon={app}\CADTBT360.exe
OutputDir=..\dist
OutputBaseFilename=CADTBT360_V1.000_Setup
Compression=lzma2
SolidCompression=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
DisableProgramGroupPage=yes
[Files]
Source: "..\CADTBT360_V1.000\target\release\cadcraft.exe"; DestDir: "{app}"; DestName: "CADTBT360.exe"; Flags: ignoreversion
Source: "..\CADTBT360_V1.000\target\release\cadcraft-cli.exe"; DestDir: "{app}"; DestName: "CADTBT360_CLI.exe"; Flags: ignoreversion
[Icons]
Name: "{autoprograms}\CAD TBT 360"; Filename: "{app}\CADTBT360.exe"
Name: "{autodesktop}\CAD TBT 360"; Filename: "{app}\CADTBT360.exe"; Tasks: desktopicon
[Tasks]
Name: "desktopicon"; Description: "Tạo biểu tượng ngoài màn hình"; GroupDescription: "Tùy chọn bổ sung:"
[Run]
Filename: "{app}\CADTBT360.exe"; Description: "Mở CAD TBT 360"; Flags: nowait postinstall skipifsilent
