; Instalador universal de Lumen (Inno Setup 6.1+): detecta si el PC es ARM64 o x64, descarga
; siempre el instalador de la ultima version publicada en GitHub y lo abre. No hace falta
; regenerarlo con cada version. Lo compila .github/workflows/lumen-release.yml:
;   ISCC.exe /DOutDir=..\..\dist lumen-web.iss   ->  dist\Lumen-Instalador.exe

#ifndef OutDir
  #define OutDir "..\..\dist"
#endif
#define Base "https://github.com/SifG22/lumen-photo-editor/releases/latest/download/"

[Setup]
AppId={{9B3E7C21-6A4D-4F8E-B1C2-7D5A3E9F0B64}
AppName=Lumen Photo Editor
AppVersion=1.0
AppPublisher=SifG22
AppPublisherURL=https://sifg.net
CreateAppDir=no
Uninstallable=no
PrivilegesRequired=lowest
DisableReadyPage=no
DisableFinishedPage=yes
DisableProgramGroupPage=yes
SetupIconFile=..\..\assets\app-icon\photocraft.ico
OutputDir={#OutDir}
OutputBaseFilename=Lumen-Instalador
WizardStyle=modern
VersionInfoCompany=SifG22
VersionInfoDescription=Instalador de Lumen Photo Editor (ultima version)
VersionInfoProductName=Lumen Photo Editor

[Languages]
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "en"; MessagesFile: "compiler:Default.isl"

[Messages]
es.WelcomeLabel2=Se descargara la ultima version de Lumen Photo Editor para tu PC desde GitHub y se abrira su instalador.
en.WelcomeLabel2=The latest version of Lumen Photo Editor for your PC will be downloaded from GitHub and its installer opened.

[Code]
var
  DownloadPage: TDownloadWizardPage;

function Arch(): String;
begin
  if IsArm64 then
    Result := 'arm64'
  else if IsX64Compatible then
    Result := 'x64'
  else
    Result := '';
end;

function InitializeSetup(): Boolean;
begin
  Result := Arch() <> '';
  if not Result then
    MsgBox('Lumen necesita Windows de 64 bits (x64 o ARM64).', mbCriticalError, MB_OK);
end;

procedure InitializeWizard;
begin
  DownloadPage := CreateDownloadPage(SetupMessage(msgWizardPreparing), SetupMessage(msgPreparingDesc), nil);
end;

function NextButtonClick(CurPageID: Integer): Boolean;
var
  File: String;
  Code: Integer;
begin
  Result := True;
  if CurPageID <> wpReady then
    exit;
  File := 'Lumen-Setup-' + Arch() + '.exe';
  DownloadPage.Clear;
  DownloadPage.Add('{#Base}' + File, File, '');
  DownloadPage.Show;
  try
    try
      DownloadPage.Download;
    except
      MsgBox(AddPeriod(GetExceptionMessage), mbCriticalError, MB_OK);
      Result := False;
      exit;
    end;
  finally
    DownloadPage.Hide;
  end;
  if not ShellExec('', ExpandConstant('{tmp}\') + File, '', '', SW_SHOWNORMAL, ewWaitUntilTerminated, Code) then
  begin
    MsgBox(SysErrorMessage(Code), mbCriticalError, MB_OK);
    Result := False;
  end;
end;
