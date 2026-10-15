; Instalador de Lumen para Windows (Inno Setup 6). Lo genera packaging\lumen\empaquetar.ps1 -Instalador,
; que pasa la version, la arquitectura y la carpeta ya preparada:
;   ISCC.exe /DVersion=0.0.1 /DArch=x64 /DSource=..\..\dist\Lumen-0.0.1-windows-x64 /DOutDir=..\..\dist lumen.iss
;
; Se instala para el usuario actual (sin pedir administrador) en %LOCALAPPDATA%\Programs\Lumen; quien
; tenga permisos puede elegir "para todos los usuarios" y va a Archivos de programa. Crea el acceso en
; el menu Inicio (y en el escritorio si se marca), abre los .lumen con doble clic y se desinstala
; desde Configuracion > Aplicaciones.

#ifndef Version
  #define Version "0.0.1"
#endif
#ifndef Arch
  #define Arch "x64"
#endif
#ifndef Source
  #define Source "..\..\dist\Lumen-" + Version + "-windows-" + Arch
#endif
#ifndef OutDir
  #define OutDir "..\..\dist"
#endif

[Setup]
; El AppId identifica a Lumen para actualizar y desinstalar: no cambiarlo nunca.
AppId={{6F1C2A8E-4B7D-4E59-9C3A-5D2E8B1F7A40}
AppName=Lumen Photo Editor
AppVersion={#Version}
AppVerName=Lumen Photo Editor {#Version}
AppPublisher=SifG22
AppPublisherURL=https://sifg.net
AppSupportURL=https://sifg.net
AppUpdatesURL=https://sifg.net
DefaultDirName={autopf}\Lumen
DefaultGroupName=Lumen Photo Editor
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
#if Arch == "arm64"
ArchitecturesAllowed=arm64
ArchitecturesInstallIn64BitMode=arm64
#else
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
#endif
SetupIconFile=..\..\assets\app-icon\photocraft.ico
UninstallDisplayIcon={app}\Lumen.exe
UninstallDisplayName=Lumen Photo Editor
LicenseFile={#Source}\TERMINOS.txt
OutputDir={#OutDir}
OutputBaseFilename=Lumen-Setup-{#Version}-{#Arch}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ChangesAssociations=yes
CloseApplications=yes
VersionInfoVersion={#Version}
VersionInfoCompany=SifG22
VersionInfoDescription=Instalador de Lumen Photo Editor
VersionInfoProductName=Lumen Photo Editor

[Languages]
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "en"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "{#Source}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\Lumen Photo Editor"; Filename: "{app}\Lumen.exe"; AppUserModelID: "SifG22.Lumen"
Name: "{autodesktop}\Lumen Photo Editor"; Filename: "{app}\Lumen.exe"; Tasks: desktopicon

[Registry]
; Los proyectos .lumen se abren con Lumen (y aparece en "Abrir con" para las imagenes habituales).
Root: HKA; Subkey: "Software\Classes\.lumen"; ValueType: string; ValueName: ""; ValueData: "Lumen.Proyecto"; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Lumen.Proyecto"; ValueType: string; ValueName: ""; ValueData: "Proyecto de Lumen Photo Editor"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Lumen.Proyecto\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\Lumen.exe,0"
Root: HKA; Subkey: "Software\Classes\Lumen.Proyecto\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Lumen.exe"" ""%1"""
Root: HKA; Subkey: "Software\Classes\Applications\Lumen.exe"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "Lumen Photo Editor"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\Lumen.exe\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Lumen.exe"" ""%1"""
Root: HKA; Subkey: "Software\Classes\Applications\Lumen.exe\SupportedTypes"; ValueType: string; ValueName: ".lumen"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\Applications\Lumen.exe\SupportedTypes"; ValueType: string; ValueName: ".jpg"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\Applications\Lumen.exe\SupportedTypes"; ValueType: string; ValueName: ".jpeg"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\Applications\Lumen.exe\SupportedTypes"; ValueType: string; ValueName: ".png"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\Applications\Lumen.exe\SupportedTypes"; ValueType: string; ValueName: ".webp"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\Applications\Lumen.exe\SupportedTypes"; ValueType: string; ValueName: ".psd"; ValueData: ""

[Run]
Filename: "{app}\Lumen.exe"; Description: "{cm:LaunchProgram,Lumen Photo Editor}"; Flags: nowait postinstall skipifsilent
