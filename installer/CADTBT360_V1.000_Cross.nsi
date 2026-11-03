; CADTBT360_V1.000 - Windows x64, NSIS cross compiled.
Unicode true
!ifndef APP_BUILD_DIR
  !error "APP_BUILD_DIR missing"
!endif
!ifndef OUT_DIR
  !error "OUT_DIR missing"
!endif
Name "CAD TBT 360 V1.000"
OutFile "${OUT_DIR}/CADTBT360_V1.000_Setup.exe"
InstallDir "$LOCALAPPDATA\Programs\CADTBT360_V1.000"
RequestExecutionLevel user
SetCompressor /SOLID lzma
ShowInstDetails show
Section "CAD TBT 360" SecMain
  SetOutPath "$INSTDIR"
  File /oname=CADTBT360.exe "${APP_BUILD_DIR}/cadcraft.exe"
  File /oname=CADTBT360_CLI.exe "${APP_BUILD_DIR}/cadcraft-cli.exe"
  CreateDirectory "$SMPROGRAMS\CAD TBT 360"
  CreateShortcut "$SMPROGRAMS\CAD TBT 360\CADTBT360_V1.000.lnk" "$INSTDIR\CADTBT360.exe"
  CreateShortcut "$DESKTOP\CADTBT360_V1.000.lnk" "$INSTDIR\CADTBT360.exe"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd
Section "Uninstall"
  Delete "$DESKTOP\CADTBT360_V1.000.lnk"
  Delete "$SMPROGRAMS\CAD TBT 360\CADTBT360_V1.000.lnk"
  RMDir "$SMPROGRAMS\CAD TBT 360"
  Delete "$INSTDIR\CADTBT360.exe"
  Delete "$INSTDIR\CADTBT360_CLI.exe"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
SectionEnd
