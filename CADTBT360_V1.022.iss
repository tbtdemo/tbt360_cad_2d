; CADTBT360 V1.022 — checked-in Inno Setup 6 script.
; Run ISCC.exe CADTBT360_V1.022.iss from the repository root after Windows Rust build.
[Setup]
AppId={{6BCA4011-C394-4C3D-AB6D-3D8D5A0B1000}
AppName=CAD TBT 360
AppVersion=1.0.22
AppVerName=CAD TBT 360 V1.022
DefaultDirName={autopf}\CAD TBT 360
DefaultGroupName=CAD TBT 360
OutputDir=dist
OutputBaseFilename=CADTBT360_V1.022_Setup
Compression=lzma
SolidCompression=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayName=CAD TBT 360 V1.022

[Tasks]
Name: "desktopicon"; Description: "Tạo biểu tượng màn hình"; GroupDescription: "Biểu tượng:"

[Files]
Source: "dist\CADTBT360_V1.022.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\CAD TBT 360"; Filename: "{app}\CADTBT360_V1.022.exe"
Name: "{autodesktop}\CAD TBT 360"; Filename: "{app}\CADTBT360_V1.022.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\CADTBT360_V1.022.exe"; Description: "Mở CAD TBT 360"; Flags: nowait postinstall skipifsilent
