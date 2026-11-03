; CAD 360 Viet Nam - TBT Windows x64 fallback installer
; Compile on Linux after verifying real Windows binaries.
Unicode true
!ifndef APP_BUILD_DIR
  !error "APP_BUILD_DIR missing: Windows binaries are required"
!endif
!ifndef OUT_DIR
  !error "OUT_DIR missing"
!endif
Name "CAD 360 Viet Nam - TBT"
OutFile "${OUT_DIR}/CAD360_VietNam_TBT_Setup.exe"
InstallDir "$LOCALAPPDATA\Programs\CAD 360 Viet Nam - TBT"
RequestExecutionLevel user
SetCompressor /SOLID lzma
ShowInstDetails show
ShowUnInstDetails show

Section "CAD360" SecMain
  SetOutPath "$INSTDIR"
  File "${APP_BUILD_DIR}/cadcraft.exe"
  File "${APP_BUILD_DIR}/cadcraft-cli.exe"
  CreateDirectory "$SMPROGRAMS\CAD 360 Viet Nam - TBT"
  CreateShortcut "$SMPROGRAMS\CAD 360 Viet Nam - TBT\CAD 360 Viet Nam - TBT.lnk" "$INSTDIR\cadcraft.exe"
  CreateShortcut "$DESKTOP\CAD 360 Viet Nam - TBT.lnk" "$INSTDIR\cadcraft.exe"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
SectionEnd

Section "Uninstall"
  Delete "$DESKTOP\CAD 360 Viet Nam - TBT.lnk"
  Delete "$SMPROGRAMS\CAD 360 Viet Nam - TBT\CAD 360 Viet Nam - TBT.lnk"
  RMDir "$SMPROGRAMS\CAD 360 Viet Nam - TBT"
  Delete "$INSTDIR\cadcraft.exe"
  Delete "$INSTDIR\cadcraft-cli.exe"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
SectionEnd
