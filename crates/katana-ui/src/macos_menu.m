#import <Cocoa/Cocoa.h>

// Tag constants for identifying menu actions.
// Should match the MenuAction enum in Rust.
enum {
    TAG_OPEN_WORKSPACE = 1,
    TAG_SAVE           = 2,
    TAG_LANG_EN        = 3,
    TAG_LANG_JA        = 4,
    TAG_ABOUT          = 5,
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

    [appMenu addItem:[NSMenuItem separatorItem]];

    NSMenuItem *hideItem = [[NSMenuItem alloc]
        initWithTitle:@"Hide KatanA"
        action:@selector(hide:)
        keyEquivalent:@"h"];
    [appMenu addItem:hideItem];

    NSMenuItem *hideOthersItem = [[NSMenuItem alloc]
        initWithTitle:@"Hide Others"
        action:@selector(hideOtherApplications:)
        keyEquivalent:@"h"];
    [hideOthersItem setKeyEquivalentModifierMask:NSEventModifierFlagCommand | NSEventModifierFlagOption];
    [appMenu addItem:hideOthersItem];

    NSMenuItem *showAllItem = [[NSMenuItem alloc]
        initWithTitle:@"Show All"
        action:@selector(unhideAllApplications:)
        keyEquivalent:@""];
    [appMenu addItem:showAllItem];

    [appMenu addItem:[NSMenuItem separatorItem]];

    NSMenuItem *quitItem = [[NSMenuItem alloc]
        initWithTitle:@"Quit KatanA"
        action:@selector(terminate:)
        keyEquivalent:@"q"];
    [appMenu addItem:quitItem];

    NSMenuItem *appMenuItem = [[NSMenuItem alloc] initWithTitle:@"" action:nil keyEquivalent:@""];
    [appMenuItem setSubmenu:appMenu];

    // --- File Menu ---
    NSMenu *fileMenu = [[NSMenu alloc] initWithTitle:@"File"];

    NSMenuItem *openItem = [[NSMenuItem alloc]
        initWithTitle:@"Open Workspace…"
        action:action
        keyEquivalent:@"o"];
    [openItem setTarget:g_target];
    [openItem setTag:TAG_OPEN_WORKSPACE];
    [fileMenu addItem:openItem];

    [fileMenu addItem:[NSMenuItem separatorItem]];

    NSMenuItem *saveItem = [[NSMenuItem alloc]
        initWithTitle:@"Save"
        action:action
        keyEquivalent:@"s"];
    [saveItem setTarget:g_target];
    [saveItem setTag:TAG_SAVE];
    [fileMenu addItem:saveItem];

    NSMenuItem *fileMenuItem = [[NSMenuItem alloc] initWithTitle:@"" action:nil keyEquivalent:@""];
    [fileMenuItem setSubmenu:fileMenu];

    // --- Settings > Language ---
    NSMenu *langMenu = [[NSMenu alloc] initWithTitle:@"Language"];

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

    NSMenuItem *langMenuItem = [[NSMenuItem alloc] initWithTitle:@"Language" action:nil keyEquivalent:@""];
    [langMenuItem setSubmenu:langMenu];

    NSMenu *settingsMenu = [[NSMenu alloc] initWithTitle:@"Settings"];
    [settingsMenu addItem:langMenuItem];

    NSMenuItem *settingsMenuItem = [[NSMenuItem alloc] initWithTitle:@"" action:nil keyEquivalent:@""];
    [settingsMenuItem setSubmenu:settingsMenu];

    // --- Build Main Menu ---
    NSMenu *mainMenu = [[NSMenu alloc] initWithTitle:@""];
    [NSApp setMainMenu:mainMenu];
    [mainMenu addItem:appMenuItem];
    [mainMenu addItem:fileMenuItem];
    [mainMenu addItem:settingsMenuItem];
}

/// Called from Rust: Gets and resets the last menu action.
/// Return value: 0 = No action, otherwise = TAG_* constant.
int katana_poll_menu_action(void) {
    int action = g_last_action;
    g_last_action = 0;
    return action;
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
