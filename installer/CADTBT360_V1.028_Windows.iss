; CAD 360 Viet Nam V1.028 - native Windows x64 installer.
#define MyAppName "CAD 360 Viet Nam - TBT"
#define MyAppVersion "1.028"
#define MyAppPublisher "Cong ty CP Cong nghe TBT Viet Nam"

[Setup]
AppId={{0B643D1A-795E-4C0A-A36A-4AF2C03D1E6F}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={localappdata}\Programs\CADTBT360
DefaultGroupName={#MyAppName}
UninstallDisplayIcon={app}\CADTBT360.exe
OutputDir=..\dist\v1028
OutputBaseFilename=CADTBT360_V1.028_Setup_Windows_x64
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
Source: "..\dist\v1028\BUILD_INFO.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\dist\v1028\pdf-tools\bin\*"; DestDir: "{app}\Tools\Poppler\bin"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\dist\v1028\pdf-tools\licenses\*"; DestDir: "{app}\Tools\Poppler\licenses"; Flags: ignoreversion skipifsourcedoesntexist
Source: "..\docs\user-guide\CADTBT360_V1.025_HUONG_DAN_AI_TU_ANH_THANH_CAD.md"; DestDir: "{app}\HuongDan"; DestName: "HUONG_DAN_AI_TU_ANH_THANH_CAD.md"; Flags: ignoreversion
Source: "..\dist\v1028\CADTBT360_V1.028_HUONG_DAN_AI_TU_ANH_THANH_CAD.docx"; DestDir: "{app}\HuongDan"; DestName: "HUONG_DAN_AI_TU_ANH_THANH_CAD.docx"; Flags: ignoreversion
Source: "..\dist\v1028\CADTBT360_V1.028_GIAO_TRINH_TONG_HOP.docx"; DestDir: "{app}\HuongDan"; DestName: "GIAO_TRINH_TONG_HOP_CAD360.docx"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\CAD 360 Viet Nam - TBT"; Filename: "{app}\CADTBT360.exe"
Name: "{autoprograms}\CAD 360 Viet Nam - TBT - Huong dan AI"; Filename: "{app}\HuongDan\HUONG_DAN_AI_TU_ANH_THANH_CAD.docx"
Name: "{autoprograms}\CAD 360 Viet Nam - TBT - Giao trinh tong hop"; Filename: "{app}\HuongDan\GIAO_TRINH_TONG_HOP_CAD360.docx"
Name: "{autodesktop}\CAD 360 Viet Nam - TBT"; Filename: "{app}\CADTBT360.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Tao bieu tuong tren man hinh"; GroupDescription: "Tuy chon bo sung:"

[Run]
Filename: "{app}\CADTBT360.exe"; Description: "Mo CAD 360 Viet Nam"; Flags: nowait postinstall skipifsilent
