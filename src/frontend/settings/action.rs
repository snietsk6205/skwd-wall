#[derive(Debug, Clone, PartialEq)]
pub enum ActionId {
    SetThemeTarget(&'static str, bool),
    SetLockScreen(crate::contracts::settings::LockScreenTarget, bool),
    ChooseSddmTheme(Option<String>),
    RefreshAppThemes,
    SetAppTheme(u8, bool),
    AdoptAppTheme(u8),
    RefreshAppTheme(u8),
    CustomizeAppTheme(u8, &'static str),
    CopyAppThemeTemplate(u8),
    ClearCache,
    CaptureWeThumbnails,
    RecomputeColors,
    OptimizeImages,
    RefreshBackdrop,
    CopyLayerRule,
    AddPostCommand,
    RemovePostCommand(u16),
    AddFolder(crate::contracts::settings::FolderKind),
    ChooseFolder(crate::contracts::settings::FolderKind, Option<usize>),
    RemoveFolder(crate::contracts::settings::FolderKind, usize),
    UndoFolderRemove,
    NewProfile,
    DeleteProfile(u16),
    CopyProfileCommand(u16, bool),
    AddIntegration,
    RemoveIntegration(u16),
    ImportSemanticModel,
    AddSemanticModel,
    RemoveSemanticModel(u16),
    DeleteSemanticModel(u16),
    DeleteActiveSemanticModel,
    AddResolutionPreset,
    CreateResolutionPresetBand(&'static str),
    RemoveResolutionPreset(u16),
    OpenScheduleEditor,
    RefreshWeather,
    ResumeWeather,
    ReviewWeather(&'static str),
    FollowOutdoors(bool),
    OutdoorNext(String),
    OutdoorExclude {
        output: String,
        key: String,
    },
    OutdoorPin(String),
    OutdoorResume(String),
    OutdoorCredits(String),
    OpenThemeDesigner,
    ChooseRunningProcess,
    GenerateBugReport,
    ResetMotionFast,
    ResetMotionStandard,
    ResetMotionSlow,
    ResetKeybinds,
    SetNavKeys(crate::domain::input::NavKeys),
    /// Custom build: move a bar button group one place earlier (`true`) or later.
    MoveBarGroup(&'static str, bool),
    /// Custom build: put the bar's button groups back in upstream's order.
    ResetBarOrder,
    /// Custom build: show only the essential bar buttons (`true`) or all of them.
    BarPreset(bool),
    /// Custom build (M3): play this transition (a key or a combination name)
    /// in the live preview without choosing it.
    PreviewTransition(String),
    /// Custom build (M3): make this the transition that plays ("playlist" and
    /// "combo" included).
    UseTransition(String),
    /// Custom build (M3): star a transition, or unstar it.
    FavouriteTransition(&'static str),
    /// Custom build (M3): add a transition (or a combination name) to the
    /// playlist, or take it out.
    PlaylistToggle(String),
    /// Custom build (M3): playlist entry `index` in the live preview.
    PlaylistPreview(usize),
    /// Custom build (M3): move playlist entry `index` one place earlier (`true`) or later.
    PlaylistMove(usize, bool),
    PlaylistRemove(usize),
    PlaylistClear,
    PlaylistAddFavourites,
    /// Custom build (M3): swap the first and second transition of the combination.
    ComboSwap,
    /// R1: show the wallpaper selected in the picker behind Home.
    HomeUseSelected,
    /// R1: add, remove and reorder the cards on Home.
    EditHomeCards,
}

impl ActionId {
    pub const fn is_destructive(&self) -> bool {
        matches!(
            self,
            Self::ClearCache
                | Self::OptimizeImages
                | Self::ResetKeybinds
                | Self::DeleteSemanticModel(_)
                | Self::DeleteActiveSemanticModel
                | Self::DeleteProfile(_)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsFocus {
    Controls,
    /// R1: the top bar (groups, view switch, palette button).
    Bar,
    /// R1: an open group menu.
    Menu,
    /// R1: the cards on Home.
    Home,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsKey {
    FocusNext {
        backwards: bool,
    },
    CategoryNext {
        backwards: bool,
    },
    Search,
    Previous,
    Next,
    Up,
    Down,
    Activate,
    Cancel,
    /// R1: Ctrl+Z.
    Undo,
    /// R1: Shift+Left/Right (move a Home card in edit mode).
    Reorder {
        earlier: bool,
    },
    /// R1: the Delete key (remove a Home card in edit mode).
    Delete,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeatherReviewTab {
    #[default]
    Suggested,
    Approved,
    Library,
}

pub const WEATHER_REVIEW_TAB_COUNT: usize = 3;

#[derive(Debug, Clone)]
pub enum SettingsMsg {
    Input(String, String),
    ResolutionInput(String, bool, String),
    ResolutionLimit(String, bool),
    Toggle(String, bool),
    Commit,
    Pick(String, String),
    BackgroundPicker(String),
    BackgroundMode(String, String),
    BackgroundColor(String, String),
    BackgroundCommit(String),
    SelectSection(usize),
    LeaveLayoutStudio,
    FocusControl(usize),
    ToggleDetails(String, usize),
    ToggleBar(String, usize),
    ToggleFold(String, bool),
    SelectProfile(usize),
    ProfileFilter(String),
    EditFolder(String),
    FolderDraft(crate::contracts::settings::FolderKind, String),
    SearchOpen,
    #[allow(dead_code, reason = "kept for old picker commands")]
    SearchClose,
    SearchInput(String),
    OpenSearchResult(String, usize, usize),
    #[cfg_attr(not(test), allow(dead_code, reason = "kept for old picker commands"))]
    SelectTab(String),
    Key(SettingsKey),
    Run(ActionId),
    KeybindCapture(String, usize),
    KeybindCaptureCancel,
    KeybindCaptureApply,
    KeybindCaptureDefault,
    KeybindCaptureUnbind,
    KeybindCaptureClick(crate::domain::input::MouseButton),
    ProcessPickerClose,
    ProcessPickerSearch(String),
    ProcessPickerManualInput(String),
    ProcessPickerManualAdd,
    ProcessPickerScroll(f32),
    ProcessPickerAdd(String),
    ProcessPickerRemove(String),
    WeatherReviewClose,
    WeatherReviewTab(WeatherReviewTab),
    WeatherReviewSearch(String),
    WeatherReviewToggle(String),
    /// R1: Home, a page (index into `nav::PAGES`), a page row, or back.
    OpenHome,
    OpenPage(usize),
    OpenPageRow(usize, usize),
    Back,
    /// R1: group menus: click toggles, the pointer opens and leaves.
    MenuClick(usize),
    MenuHover(usize),
    MenuLeave,
    /// R1: Simple / Advanced view.
    ToggleView,
    /// R1: show every row of this page (Simple view hides some).
    ShowAllRows,
    /// R1: put a setting back to its default.
    ResetPath(String),
    /// R1: open or close a row's longer explanation.
    ToggleMore(String),
    /// R1: command palette.
    PaletteOpen,
    PaletteClose,
    PaletteInput(String),
    PaletteSubmit,
    PaletteRun(usize),
    PaletteHover(usize),
    /// Change result `index` in place (`Some(value)` picks that choice).
    PaletteChange(usize, Option<String>),
    /// R1: undo the last change (toast button or Ctrl+Z).
    Undo,
    /// R1: Home cards.
    EditCards(bool),
    CardMove(usize, bool),
    CardRemove(usize),
    CardAdd(usize),
    CardHover(Option<usize>),
    HomePause,
    /// R1b: the pointer entered (`true`) or left a card (row, choice; 0 for
    /// a card that is a row of its own).
    TileHover(usize, usize, bool),
    /// R1b: where the pointer is on a card, from its left edge (only sent
    /// while the preview follows the cursor).
    CursorMoved(usize, usize, f32),
    /// R1b: close the big preview.
    BigPreviewClose,
}
