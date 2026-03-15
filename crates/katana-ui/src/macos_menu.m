#import <Cocoa/Cocoa.h>

// Tag constants for identifying menu actions.
// Should match the MenuAction enum in Rust.
enum {
    TAG_OPEN_WORKSPACE = 1,
    TAG_SAVE           = 2,
    TAG_LANG_EN        = 3,
    TAG_LANG_JA        = 4,
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

/// Called from Rust: Builds the native menu bar.
void katana_setup_native_menu(void) {
    g_target = [[KatanaMenuTarget alloc] init];
    SEL action = @selector(menuAction:);

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

    // --- Settings > Language メニュー ---
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

    // --- Add to Main Menu ---
    NSMenu *mainMenu = [NSApp mainMenu];
    if (!mainMenu) {
        mainMenu = [[NSMenu alloc] initWithTitle:@""];
        [NSApp setMainMenu:mainMenu];
    }
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
