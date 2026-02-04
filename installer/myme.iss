; MyMe Installer Script
; Requires Inno Setup 6.x

#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif

[Setup]
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName=MyMe
AppVersion={#AppVersion}
AppVerName=MyMe {#AppVersion}
AppPublisher=jonesrussell
AppPublisherURL=https://github.com/jonesrussell/myme
AppSupportURL=https://github.com/jonesrussell/myme/issues
AppUpdatesURL=https://github.com/jonesrussell/myme/releases
DefaultDirName={autopf}\MyMe
DefaultGroupName=MyMe
AllowNoIcons=yes
LicenseFile=..\LICENSE
OutputDir=..\dist
OutputBaseFilename=myme-{#AppVersion}-windows-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=admin
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; Main application and all bundled files from windeployqt
Source: "..\dist\myme\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\MyMe"; Filename: "{app}\myme-qt.exe"
Name: "{group}\{cm:UninstallProgram,MyMe}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\MyMe"; Filename: "{app}\myme-qt.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\myme-qt.exe"; Description: "{cm:LaunchProgram,MyMe}"; Flags: nowait postinstall skipifsilent
