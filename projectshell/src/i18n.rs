use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Korean,
    English,
}

impl Default for Language {
    fn default() -> Self {
        Self::Korean
    }
}

#[derive(Debug, Clone, Copy)]
pub struct I18n {
    language: Language,
}

impl I18n {
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    pub fn language(self) -> Language {
        self.language
    }

    pub fn set_language(&mut self, language: Language) {
        self.language = language;
    }

    pub fn t(self, key: Text) -> &'static str {
        match self.language {
            Language::Korean => key.ko(),
            Language::English => key.en(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Text {
    Active,
    Alias,
    AppNotFound,
    Apps,
    AppPathMissing,
    AppProcessHint,
    AppAdded,
    AppDeleted,
    Args,
    Assigned,
    AssignSelectedRunningApp,
    Back,
    CtrlAssign,
    Delete,
    DeleteProject,
    Description,
    EnterFocus,
    EnterLaunch,
    EnterResume,
    ExecutablePath,
    Failed,
    Focused,
    FocusFailed,
    Help,
    HelpShortcuts,
    HelpUsage,
    HintFooter,
    Language,
    Launched,
    LaunchSetEmpty,
    MissingPath,
    Name,
    NewApp,
    NewWorkspace,
    NoAppSelected,
    NoProjectSelected,
    NoRowSelected,
    NoWorkspaceSelected,
    NoWorkspaceFound,
    OpenedWorkspace,
    ProcessStatusUnavailable,
    Project,
    ProjectShellTitle,
    RunningApp,
    RunningAppNotFound,
    RunningApps,
    ResidentActive,
    ResidentUnavailable,
    SavedSettings,
    SearchHint,
    SelectRunningOrWorkspace,
    SelectedApp,
    SelectedProject,
    Settings,
    Status,
    Stopped,
    Unknown,
    WindowTitleMatch,
    Workspace,
    WorkspaceAdded,
    WorkspaceDeleted,
    WorkspaceNav,
    WorkspaceNotFound,
}

impl Text {
    fn ko(self) -> &'static str {
        match self {
            Text::Active => "활성",
            Text::Alias => "별명",
            Text::AppNotFound => "앱을 찾을 수 없습니다.",
            Text::Apps => "앱",
            Text::AppPathMissing => "앱 실행 경로가 없습니다.",
            Text::AppProcessHint => "프로세스명, 예: code.exe",
            Text::AppAdded => "앱이 추가되었습니다.",
            Text::AppDeleted => "앱이 삭제되었습니다.",
            Text::Args => "인자",
            Text::Assigned => "귀속됨",
            Text::AssignSelectedRunningApp => "귀속할 실행중 앱을 선택하세요.",
            Text::Back => "뒤로",
            Text::CtrlAssign => "Ctrl+A 귀속",
            Text::Delete => "삭제",
            Text::DeleteProject => "프로젝트 삭제",
            Text::Description => "설명",
            Text::EnterFocus => "Enter 포커스",
            Text::EnterLaunch => "Enter 실행",
            Text::EnterResume => "Enter 재개",
            Text::ExecutablePath => "실행 파일 경로",
            Text::Failed => "실패",
            Text::Focused => "포커스",
            Text::FocusFailed => "포커스 실패",
            Text::Help => "도움말",
            Text::HelpShortcuts => "단축키",
            Text::HelpUsage => "사용 방법",
            Text::HintFooter => "Enter 포커스/재개 | Ctrl+A 귀속 | Ctrl+O 열기 | Ctrl+, 설정",
            Text::Language => "언어",
            Text::Launched => "실행",
            Text::LaunchSetEmpty => "실행할 앱이 없습니다.",
            Text::MissingPath => "경로 없음",
            Text::Name => "이름",
            Text::NewApp => "새 앱",
            Text::NewWorkspace => "새 작업공간",
            Text::NoAppSelected => "선택된 앱이 없습니다.",
            Text::NoProjectSelected => "선택된 프로젝트가 없습니다.",
            Text::NoRowSelected => "선택된 항목이 없습니다.",
            Text::NoWorkspaceSelected => "선택된 작업공간이 없습니다.",
            Text::NoWorkspaceFound => "작업공간을 찾을 수 없습니다",
            Text::OpenedWorkspace => "작업공간 열림",
            Text::ProcessStatusUnavailable => "프로세스 상태를 확인할 수 없습니다.",
            Text::Project => "프로젝트",
            Text::ProjectShellTitle => "ProjectShell",
            Text::RunningApp => "실행중 앱",
            Text::RunningAppNotFound => "실행중 앱을 찾을 수 없습니다.",
            Text::RunningApps => "실행중 앱",
            Text::ResidentActive => "백그라운드 모드 활성화. Win+`로 ProjectShell을 엽니다.",
            Text::ResidentUnavailable => "백그라운드 모드를 사용할 수 없습니다.",
            Text::SavedSettings => "설정이 저장되었습니다.",
            Text::SearchHint => "작업공간 또는 앱 검색...",
            Text::SelectRunningOrWorkspace => "실행중 앱 또는 작업공간을 선택하세요",
            Text::SelectedApp => "선택된 앱",
            Text::SelectedProject => "선택된 프로젝트",
            Text::Settings => "설정",
            Text::Status => "상태",
            Text::Stopped => "중지",
            Text::Unknown => "알 수 없음",
            Text::WindowTitleMatch => "창/탭 제목 매칭",
            Text::Workspace => "작업공간",
            Text::WorkspaceAdded => "작업공간이 추가되었습니다.",
            Text::WorkspaceDeleted => "작업공간이 삭제되었습니다.",
            Text::WorkspaceNav => "작업공간 전환",
            Text::WorkspaceNotFound => "작업공간을 찾을 수 없습니다.",
        }
    }

    fn en(self) -> &'static str {
        match self {
            Text::Active => "Active",
            Text::Alias => "Alias",
            Text::AppNotFound => "App not found.",
            Text::Apps => "Apps",
            Text::AppPathMissing => "App path missing.",
            Text::AppProcessHint => "Process name, e.g. code.exe",
            Text::AppAdded => "App added.",
            Text::AppDeleted => "App deleted.",
            Text::Args => "Args",
            Text::Assigned => "Assigned",
            Text::AssignSelectedRunningApp => "Select a running app to assign.",
            Text::Back => "Back",
            Text::CtrlAssign => "Ctrl+A Assign",
            Text::Delete => "Delete",
            Text::DeleteProject => "Delete Project",
            Text::Description => "Description",
            Text::EnterFocus => "Enter Focus",
            Text::EnterLaunch => "Enter Launch",
            Text::EnterResume => "Enter Resume",
            Text::ExecutablePath => "Executable path",
            Text::Failed => "Failed",
            Text::Focused => "Focused",
            Text::FocusFailed => "Focus failed",
            Text::Help => "Help",
            Text::HelpShortcuts => "Shortcuts",
            Text::HelpUsage => "How to use",
            Text::HintFooter => {
                "Enter Focus/Resume | Ctrl+A Assign | Ctrl+O Open | Ctrl+, Settings"
            }
            Text::Language => "Language",
            Text::Launched => "Launched",
            Text::LaunchSetEmpty => "Launch set is empty.",
            Text::MissingPath => "Missing Path",
            Text::Name => "Name",
            Text::NewApp => "New App",
            Text::NewWorkspace => "New Workspace",
            Text::NoAppSelected => "No app selected.",
            Text::NoProjectSelected => "No project selected.",
            Text::NoRowSelected => "No row selected.",
            Text::NoWorkspaceSelected => "No workspace selected.",
            Text::NoWorkspaceFound => "No workspace found",
            Text::OpenedWorkspace => "Opened workspace",
            Text::ProcessStatusUnavailable => "Process status unavailable.",
            Text::Project => "Project",
            Text::ProjectShellTitle => "ProjectShell",
            Text::RunningApp => "Running App",
            Text::RunningAppNotFound => "Running app not found.",
            Text::RunningApps => "Running Apps",
            Text::ResidentActive => "Resident mode active. Press Win+` to show ProjectShell.",
            Text::ResidentUnavailable => "Resident mode unavailable.",
            Text::SavedSettings => "Settings saved.",
            Text::SearchHint => "Search workspace or app...",
            Text::SelectRunningOrWorkspace => "Select a running app or workspace row",
            Text::SelectedApp => "Selected App",
            Text::SelectedProject => "Selected Project",
            Text::Settings => "Settings",
            Text::Status => "Status",
            Text::Stopped => "Stopped",
            Text::Unknown => "Unknown",
            Text::WindowTitleMatch => "Window/tab title match",
            Text::Workspace => "Workspace",
            Text::WorkspaceAdded => "Workspace added.",
            Text::WorkspaceDeleted => "Workspace deleted.",
            Text::WorkspaceNav => "Workspace Nav",
            Text::WorkspaceNotFound => "Workspace not found.",
        }
    }
}
