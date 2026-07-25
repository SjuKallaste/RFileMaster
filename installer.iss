#define AppName "RFileMaster"
#define AppVersion "0.1.0"
#define AppPublisher "RFileMaster"
#define AppExe "rfilemaster.exe"
#define LibreOfficeVersion "25.8.7"

[Setup]
AppId={{7C1F2A3B-4D5E-4A6B-9C8D-1E2F3A4B5C6D}}
AppName={#AppName}
AppVersion={#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
OutputDir=dist
OutputBaseFilename=RFileMaster-Setup-{#AppVersion}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=admin

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"
Name: "getytdlp"; Description: "yt-dlp (enables downloading video and audio from YouTube)"; GroupDescription: "Install automatically (recommended):"
Name: "getffmpeg"; Description: "ffmpeg (enables audio and video conversion)"; GroupDescription: "Install automatically (recommended):"
Name: "getlibreoffice"; Description: "LibreOffice (enables Word, PowerPoint, and PDF conversion)"; GroupDescription: "Install automatically (recommended):"

[Files]
Source: "target\release\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "icon\icon.png"; DestDir: "{app}\icon"; Flags: ignoreversion
Source: "icon\icon.ico"; DestDir: "{app}\icon"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{group}\Uninstall {#AppName}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExe}"; Description: "Launch {#AppName}"; Flags: nowait postinstall skipifsilent

[Code]
var
  DownloadPage: TDownloadWizardPage;
  StatusPage: TOutputProgressWizardPage;

function OnDownloadProgress(const Url, FileName: String; const Progress, ProgressMax: Int64): Boolean;
begin
  Result := True;
end;

procedure InitializeWizard;
begin
  DownloadPage := CreateDownloadPage(SetupMessage(msgWizardPreparing), SetupMessage(msgPreparingDesc), @OnDownloadProgress);
  StatusPage := CreateOutputProgressPage('Setting up components', 'Please wait while additional components are installed.');
end;

function FindFileRecursive(RootDir, TargetName: String): String;
var
  FindRec: TFindRec;
  FullPath, Found: String;
begin
  Result := '';
  if FindFirst(RootDir + '\*', FindRec) then
  begin
    try
      repeat
        if (FindRec.Name <> '.') and (FindRec.Name <> '..') then
        begin
          FullPath := RootDir + '\' + FindRec.Name;
          if (FindRec.Attributes and FILE_ATTRIBUTE_DIRECTORY) <> 0 then
          begin
            Found := FindFileRecursive(FullPath, TargetName);
            if Found <> '' then
            begin
              Result := Found;
              Exit;
            end;
          end
          else if CompareText(FindRec.Name, TargetName) = 0 then
          begin
            Result := FullPath;
            Exit;
          end;
        end;
      until not FindNext(FindRec);
    finally
      FindClose(FindRec);
    end;
  end;
end;

function NextButtonClick(CurPageID: Integer): Boolean;
begin
  Result := True;
  if CurPageID = wpReady then
  begin
    DownloadPage.Clear;
    if WizardIsTaskSelected('getytdlp') then
      DownloadPage.Add('https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe', 'yt-dlp.exe', '');
    if WizardIsTaskSelected('getffmpeg') then
      DownloadPage.Add('https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip', 'ffmpeg.zip', '');
    if WizardIsTaskSelected('getlibreoffice') then
      DownloadPage.Add('https://download.documentfoundation.org/libreoffice/stable/{#LibreOfficeVersion}/win/x86_64/LibreOffice_{#LibreOfficeVersion}_Win_x86-64.msi', 'libreoffice.msi', '');

    if (WizardIsTaskSelected('getytdlp')) or (WizardIsTaskSelected('getffmpeg')) or (WizardIsTaskSelected('getlibreoffice')) then
    begin
      DownloadPage.Show;
      try
        try
          DownloadPage.Download;
        except
          if not DownloadPage.AbortedByUser then
            MsgBox('Some optional components could not be downloaded. RFileMaster will still install, but the affected conversion types will show an error until you install them manually later.' + #13#10#13#10 + AddPeriod(GetExceptionMessage), mbInformation, MB_OK);
        end;
      finally
        DownloadPage.Hide;
      end;
    end;
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  ResultCode: Integer;
  ExtractDir, FoundExe, ToolsDir, PowerShellCmd: String;
begin
  if CurStep = ssPostInstall then
  begin
    ToolsDir := ExpandConstant('{app}\tools');
    ForceDirectories(ToolsDir);

    StatusPage.Show;
    try
      if WizardIsTaskSelected('getytdlp') and FileExists(ExpandConstant('{tmp}\yt-dlp.exe')) then
      begin
        StatusPage.SetText('Installing yt-dlp...', '');
        FileCopy(ExpandConstant('{tmp}\yt-dlp.exe'), ToolsDir + '\yt-dlp.exe', False);
      end;

      if WizardIsTaskSelected('getffmpeg') and FileExists(ExpandConstant('{tmp}\ffmpeg.zip')) then
      begin
        StatusPage.SetText('Installing ffmpeg...', 'Extracting archive');
        ExtractDir := ExpandConstant('{tmp}\ffmpeg_extracted');
        ForceDirectories(ExtractDir);
        PowerShellCmd := '-NoProfile -ExecutionPolicy Bypass -Command "Expand-Archive -LiteralPath ''' + ExpandConstant('{tmp}\ffmpeg.zip') + ''' -DestinationPath ''' + ExtractDir + ''' -Force"';
        Exec('powershell.exe', PowerShellCmd, '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
        FoundExe := FindFileRecursive(ExtractDir, 'ffmpeg.exe');
        if FoundExe <> '' then
          FileCopy(FoundExe, ToolsDir + '\ffmpeg.exe', False);
      end;

      if WizardIsTaskSelected('getlibreoffice') and FileExists(ExpandConstant('{tmp}\libreoffice.msi')) then
      begin
        StatusPage.SetText('Installing LibreOffice...', 'This may take a few minutes');
        Exec('msiexec.exe', '/i "' + ExpandConstant('{tmp}\libreoffice.msi') + '" /qn /norestart', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
      end;
    finally
      StatusPage.Hide;
    end;
  end;
end;

[UninstallDelete]
Type: filesandordirs; Name: "{app}\tools"
