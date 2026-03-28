#import <Cocoa/Cocoa.h>

// Tag constants for identifying menu actions.
// Should match the MenuAction enum in Rust.
enum {
    TAG_OPEN_WORKSPACE = 1,
    TAG_SAVE           = 2,
    TAG_LANG_EN        = 3,
    TAG_LANG_JA        = 4,
    TAG_ABOUT          = 5,
    TAG_SETTINGS       = 6,
    TAG_LANG_ZH_CN     = 7,
    TAG_LANG_ZH_TW     = 8,
    TAG_LANG_KO        = 9,
    TAG_LANG_PT        = 10,
    TAG_LANG_FR        = 11,
    TAG_LANG_DE        = 12,
    TAG_LANG_ES        = 13,
    TAG_LANG_IT        = 14,
    TAG_CHECK_UPDATES  = 15,
    TAG_RELEASE_NOTES  = 16,
};

// Global: Tag of the last selected menu action.
// Read by polling from Rust.
static volatile int g_last_action = 0;

@interface KatanaMenuTarget : NSObject
- (void)menuAction:(id)sender;
@end

@implementation KatanaMenuTarget
- (void)menuAction:(id)sender {
    NSMenuItem *item = (NSMenuItem *)sender;
    g_last_action = (int)[item tag];
}
@end

static KatanaMenuTarget *g_target = nil;

static NSMenu *g_file_menu = nil;
static NSMenuItem *g_open_workspace_item = nil;
static NSMenuItem *g_save_item = nil;
static NSMenu *g_settings_menu = nil;
static NSMenuItem *g_preferences_item = nil;
static NSMenu *g_language_menu = nil;
static NSMenuItem *g_about_item = nil;
static NSMenuItem *g_check_updates_item = nil;
static NSMenuItem *g_hide_item = nil;
static NSMenuItem *g_hide_others_item = nil;
static NSMenuItem *g_show_all_item = nil;
static NSMenuItem *g_quit_item = nil;
static NSMenu *g_help_menu = nil;
static NSMenuItem *g_release_notes_item = nil;

/// Called from Rust at the very start of main(), before eframe creates the window.
/// Must be called before the window server registers the process to ensure
/// the Dock label shows "KatanA" instead of the binary name "katana".
void katana_set_process_name(void) {
    [[NSProcessInfo processInfo] setProcessName:@"KatanA"];
}

/// Called from Rust: Builds the native menu bar.
void katana_setup_native_menu(void) {
    g_target = [[KatanaMenuTarget alloc] init];
    SEL action = @selector(menuAction:);

    // --- Application Menu ---
    NSMenu *appMenu = [[NSMenu alloc] initWithTitle:@"KatanA"];

    NSMenuItem *aboutItem = [[NSMenuItem alloc]
        initWithTitle:@"About KatanA"
        action:action
        keyEquivalent:@""];
    [aboutItem setTarget:g_target];
    [aboutItem setTag:TAG_ABOUT];
    [appMenu addItem:aboutItem];
    g_about_item = aboutItem;

    NSMenuItem *checkUpdatesItem = [[NSMenuItem alloc]
        initWithTitle:@"Check for Updates…"
        action:action
        keyEquivalent:@""];
    [checkUpdatesItem setTarget:g_target];
    [checkUpdatesItem setTag:TAG_CHECK_UPDATES];
    [appMenu addItem:checkUpdatesItem];
    g_check_updates_item = checkUpdatesItem;

    [appMenu addItem:[NSMenuItem separatorItem]];

    NSMenuItem *hideItem = [[NSMenuItem alloc]
        initWithTitle:@"Hide KatanA"
        action:@selector(hide:)
        keyEquivalent:@"h"];
    [appMenu addItem:hideItem];
    g_hide_item = hideItem;

    NSMenuItem *hideOthersItem = [[NSMenuItem alloc]
        initWithTitle:@"Hide Others"
        action:@selector(hideOtherApplications:)
        keyEquivalent:@"h"];
    [hideOthersItem setKeyEquivalentModifierMask:NSEventModifierFlagCommand | NSEventModifierFlagOption];
    [appMenu addItem:hideOthersItem];
    g_hide_others_item = hideOthersItem;

    NSMenuItem *showAllItem = [[NSMenuItem alloc]
        initWithTitle:@"Show All"
        action:@selector(unhideAllApplications:)
        keyEquivalent:@""];
    [appMenu addItem:showAllItem];
    g_show_all_item = showAllItem;

    [appMenu addItem:[NSMenuItem separatorItem]];

    NSMenuItem *quitItem = [[NSMenuItem alloc]
        initWithTitle:@"Quit KatanA"
        action:@selector(terminate:)
        keyEquivalent:@"q"];
    [appMenu addItem:quitItem];
    g_quit_item = quitItem;

    NSMenuItem *appMenuItem = [[NSMenuItem alloc] initWithTitle:@"" action:nil keyEquivalent:@""];
    [appMenuItem setSubmenu:appMenu];

    // --- File Menu ---
    NSMenu *fileMenu = [[NSMenu alloc] initWithTitle:@"File"];
    g_file_menu = fileMenu;

    NSMenuItem *openItem = [[NSMenuItem alloc]
        initWithTitle:@"Open Workspace…"
        action:action
        keyEquivalent:@"o"];
    [openItem setTarget:g_target];
    [openItem setTag:TAG_OPEN_WORKSPACE];
    [fileMenu addItem:openItem];
    g_open_workspace_item = openItem;

    [fileMenu addItem:[NSMenuItem separatorItem]];

    NSMenuItem *saveItem = [[NSMenuItem alloc]
        initWithTitle:@"Save"
        action:action
        keyEquivalent:@"s"];
    [saveItem setTarget:g_target];
    [saveItem setTag:TAG_SAVE];
    [fileMenu addItem:saveItem];
    g_save_item = saveItem;

    NSMenuItem *fileMenuItem = [[NSMenuItem alloc] initWithTitle:@"" action:nil keyEquivalent:@""];
    [fileMenuItem setSubmenu:fileMenu];

    // --- Settings > Language ---
    NSMenu *langMenu = [[NSMenu alloc] initWithTitle:@"Language"];
    g_language_menu = langMenu;

    NSMenuItem *enItem = [[NSMenuItem alloc]
        initWithTitle:@"English"
        action:action
        keyEquivalent:@""];
    [enItem setTarget:g_target];
    [enItem setTag:TAG_LANG_EN];
    [langMenu addItem:enItem];

    NSMenuItem *jaItem = [[NSMenuItem alloc]
        initWithTitle:@"日本語"
        action:action
        keyEquivalent:@""];
    [jaItem setTarget:g_target];
    [jaItem setTag:TAG_LANG_JA];
    [langMenu addItem:jaItem];

    NSMenuItem *zhCNItem = [[NSMenuItem alloc] initWithTitle:@"简体中文" action:action keyEquivalent:@""];
    [zhCNItem setTarget:g_target];
    [zhCNItem setTag:TAG_LANG_ZH_CN];
    [langMenu addItem:zhCNItem];

    NSMenuItem *zhTWItem = [[NSMenuItem alloc] initWithTitle:@"繁體中文" action:action keyEquivalent:@""];
    [zhTWItem setTarget:g_target];
    [zhTWItem setTag:TAG_LANG_ZH_TW];
    [langMenu addItem:zhTWItem];

    NSMenuItem *koItem = [[NSMenuItem alloc] initWithTitle:@"한국어" action:action keyEquivalent:@""];
    [koItem setTarget:g_target];
    [koItem setTag:TAG_LANG_KO];
    [langMenu addItem:koItem];

    NSMenuItem *ptItem = [[NSMenuItem alloc] initWithTitle:@"Português" action:action keyEquivalent:@""];
    [ptItem setTarget:g_target];
    [ptItem setTag:TAG_LANG_PT];
    [langMenu addItem:ptItem];

    NSMenuItem *frItem = [[NSMenuItem alloc] initWithTitle:@"Français" action:action keyEquivalent:@""];
    [frItem setTarget:g_target];
    [frItem setTag:TAG_LANG_FR];
    [langMenu addItem:frItem];

    NSMenuItem *deItem = [[NSMenuItem alloc] initWithTitle:@"Deutsch" action:action keyEquivalent:@""];
    [deItem setTarget:g_target];
    [deItem setTag:TAG_LANG_DE];
    [langMenu addItem:deItem];

    NSMenuItem *esItem = [[NSMenuItem alloc] initWithTitle:@"Español" action:action keyEquivalent:@""];
    [esItem setTarget:g_target];
    [esItem setTag:TAG_LANG_ES];
    [langMenu addItem:esItem];

    NSMenuItem *itItem = [[NSMenuItem alloc] initWithTitle:@"Italiano" action:action keyEquivalent:@""];
    [itItem setTarget:g_target];
    [itItem setTag:TAG_LANG_IT];
    [langMenu addItem:itItem];

    NSMenuItem *langMenuItem = [[NSMenuItem alloc] initWithTitle:@"Language" action:nil keyEquivalent:@""];
    [langMenuItem setSubmenu:langMenu];

    NSMenu *settingsMenu = [[NSMenu alloc] initWithTitle:@"Settings"];
    g_settings_menu = settingsMenu;

    NSMenuItem *prefsItem = [[NSMenuItem alloc]
        initWithTitle:@"Preferences…"
        action:action
        keyEquivalent:@","];
    [prefsItem setTarget:g_target];
    [prefsItem setTag:TAG_SETTINGS];
    [settingsMenu addItem:prefsItem];
    g_preferences_item = prefsItem;

    [settingsMenu addItem:[NSMenuItem separatorItem]];
    [settingsMenu addItem:langMenuItem];

    NSMenuItem *settingsMenuItem = [[NSMenuItem alloc] initWithTitle:@"" action:nil keyEquivalent:@""];
    [settingsMenuItem setSubmenu:settingsMenu];

    // --- Help Menu ---
    NSMenu *helpMenu = [[NSMenu alloc] initWithTitle:@"Help"];
    g_help_menu = helpMenu;
    
    NSMenuItem *releaseNotesItem = [[NSMenuItem alloc] 
        initWithTitle:@"Release Notes" 
        action:action 
        keyEquivalent:@""];
    [releaseNotesItem setTarget:g_target];
    [releaseNotesItem setTag:TAG_RELEASE_NOTES];
    [helpMenu addItem:releaseNotesItem];
    g_release_notes_item = releaseNotesItem;

    NSMenuItem *helpMenuItem = [[NSMenuItem alloc] initWithTitle:@"Help" action:nil keyEquivalent:@""];
    [helpMenuItem setSubmenu:helpMenu];

    // --- Build Main Menu ---
    NSMenu *mainMenu = [[NSMenu alloc] initWithTitle:@""];
    [NSApp setMainMenu:mainMenu];
    [mainMenu addItem:appMenuItem];
    [mainMenu addItem:fileMenuItem];
    [mainMenu addItem:settingsMenuItem];
    [mainMenu addItem:helpMenuItem];

    // Prevent macOS from auto-injecting the Spotlight Search box into our custom "Help" menu
    // by explicitly giving it a dummy, detached Help Menu.
    NSMenu *dummyHelpMenu = [[NSMenu alloc] initWithTitle:@"DummyHelp"];
    [NSApp setHelpMenu:dummyHelpMenu];
}

/// Called from Rust: Gets and resets the last menu action.
/// Return value: 0 = No action, otherwise = TAG_* constant.
int katana_poll_menu_action(void) {
    int action = g_last_action;
    g_last_action = 0;
    return action;
}

/// Called from Rust to dynamically update menu strings for i18n
void katana_update_menu_strings(
    const char* file, 
    const char* open_workspace, 
    const char* save, 
    const char* settings, 
    const char* preferences, 
    const char* language,
    const char* about,
    const char* quit,
    const char* hide,
    const char* hide_others,
    const char* show_all,
    const char* check_updates,
    const char* help,
    const char* release_notes
) {
    @autoreleasepool {
        if (g_file_menu && file) {
            [g_file_menu setTitle:[NSString stringWithUTF8String:file]];
        }
        if (g_open_workspace_item && open_workspace) {
            [g_open_workspace_item setTitle:[NSString stringWithUTF8String:open_workspace]];
        }
        if (g_save_item && save) {
            [g_save_item setTitle:[NSString stringWithUTF8String:save]];
        }
        if (g_settings_menu && settings) {
            [g_settings_menu setTitle:[NSString stringWithUTF8String:settings]];
        }
        if (g_preferences_item && preferences) {
            [g_preferences_item setTitle:[NSString stringWithUTF8String:preferences]];
        }
        if (g_language_menu && language) {
            [g_language_menu setTitle:[NSString stringWithUTF8String:language]];
        }
        if (g_about_item && about) {
            [g_about_item setTitle:[NSString stringWithUTF8String:about]];
        }
        if (g_check_updates_item && check_updates) {
            [g_check_updates_item setTitle:[NSString stringWithUTF8String:check_updates]];
        }
        if (g_quit_item && quit) {
            [g_quit_item setTitle:[NSString stringWithUTF8String:quit]];
        }
        if (g_hide_item && hide) {
            [g_hide_item setTitle:[NSString stringWithUTF8String:hide]];
        }
        if (g_hide_others_item && hide_others) {
            [g_hide_others_item setTitle:[NSString stringWithUTF8String:hide_others]];
        }
        if (g_show_all_item && show_all) {
            [g_show_all_item setTitle:[NSString stringWithUTF8String:show_all]];
        }
        if (g_help_menu && help) {
            [g_help_menu setTitle:[NSString stringWithUTF8String:help]];
        }
        if (g_release_notes_item && release_notes) {
            [g_release_notes_item setTitle:[NSString stringWithUTF8String:release_notes]];
        }
    }
}

static NSImage *g_app_icon = nil;

/// Called from Rust: Sets the application and dock icon from PNG bytes.
void katana_set_app_icon_png(const unsigned char *png_data, unsigned long png_len) {
    @autoreleasepool {
        NSData *data = [NSData dataWithBytes:png_data length:png_len];
        NSImage *image = [[NSImage alloc] initWithData:data];
        if (image) {
            g_app_icon = image;
            [NSApp setApplicationIconImage:image];
        }
    }
}

/// Called from Rust: Gets the current user locale from macOS.
void katana_get_mac_locale(char *buf, size_t max_len) {
    if (buf == NULL || max_len == 0) return;
    buf[0] = '\0';
    @autoreleasepool {
        NSArray *languages = [NSLocale preferredLanguages];
        if (languages != nil && languages.count > 0) {
            NSString *preferred = languages[0];
            const char *utf8 = [preferred UTF8String];
            if (utf8) {
                strncpy(buf, utf8, max_len - 1);
                buf[max_len - 1] = '\0';
            }
        }
    }
}
