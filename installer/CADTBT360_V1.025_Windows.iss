; CAD 360 Viet Nam V1.025 - native Windows x64 installer.
#define MyAppName "CAD 360 Viet Nam - TBT"
#define MyAppVersion "1.025"
#define MyAppPublisher "Cong ty CP Cong nghe TBT Viet Nam"

[Setup]
AppId={{0B643D1A-795E-4C0A-A36A-4AF2C03D1E6F}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={localappdata}\Programs\CADTBT360
DefaultGroupName={#MyAppName}
UninstallDisplayIcon={app}\CADTBT360.exe
OutputDir=..\dist\v1025
OutputBaseFilename=CADTBT360_V1.025_Setup_Windows_x64
Compression=lzma2
SolidCompression=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern
DisableProgramGroupPage=yes
CloseApplications=yes
RestartApplications=no

[Files]
Source: "..\CADTBT360_V1.000\target\release\cadcraft.exe"; DestDir: "{app}"; DestName: "CADTBT360.exe"; Flags: ignoreversion
Source: "..\CADTBT360_V1.000\target\release\cadcraft-cli.exe"; DestDir: "{app}"; DestName: "CADTBT360_CLI.exe"; Flags: ignoreversion
Source: "..\dist\v1025\BUILD_INFO.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\CAD 360 Viet Nam - TBT"; Filename: "{app}\CADTBT360.exe"
Name: "{autodesktop}\CAD 360 Viet Nam - TBT"; Filename: "{app}\CADTBT360.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Tao bieu tuong tren man hinh"; GroupDescription: "Tuy chon bo sung:"

[Run]
Filename: "{app}\CADTBT360.exe"; Description: "Mo CAD 360 Viet Nam"; Flags: nowait postinstall skipifsilent
