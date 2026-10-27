nchat
=====

| **Linux** | **Mac** |
|-----------|---------|
| [![Linux](https://github.com/d99kris/nchat/workflows/Linux/badge.svg)](https://github.com/d99kris/nchat/actions?query=workflow%3ALinux) | [![macOS](https://github.com/d99kris/nchat/workflows/macOS/badge.svg)](https://github.com/d99kris/nchat/actions?query=workflow%3AmacOS) |

nchat is a terminal-based multi-protocol messaging client for Linux and macOS
with support for Telegram, WhatsApp and Signal.

![screenshot nchat](/doc/screenshot-nchat.png)

Features
--------
- Send and receive markdown formatted messages
- Reply, delete, edit and forward messages
- Mention users in group chats
- Send and display message reactions
- View and save media (documents, photos, videos)
- Show message read and edited indicators
- Show user status (online, away, typing)
- Jump to next unread chat
- Archive, pin and delete chats
- Search messages within a chat
- List dialogs for chats, contacts, emojis, files
- Message history cache with text export
- Desktop notifications for new messages
- Toggle textized vs. graphical emojis
- Customizable [themes](https://github.com/d99kris/nchat/wiki/Themes) and key bindings


Usage
=====
Usage:

    nchat [OPTION]

Command-line Options:

    -d, --confdir <DIR>    use a different directory than ~/.config/nchat
    -e, --verbose          enable verbose logging
    -ee, --extra-verbose   enable extra verbose logging
    -h, --help             display this help and exit
    -k, --keydump          key code dump mode
    -m, --devmode          developer mode
    -r, --remove           remove chat protocol account
    -s, --setup            set up chat protocol account
    -v, --version          output version information and exit
    -x, --export <DIR>     export message cache to specified dir

Interactive Commands:

    PageDn      history next page
    PageUp      history previous page
    Tab         next pane (list, history, entry)
    Sh-Tab      previous pane
    Esc         focus text entry
    Esc Esc     clear input (twice within 0.5 s, in the entry; undo with Ctrl-c)
    Enter       send message (see `enter_sends`)
    Alt-Enter   insert linebreak
    Ctrl-f      jump to unread chat
    Ctrl-g      toggle show help bar
    Ctrl-l      toggle show contact list
    Ctrl-n      goto chat
    Ctrl-p      toggle show top bar
    Ctrl-q      quit
    Ctrl-s      insert emoji
    Ctrl-t      send file
    Ctrl-x      send message
    Ctrl-y      toggle show emojis
    KeyUp       select message
    Alt-@       insert mention
    Alt-a       archive current chat
    Alt-d       delete/leave current chat
    Alt-e       external editor compose
    Alt-g       send a sticker (Telegram)
    Alt-i       auto-compose reply
    Alt-j       next chat
    Alt-k       previous chat
    Alt-n       search contacts
    Alt-p       pin/unpin current chat
    Alt-t       external telephone call
    Alt-/       find in chat
    Alt-?       find next in chat
    Alt-$       external spell check
    Alt-,       decrease contact list width
    Alt-.       increase contact list width

Interactive Commands for Selected Message:

    Ctrl-d      delete selected message (`x` in history focus if `vim_keys=1`)
    Ctrl-r      download attached file
    Ctrl-v      open/view attached file
    Ctrl-x      send reply to selected message
    Ctrl-z      edit selected message
    Alt-c       copy selected message to clipboard
    Alt-u       open link
    Alt-q       jump to quoted/replied message
    Alt-r       forward selected message
    Alt-s       add/remove reaction on selected message
    Alt-w       external message viewer

Interactive Commands for Text Input:

    Ctrl-a      move cursor to start of line
    Ctrl-c      clear input buffer
    Ctrl-e      move cursor to end of line
    Ctrl-k      delete from cursor to end of line
    Ctrl-u      delete from cursor to start of line
    Alt-Left    move cursor backward one word
    Alt-Right   move cursor forward one word
    Alt-Backsp  delete previous word
    Alt-Delete  delete next word
    Alt-Tab     insert tab (four spaces)
    Alt-c       copy input buffer to clipboard (if no message selected)
    Alt-v       paste into input buffer from clipboard
    Alt-x       cut input buffer to clipboard

Focus and Navigation:

nchat has three panes: chat list, message history and text entry. Focus starts
in the entry. Tab / Sh-Tab cycle the focus, Esc returns to the entry, and
the focused pane is highlighted (see `focus_*` and `dim_attr` in `color.conf`).
In list or history focus, typing a letter that is not a navigation key jumps to
the entry. Up/Down/Home/End navigate in both panes, Enter opens the selected
chat (list) or replies to the selected message (history).

With `vim_keys=1` (default in `ui.conf`) the following keys are also available
in list and history focus:

    j / k       down / up
    gg / G      first / last
    Ctrl-d/u    half page down / up
    /           search (filter in list, find in history)
    i           focus text entry
    l           list: open chat
    h           history: focus list
    u           list: toggle unread-only
    z           list: expand all groups if any collapsed, otherwise
                collapse the group of the selected chat
    n / b       list: next / previous folder tab
    n           history: find next
    r           history: reply to selected message
    e           history: edit selected message
    y           history: copy selected message
    x           history: delete selected message

With `vim_keys=1` Ctrl-w followed by h / j / k / l also moves between panes from any
focus, including the text entry: h list, l history (from the list), j entry (from
history), k history (from the entry). Ctrl-w a opens the contact dialog (same as
Alt-n), where a Telegram `@username` or phone number can be typed to open a chat.
Any other key after Ctrl-w is ignored.

With `vim_keys=1` Ctrl-d / Ctrl-u are half-page keys in list and history focus,
so use `x` to delete a message there. With `vim_keys=0` these letters type into
the entry and Ctrl-d deletes the selected message again.

**Custom navigation keys (nav.conf)**

The vim keys above are built-in defaults, active without any file and only with
`vim_keys=1`. To change them create `~/.config/nchat/nav.conf` (read at startup;
rejected lines are logged as `nav.conf line N: ...` and skipped). Bindings written
in `nav.conf` are active regardless of `vim_keys`.

    # comment lines start with #
    map   <focus> <keys> <action>
    unmap <focus> <keys>
    mapclear

- `focus`: `list`, `history`, `entry` or `all`, comma-separated to combine (for
  example `list,history`). Plain keys mapped in `entry` or `all` no longer type into
  the text entry.
- `keys`: literal characters or a sequence such as `gg`, plus `<C-x>` (Ctrl),
  `<Space>`, `<CR>`, `<Tab>`, `<Esc>`, `<lt>` (for `<`) and `<Pane>` (the pane prefix,
  Ctrl-w by default, `pane_prefix` in `key.conf`). Keys cannot contain spaces.
- `action`: `up down first last halfup halfdown open reply edit copy delete search
  findnext unreadonly togglegroup foldernext folderprev selectcontact focusentry
  focuslist focushistory focusnext focusprev none` (`none` disables a key).
- Later lines override earlier ones and the built-in defaults. `unmap` removes a
  binding, `mapclear` removes all defaults.
- If a key is bound on its own and also starts a longer sequence, the single key
  wins, so `unmap` it first to use the sequence.
- Up/Down/Home/End/Enter and Tab/Sh-Tab/Esc are not mapped here, set them in `key.conf`.
- The help line shown in list and history focus follows these bindings.
- While a prefix or multi-key sequence is pending (e.g. after `Ctrl-w`), the help line lists the keys that can follow.

Example:

    map all <Pane>a selectcontact
    map list,history <C-n> down
    unmap history x
    map list zz first

The chat list shows two lines per chat (name and last message preview) with
chats grouped under a "Pinned" header and one header per account (shown when
there are two or more groups). Groups can be collapsed with `z`, `/` filters
chats by name (case-insensitive) and `u` shows only chats with unread messages.
Telegram chat folders show as tabs above the chat list; `folder_next` /
`folder_prev` (default shift-right / shift-left) switch between All and the folders.
The text entry shows a title line while composing, replying or editing, and
grows with its content up to half the screen height.

Alt-Enter inserts a linebreak. With the default `linefeed_on_enter=1` ncurses
turns the CR sent by Alt-Enter into LF, so the `newline` key defaults to
`\33\12` (ESC LF). If `linefeed_on_enter=0`, set `newline=\33\15` (ESC CR) in
`key.conf` instead. Most terminals, including Ghostty, send Alt-Enter as ESC CR.
Use `nchat -k` to verify the codes your terminal sends. nchat sets the
escape delay to 25 ms, so Esc responds quickly and Alt-combinations still work.

Message layout and theme:

- In the chat list the chat name is bold and the last message preview is gray
  (`list_name_attr`, `list_preview_attr`, `list_preview_color_*` in `color.conf`).
- In the message history the sender name is underlined, over the text only
  (`history_name_attr` in `color.conf`).
- With `sent_align_right=1` in `ui.conf` your own messages are shown on the right
  and other people's on the left, both limited to 75% of the pane width. Default
  is off (everything left-aligned).


Inline Images
=============
Photos and stickers can be shown inline in the message history (and sticker
previews in the Telegram sticker picker, Alt-g / Opt-g). nchat draws them with
the kitty graphics protocol, so it needs a terminal that supports it, and it
needs a tool to convert the downloaded file to PNG. Nothing else has to be
configured on the nchat side, `inline_images=auto` is the default.

**1. Terminal**

[Ghostty](https://ghostty.org) and [kitty](https://sw.kovidgoyal.net/kitty/)
are supported. With `inline_images=auto` nchat detects them through the
`GHOSTTY_RESOURCES_DIR` / `KITTY_WINDOW_ID` environment variables or a `TERM`
containing `ghostty` / `kitty`. If you set `term = "xterm-256color"` in the
Ghostty config, only the environment variable identifies the terminal, and it is
missing when the shell was not started by Ghostty (for example a tmux server
started elsewhere, or ssh). In that case set `inline_images=on` in `ui.conf`.
The terminal must also report at least 256 colors.

To use Alt/Opt shortcuts such as Alt-g on macOS, enable it in the Ghostty config
(`~/Library/Application Support/com.mitchellh.ghostty/config.ghostty`):

    macos-option-as-alt = true

**2. tmux (optional)**

Inside tmux nchat wraps the graphics in passthrough sequences, which tmux only
forwards when this is set in `~/.tmux.conf` (`on` or `all` both work):

    set -g allow-passthrough on

Apply it to a running server with `tmux set -g allow-passthrough on`. Without it
nchat keeps the plain text lines and logs `inline images disabled: tmux
allow-passthrough is ...`.

**3. Image conversion tools**

nchat tries these in order and uses the first that works. Install at least one:

    macOS:    sips is built in; for webp and video stickers also
              brew install imagemagick webp ffmpeg
    Ubuntu:   sudo apt install imagemagick webp ffmpeg
    Arch:     sudo pacman -S imagemagick libwebp ffmpeg

`magick` handles photos and webp stickers, `dwebp` handles webp, and `ffmpeg`
is required for video (`.webm`) stickers. Animated stickers are shown as a still
first frame, and Lottie (`.tgs`) stickers keep the text line. Converted images are
cached in `~/.config/nchat/images`.

**4. Related settings in ~/.config/nchat/ui.conf**

    inline_images=auto
    inline_image_max_rows=10
    inline_sticker_max_rows=6

See the descriptions of `inline_images`, `inline_image_max_rows` and
`inline_sticker_max_rows` below.

**Troubleshooting**

- Only text lines like `[Photo]` are shown: check `inline_images` is not `off`,
  that you run Ghostty or kitty (try `inline_images=on`), that tmux has
  `allow-passthrough` set, and that one of `magick`, `sips`, `dwebp`, `ffmpeg` is
  installed. The reason is logged in `~/.config/nchat/log.txt` (start nchat with
  `-e` for verbose logging, which also logs when no supported terminal is detected).
- Photos and stickers are downloaded when they become visible, so they can appear
  a moment after scrolling to them.
- Other terminals (Terminal.app, iTerm2, Alacritty, ...) are not supported and
  always fall back to text lines.


Supported Platforms
===================
nchat is developed and tested on Linux and macOS. Current version has been
tested on:

- macOS Sequoia 15.5
- Ubuntu 24.04 LTS


Install using Package Manager
=============================

macOS
-----
**Build / Install Stable Release using Brew**

    brew tap d99kris/nchat
    brew install nchat

Optionally one can disable protocols using `--without-whatsapp` and
`--without-telegram`, for example:

    brew install nchat --without-telegram


Arch Linux
----------
**Build / Install Latest Git**

    yay -S nchat-git

**Build / Install Stable Release**

    yay -S nchat

**Install Pre-built Binary**

    yay -S nchat-bin


Install Pre-built Binary
========================
Experimental support for Linux (glibc >= 2.28 or musl) x86_64 and arm64, as
well as macOS arm64.

**Install**

    curl -fsSL https://raw.githubusercontent.com/d99kris/nchat/master/utils/install.sh | bash

**Uninstall**

    curl -fsSL https://raw.githubusercontent.com/d99kris/nchat/master/utils/uninstall.sh | bash

**Install Location**

Installs to `~/.local/{bin,share}`. Override via the `NCHAT_PREFIX` environment
variable, e.g.:

    export NCHAT_PREFIX=/usr/local


Build from Source
=================
nchat consists of a large code-base (mainly the Telegram library tdlib), so be
prepared for a relatively long first build time.

**Get Source**

    git clone https://github.com/d99kris/nchat && cd nchat

Using make.sh script
--------------------
If using macOS, Alpine, Arch, Fedora, Gentoo, Raspbian, Ubuntu or Void, one
can use the `make.sh` script provided.

**Dependencies**

    ./make.sh deps

**Build / Install**

    ./make.sh build && ./make.sh install

**Install Location**

Installs to a system prefix such as `/usr/local`. Override via the
`NCHAT_PREFIX` environment variable (a user-writable prefix installs without
sudo/doas), e.g.:

    export NCHAT_PREFIX=~/.local

Manually
--------
**Dependencies**

macOS

    brew install gperf cmake openssl ncurses ccache readline help2man sqlite libmagic go

Arch

    sudo pacman -S ccache cmake file go gperf help2man ncurses openssl readline sqlite zlib base-devel

Debian-based (Ubuntu, Raspbian, etc)

    sudo apt install ccache cmake build-essential gperf help2man libreadline-dev libssl-dev libncurses-dev libncursesw5-dev ncurses-doc zlib1g-dev libsqlite3-dev libmagic-dev golang

Fedora

    sudo dnf install git cmake clang golang ccache file-devel file-libs gperf readline-devel openssl-devel ncurses-devel sqlite-devel zlib-devel

Gentoo

    sudo emerge -n dev-util/cmake dev-util/ccache dev-util/gperf sys-apps/help2man sys-libs/readline dev-libs/openssl sys-libs/ncurses sys-libs/zlib dev-db/sqlite sys-apps/file dev-lang/go

Void

    sudo xbps-install base-devel go ccache cmake gperf help2man libmagick-devel readline-devel sqlite-devel file-devel openssl-devel

**Extra Dependencies**

For Wayland-based systems install `wl-clipboard` to enable clipboard
functionality.

To support pasting images directly from clipboard `libpng-dev` is needed. On
X11 systems `libxcb1-dev` and `libx11-dev` are needed as well.

To show photos and stickers inline, install one or more image conversion tools
at runtime (`sips` is built into macOS, otherwise `imagemagick`, `webp` and
`ffmpeg`). See [Inline Images](#inline-images).

**Build**

    mkdir -p build && cd build && cmake .. && make -s

**Install**

    sudo make install

Using vcpkg
-----------
As an alternative to system packages, the C/C++ libraries listed in
`vcpkg.json` (ncurses, sqlite3, zlib, libpng, libmagic) can be managed by
[vcpkg](https://vcpkg.io) in manifest mode. This is optional - the
system-package build above remains the default and is unaffected.

**Prerequisites**

Install the vcpkg tool and clone its registry (the Fedora vcpkg package ships
only the tool, so the registry must be cloned separately):

    git clone https://github.com/microsoft/vcpkg.git ~/.local/share/vcpkg

If using a different location, set `VCPKG_ROOT` accordingly. The following
remain system dependencies even with vcpkg - OpenSSL (used by tdlib, present
on most Linux distributions, `brew install openssl` on macOS), Go (>= 1.23),
gperf, cmake and a C++ compiler. Building some ports additionally requires
autotools and perl, for example on Fedora:

    sudo dnf install autoconf autoconf-archive automake libtool perl-core openssl-devel golang gperf cmake

**Build / Install**

    USE_VCPKG=1 ./make.sh build && USE_VCPKG=1 ./make.sh install

Or manually with cmake:

    cmake -B build -DCMAKE_TOOLCHAIN_FILE=$VCPKG_ROOT/scripts/buildsystems/vcpkg.cmake && cmake --build build -s

On first configure vcpkg downloads and builds the manifest dependencies into
`build/vcpkg_installed`, which takes some extra time and is cached for later
builds.

Advanced Build Options
----------------------
By default nchat requires ~3.5GB RAM to build using G++ and ~1.5GB RAM with
clang++, but it is possible to reduce the memory needed,
see [Building on Low Memory Systems](/doc/LOWMEMORY.md).

When building from source, all features except Signal support are enabled by
default. To enable Signal, see [Signal](/doc/SIGNAL.md); for other feature
flags, see [Feature Flags](/doc/FLAGS.md).


Getting Started
===============
In order to configure / setup an account one needs to run nchat in setup mode:

    nchat --setup

The setup mode prompts for phone number, which shall be entered with country
code. Example:

    $ nchat --setup
    Protocols:
    0. Dummy
    1. Telegram
    2. WhatsAppMd
    3. Signal
    4. Exit setup
    Select protocol (4): 1
    Enter phone number (ex. +6511111111): +6511111111

    Open Telegram on your phone, go to Settings -> Devices
    and click Link Desktop Device and scan the QR code.
    ...
    Succesfully set up profile Telegram_+6511111111

By default, nchat uses QR code authentication. The QR code should be scanned
using the official app on the primary device. To use authentication code
instead, set the `USE_PAIRING_CODE` environment variable (see FAQ below).

If unsure of what phone number to enter, open the official app on the phone
and look for the phone number under Settings or Profile section, and use
the number displayed there (omitting spaces, so for the below screenshot
the number to enter is `+6511111111`).

![screenshot telegram phone](/doc/screenshot-phone.png)

Once the setup process is completed, the main UI of nchat will be loaded.

In order to set up multiple protocols/profiles, exit nchat and perform the
setup step again. To remove a protocol account, use `nchat --remove`.

Telegram forum groups (topics) appear as one chat for the group, which is the
General topic, plus one chat per topic named `Group › Topic`. Topic chats can
be read and written, but creating, renaming, closing and pinning topics is not
supported.

Known limitations of forum topics:
- Deleting a topic chat only removes it from nchat, it reappears after restart.
- Archiving a forum hides its topic chats and un-archiving restores them, but
  the group chat itself only reappears after restarting nchat.
- Leaving a forum, or a group becoming a forum while nchat is running, is only
  reflected after restarting nchat.


Troubleshooting
===============
Refer to [Debugging](/doc/DEBUGGING.md) for details.


Telegram Group
==============
A Telegram group [https://t.me/nchatusers](https://t.me/nchatusers) is
available for users to discuss nchat usage and related topics.


Security
========
User data is stored locally in `~/.config/nchat`. Default file permissions
only allow user access, but anyone who can gain access to a user's private
files can also access the user's personal nchat data. To protect against
the most simple attack vectors it may be suitable to use disk encryption and
to ensure `~/.config/nchat` is not backed up unencrypted.


Configuration
=============
The following configuration files (listed with current default values) can be
used to configure nchat.

~/.config/nchat/app.conf
------------------------
This configuration file holds general application settings. Default content:

    assert_abort=0
    attachment_prefetch=1
    attachment_send_type=1
    cache_enabled=1
    cache_read_only=0
    clipboard_copy_command=
    clipboard_has_image_command=
    clipboard_paste_command=
    clipboard_paste_image_command=
    coredump_enabled=0
    downloads_dir=
    emoji_list_all=0
    link_send_preview=1
    logdump_enabled=0
    mentions_quoted=1
    message_delete=1
    proxy_host=
    proxy_pass=
    proxy_port=
    proxy_user=
    timestamp_iso=0
    use_pairing_code=0
    use_qr_terminal=0
    version_used=

### assert_abort

Specifies whether to abort execution (crash) if assertions fail. Primarily
intended for debugging.

### attachment_send_type

Specifies how attachments are sent:

    0 = send all attachments as document type (typically preserves file content)
    1 = detect file type (audio, video, image, document) and send as that type <- default
    2 = like 1, but when sending just a file (no text or quote), webp files are
        sent as stickers and mp4/m4v files as borderless auto-playing "GIFs"

### attachment_prefetch

Specifies level of attachment prefetching:

    0 = no prefetch (download upon open/save)
    1 = selected (download upon message selection) <- default
    2 = all (download when message is received)

### cache_enabled

Specifies whether to enable cache functionality.

### cache_read_only

Specifies whether to access cache read-only. Primarily intended for debugging.

### clipboard_*_command

Specifies custom clipboard commands to be used instead of system clipboard.
Refer to [Clipboard](/doc/CLIPBOARD.md) for details.

### coredump_enabled

Specifies whether to enable core dumps on application crash.

### downloads_dir

Specifies a custom downloads directory path to save attachments to. If not
specified, the default dir is `~/Downloads` if exists, otherwise `~`.

### emoji_list_all

Specifies whether the emoji dialog should list all emojis, it is otherwise
restricted to listing emojis that renders properly in common terminals.

### link_send_preview

Specifies whether to enable preview for links in messages sent (Telegram only).

### logdump_enabled

Specifies whether to dump warning and error log messages to stdout upon exit.

### mentions_quoted

Specifies whether to use bracket quoting for display-name mentions with spaces.

### message_delete

Specifies handling of message deletion by other users (WhatsApp and Signal):

    1 = erase message <- default
    2 = replace message text with `[This message was deleted]`
    3 = prefix message text with `[This message was deleted]`

Messages deleted by oneself are always erased locally regardless of this
setting. Deleting messages already marked deleted erases them locally.

### proxy_

SOCKS5 proxy server details. To enable proxy usage the parameters `host` and
`port` are required, while `user` and `pass` are optional (depending on the
SOCKS server). Note: In order to use a proxy while setting up nchat the first
time, it is recommended to first run nchat without arguments (`nchat`) for its
config dir to be created, and then edit proxy settings in
`~/.config/nchat/app.conf` as needed, before running `nchat -s` to setup an
account.

### timestamp_iso

Specifies whether to use ISO-style timestamps (`YYYY-MM-DD HH:MM`) in the UI
and at export of chat history. By default nchat uses a dynamic "human-friendly"
format:

- `HH:MM` for timestamps on same date as today, e.g. `19:00`
- `DAY HH:MM` for timestamps in the last week, e.g. `Mon 19:00`
- `DD MMM HH:MM` for timestamps in the current year, e.g. `14 Nov 19:00`
- `DD MMM YYYY HH:MM` for timestamps in non-current year, e.g. `14 Nov 2022 19:00`
- `DD MMM YYYY HH:MM` for timestamps during export, e.g. `14 Nov 2022 19:00`

### use_pairing_code

Stores the environment variable flag `USE_PAIRING_CODE` if set during setup.
It specifies whether to use pairing code / authentication code instead of
QR code (Telegram/WhatsApp).

### use_qr_terminal

Stores the environment variable flag `USE_QR_TERMINAL` if set during setup.
It specifies whether to display QR code in the terminal, disabling detection
of GUI capability for displaying images.

### version_used

For internal use only. Stores version used for last successful execution. May
be used for debugging startup crash / hang regressions.

~/.config/nchat/ui.conf
-----------------------
This configuration file holds general user interface settings. Default content:

    attachment_indicator=📎
    attachment_open_command=
    away_status_indication=0
    auto_compose_command=
    auto_compose_enabled=0
    auto_compose_history_count=25
    auto_select_chat_timeout_sec=1
    call_command=
    chat_picker_sorted_alphabetically=0
    confirm_archiving=1
    confirm_deletion=1
    confirm_send_pasted_image=1
    delivered_indicator=○
    desktop_notify_active_current=0
    desktop_notify_active_noncurrent=1
    desktop_notify_command=
    desktop_notify_connectivity=1
    desktop_notify_enabled=0
    desktop_notify_inactive=1
    downloadable_indicator=+
    edited_indicator=✎
    emoji_enabled=1
    enter_sends=1
    entry_height=4
    failed_indicator=✗
    file_picker_command=
    file_picker_persist_dir=1
    help_enabled=1
    help_prefix_ctrl=^
    help_prefix_meta=M-
    home_fetch_all=0
    image_cache_days=30
    inline_image_max_rows=10
    inline_images=auto
    inline_sticker_max_rows=6
    linefeed_on_enter=1
    link_open_command=
    list_enabled=1
    list_width=30
    listdialog_show_filter=1
    mark_read_any_chat=0
    mark_read_on_view=1
    mark_read_when_inactive=0
    message_edit_command=
    message_open_command=
    muted_indicate_unread=1
    muted_notify_unread=0
    muted_position_by_timestamp=1
    notify_every_unread=1
    online_status_share=1
    online_status_dynamic=1
    phone_number_indicator=
    pinned_indicator=⚲
    proxy_indicator=🔒
    read_indicator=✓
    reactions_enabled=1
    sent_align_right=0
    sent_indicator=·
    spell_check_command=
    status_broadcast=1
    status_enabled=1
    syncing_indicator=⇄
    tab_size=4
    terminal_bell_active=0
    terminal_bell_inactive=1
    terminal_title=
    top_enabled=1
    top_show_version=0
    transfer_send_caption=1
    typing_status_share=1
    undo_clear_input=1
    unread_indicator=*
    vim_keys=1

### attachment_indicator

Specifies text to prefix attachment filenames in message view.

### attachment_open_command

Specifies a custom command to use for opening/viewing attachments. The
command shall include `%1` which will be replaced by the filename
to open. If not specified, the following default commands are used:

Linux: `xdg-open >/dev/null 2>&1 '%1' &`

macOS: `open '%1' &`

Note: Omit the trailing `&` for commands taking over the terminal, for
example `w3m -o confirm_qq=false '%1'` and `see '%1'`.

### away_status_indication

Specifies whether to indicate away status in the top bar while sharing away
status with other users. I.e. the status will read `Away` instead of `Online`
when the terminal is inactive (assuming `online_status_share=1` and
`online_status_dynamic=1`).


### auto_compose_command

Specifies a custom command to use to auto-compose message reply. The
command shall include `%1` which will be replaced by the path of a temporary
text file with message history. If not specified, nchat extracts its bundled
[compose](/src/compose) script to a temporary location and uses the following
default command:

    python3 '(tempdir)/compose' -c '%1'

Refer to [Auto-Compose](/doc/AUTOCOMPOSE.md) for details.

### auto_compose_enabled

Specifies whether to enable using auto-compose. The reason for not enabling
it by default is that the `compose` tool may use third-party services (if API
key is set up in the shell environment) that can incur costs.

### auto_compose_history_count

Specifies the maximum number of chat messages up to latest (or current
selected) to pass to the auto-compose tool.

### auto_select_chat_timeout_sec

Specifies a timeout after receiving previous history sync message (with
timestamp before the application went online) for automatically setting
current chat. If disabled (value 0) current chat will only be set upon
any user input, or receiving a message with timestamp after application
went online (connected to server).

### call_command

Specifies a custom command to use for starting a call using an external
tool. The command shall include `%1` which will be replaced by the phone
number of the contact. If not specified, the following default commands
are used:

Linux: `xdg-open >/dev/null 2>&1 'tel://%1' &`

macOS: `open 'tel://%1' &`

### chat_picker_sorted_alphabetically

Specifies whether the chat selection dialog (used when forwarding message)
should be sorted alphabetically. If not, its order follows the main chat
list order.

### confirm_archiving

Specifies whether to prompt the user for confirmation when archiving a chat.

### confirm_deletion

Specifies whether to prompt the user for confirmation when deleting a message
or a chat.

### confirm_send_pasted_image

Specifies whether to prompt the user for confirmation when pasting an image
to a chat.

### delivered_indicator

Specifies text to indicate a sent message has been delivered to the receiver but
not yet read.

### desktop_notify_active_current

Specifies whether new message in current chat shall trigger desktop
notification when nchat terminal window is active.

### desktop_notify_active_noncurrent

Specifies whether new message in non-current chat shall trigger desktop
notification when nchat terminal window is active.

### desktop_notify_command

Specifies a custom command to use for desktop notifications. The command may
include `%1` (will be replaced by `sender name` or `group name - sender name`)
and `%2` (will be replaced by `message text`) enclosed in single quotes (to
prevent shell injection). Default command used, if not specified:

Linux: `notify-send 'nchat' '%1: %2'`

macOS: `osascript -e 'display notification "%1: %2" with title "nchat"'`

### desktop_notify_connectivity

Specifies whether change of connectivity status (going offline or back online)
shall trigger desktop notification.

### desktop_notify_enabled

Specifies whether desktop notifications are permitted. Simply enabling this
parameter will not display notifications, unless other `desktop_notify_*`
parameters are enabled.

### desktop_notify_inactive

Specifies whether new message shall trigger desktop notification when nchat
terminal window is inactive.

### downloadable_indicator

Specifies text to suffix attachment filenames in message view for attachments
not yet downloaded. This is only shown for `attachment_prefetch` < 2.

### edited_indicator

Specifies text to suffix message header in message view for messages that have
been edited.

### emoji_enabled

Specifies whether to display emojis. Controlled by Ctrl-y in run-time.

### enter_sends

Specifies whether the Enter key (`send_enter` in `key.conf`) sends the message.
Alt-Enter (`newline`) then inserts a linebreak. If disabled, Enter inserts a
linebreak and messages are sent using `send_msg` (Ctrl-x).

### entry_height

Specifies minimum height of text entry area. The entry grows with its content
up to half of the screen height.

### failed_indicator

Specifies text to suffix attachment filenames in message view for failed
downloads.

### file_picker_command

Specifies a command to use for file selection, in place of the internal file
selection dialog used when sending files. The command shall include `%1` (a
temporary file path) which the command should write its result to. Examples:

nnn: `nnn -p '%1'`

ranger: `ranger --choosefiles='%1'`

### file_picker_persist_dir

Specifies whether the file selection dialog shall persist the directory of
last selected file.

### help_enabled

Specifies whether to display help bar. Controlled by Ctrl-g in run-time.

### help_prefix_ctrl

Specifies the prefix used in the help bar for control key combinations,
i.e. `^` results in `^X` being displayed for Ctrl-X. Users on macOS may
prefer the Apple modifier symbol `⌃` here.

### help_prefix_meta

Specifies the prefix used in the help bar for meta / alt key combinations,
i.e. `M-` results in `M-/` being displayed for Alt-/. Users on macOS may
prefer the Apple modifier symbol `⌥` here. Other alternatives include
`A-` and `Alt-`.

Note that these prefixes are only cosmetic, the actual key bindings are
configured in `key.conf`.

### home_fetch_all

Specifies whether `home` button shall repeatedly fetch all chat history.

### image_cache_days

Specifies after how many days without use the inline image previews in the
`images` directory below the nchat data directory are deleted. They are
checked when the first inline image is requested after startup, and are
recreated on demand. Set to `0` to never delete them. Default is `30`.

### inline_image_max_rows

Specifies the maximum height, in terminal rows, of a photo shown inline
in the message history. See `inline_images`.

### inline_images

Specifies whether photos and stickers are shown inline in the message
history, using the kitty graphics protocol (supported by Ghostty and
kitty, also inside tmux with `set -g allow-passthrough on`, see "Inline
Images" above for the full setup). Animated
stickers are shown as a still first frame, and Lottie (`.tgs`) stickers
keep the text line.
Supported values are `auto` (enabled when a supporting terminal is
detected), `on` and `off`. Images are converted with the first available
of `magick`, `sips`, `dwebp` and `ffmpeg` (needed for video stickers), and
cached in the `images` directory below the nchat data directory. Photos
and stickers are downloaded automatically when they become visible.
Unsupported terminals, missing tools and other file types keep the text
line. The Telegram sticker picker (Alt-g / Opt-g) shows sticker previews
when inline images are available, otherwise emoji captions only. Tab /
Shift-Tab switches between recent stickers and installed sticker sets,
typing an emoji or a keyword searches the installed sets, and Backspace
clears the search.

### inline_sticker_max_rows

Specifies the maximum height, in terminal rows, of a sticker shown
inline in the message history. See `inline_images`.

### linefeed_on_enter

Specifies if enter key press should be read as linefeed (LF `\12`).
Otherwise read as carriage return (CR `\15`). This setting is only
relevant if `key.conf` uses numerical key value for enter (LF `\12`,
CR `\15`). The key name `KEY_RETURN` always maps to the one in use. The
`newline` key (Alt-Enter) default `\33\12` assumes this is enabled, use
`\33\15` if disabled.

### link_open_command

Specifies a custom command to use for opening/viewing links. The
command shall include `%1` which will be replaced by the url
to open. If not specified, the following default commands are used:

Linux: `xdg-open >/dev/null 2>&1 '%1' &`

macOS: `open '%1' &`

Note: Omit the trailing `&` for commands taking over the terminal, for
example `w3m -o confirm_qq=false '%1'` and `see '%1'`.

### list_enabled

Specifies whether to display chat list. Controlled by Ctrl-l in run-time.

### list_width

Specifies width of chat list.

### listdialog_show_filter

Specifies whether list dialogs should display the search filter input by user.

### mark_read_any_chat

Specifies whether nchat may mark messages as read before current chat has
been set (either through user activity or time elapsed since last old sync
message was received).

### mark_read_on_view

Specifies whether nchat should send message read receipts upon viewing. If
false nchat will only mark the messages read upon `next_page` (page down),
`end` (end) or upon sending a message/file in the chat.

### mark_read_when_inactive

Controls whether nchat marks messages in the current chat as read while the
terminal is inactive.

### message_edit_command

Specifies a custom command to use for external editor compose. If not
specified, nchat will use `EDITOR` environment variable if set, or
otherwise use `nano`.

### message_open_command

Specifies a custom command to use for opening/viewing message text part. If
not specified, nchat will use `PAGER` environment variable if set, or
otherwise use `less`.

### muted_indicate_unread

Specifies whether chat list should indicate unread status `*` for muted chats.
This also determines whether the such chats are included in jump to unread.

### muted_notify_unread

Specifies whether to notify (terminal bell) new unread messages in muted chats.

### muted_position_by_timestamp

Specifies whether chat list position of muted chats should reflect the time of
their last received/sent message. Otherwise muted chats are listed last.

### notify_every_unread

Specifies whether to notify upon receiving more messages in an unread chat.

### online_status_share

Share online status with other users.
Note: Disabling this stops updates on other users online/typing status for
WhatsApp.

### online_status_dynamic

Dynamically update online status based on terminal active state.
Note: Enabling this stops updates on other users online/typing status for
WhatsApp when the terminal is not active.

### phone_number_indicator

Specifies status bar text to indicate phone number of the current chat is
available. This field may contain `%1` which will be replaced with the actual
phone number of the contact. Other examples: `🎧`

### pinned_indicator

Specifies text to suffix message header in message view for messages that have
been pinned.

### proxy_indicator

Specifies top bar text to indicate proxy is enabled.

### read_indicator

Specifies text to indicate a message has been read by the receiver.

### reactions_enabled

Specifies whether to display reactions.

### sent_align_right

Specifies whether your own messages are shown on the right side of the message
history and other people's on the left, instead of everything on the left. Both
sides use at most 75% of the pane width, and the sender line, text, quote,
reactions and inline images of a message are aligned together.

### sent_indicator

Specifies text to indicate a message has been sent but not yet delivered.

### spell_check_command

Specifies a custom command to use for spell checking composed messages. If not
specified, nchat checks if `aspell` or `ispell` is available on the system (in
that order), and uses the first found.

### status_broadcast

Specifies (WhatsApp) Status Updates chat level of visibility:

    0 = hidden
    1 = visible and muted  <- default
    2 = visible

### status_enabled

Specifies whether to display status bar.

### syncing_indicator

Specifies text to suffix attachment filenames in message view for downloads
in progress.

### tab_size

Specifies number of spaces to insert when user inputs a tab character.

### terminal_bell_active

Specifies whether new message shall trigger terminal bell when nchat terminal
window is active.

### terminal_bell_inactive

Specifies whether new message shall trigger terminal bell when nchat terminal
window is inactive.

### terminal_title

Specifies custom terminal title, ex: `terminal_title=nchat - telegram`.

### top_enabled

Specifies whether to display top bar. Controlled by Ctrl-p in run-time.

### top_show_version

Specifies whether to display nchat version in top bar.

### transfer_send_caption

Specifies if entered text should be sent as caption when transferring a file.

### typing_status_share

Specifies whether to share typing status with other user(s) in the
conversation.

### undo_clear_input

Specifies whether undoing clearing of input buffer (by pressing ctrl-c again)
is enabled.

### unread_indicator

Specifies the character to suffix chats with unread messages in the chat list.

### vim_keys

Specifies whether vim-style navigation keys (`j k gg G Ctrl-d Ctrl-u / i l h u z
n r e y x`) are enabled in chat list and message history focus, and Ctrl-w
followed by `h j k l` moves between panes from any focus (the Ctrl-w prefix key
is `pane_prefix` in `key.conf`). See "Focus and Navigation" above. When
disabled, letters type into the text entry from any focus and Ctrl-w is not a
prefix.

~/.config/nchat/key.conf
------------------------
This configuration file holds user interface key bindings. Default content:

    archive_chat=\33\141
    auto_compose=\33\151
    backspace=KEY_BACKSPACE
    backspace_alt=KEY_ALT_BACKSPACE
    backward_kill_word=\33\177
    backward_word=
    begin_line=KEY_CTRLA
    cancel=KEY_CTRLC
    clear=KEY_CTRLC
    copy=\33\143
    cut=\33\170
    decrease_list_width=\33\54
    delete=KEY_DC
    delete_chat=\33\144
    delete_line_after_cursor=KEY_CTRLK
    delete_line_before_cursor=KEY_CTRLU
    delete_msg=KEY_CTRLD
    down=KEY_DOWN
    edit_msg=KEY_CTRLZ
    end=KEY_END
    end_line=KEY_CTRLE
    ext_call=\33\164
    ext_edit=\33\145
    find=\33\57
    find_next=\33\77
    focus_entry=KEY_CTRL[
    focus_next=KEY_TAB
    focus_prev=KEY_BTAB
    folder_next=KEY_SRIGHT
    folder_prev=KEY_SLEFT
    forward_msg=\33\162
    forward_word=
    goto_chat=KEY_CTRLN
    home=KEY_HOME
    increase_list_width=\33\56
    jump_pinned=KEY_CTRLP
    jump_quoted=\33\161
    kill_word=
    left=KEY_LEFT
    linebreak=KEY_RETURN
    newline=\33\12
    next_chat=\33\152
    next_page=KEY_NPAGE
    ok=KEY_RETURN
    open=KEY_CTRLV
    open_link=\33\165
    open_msg=\33\167
    other_commands_help=KEY_CTRLO
    pane_prefix=KEY_CTRLW
    paste=\33\166
    pin=\33\160
    prev_chat=\33\153
    prev_page=KEY_PPAGE
    quit=KEY_CTRLQ
    react=\33\163
    right=KEY_RIGHT
    save=KEY_CTRLR
    select_contact=\33\156
    select_emoji=KEY_CTRLS
    select_mention=\33\62
    select_sticker=\33\147
    send_enter=KEY_RETURN
    send_msg=KEY_CTRLX
    spell=\33\44
    tab=\33\11
    terminal_focus_in=KEY_FOCUS_IN
    terminal_focus_out=KEY_FOCUS_OUT
    terminal_resize=KEY_RESIZE
    toggle_emoji=KEY_CTRLY
    toggle_help=KEY_CTRLG
    toggle_list=KEY_CTRLL
    toggle_top=KEY_NONE
    transfer=KEY_CTRLT
    unread_chat=KEY_CTRLF
    up=KEY_UP

The key bindings may be specified in the following formats:
- Ncurses macro (ex: `KEY_CTRLK`)
- Hex key code (ex: `0x22e`)
- Octal key code sequence (ex: `\033\177`)
- Plain-text single-char ASCII (ex: `r`)
- Disable key binding (`KEY_NONE`)

To determine the key code sequence for a key, one can run nchat in key code
dump mode `nchat -k` which will output the octal code, and ncurses macro name
(if present).

~/.config/nchat/color.conf
--------------------------
This configuration file holds user interface color settings. Default content:

    default_color_bg=
    default_color_fg=
    dialog_attr=
    dialog_attr_selected=reverse
    dialog_color_bg=
    dialog_color_fg=
    dim_attr=dim
    entry_attr=
    entry_color_bg=
    entry_color_fg=
    focus_attr=reverse
    focus_color_bg=
    focus_color_fg=
    help_attr=reverse
    help_color_bg=black
    help_color_fg=white
    history_name_attr=underline
    history_name_attr_selected=reverse
    history_name_recv_color_bg=
    history_name_recv_color_fg=
    history_name_recv_group_color_bg=
    history_name_recv_group_color_fg=
    history_name_sent_color_bg=
    history_name_sent_color_fg=gray
    history_text_attachment_color_bg=
    history_text_attachment_color_fg=gray
    history_text_attr=
    history_text_attr_selected=reverse
    history_text_quoted_color_bg=
    history_text_quoted_color_fg=gray
    history_text_reaction_color_bg=
    history_text_reaction_color_fg=gray
    history_text_recv_color_bg=
    history_text_recv_color_fg=
    history_text_recv_group_color_bg=
    history_text_recv_group_color_fg=
    history_text_sent_color_bg=
    history_text_sent_color_fg=gray
    list_attr=
    list_attr_selected=reverse
    list_color_bg=
    list_color_fg=
    list_color_unread_bg=
    list_color_unread_fg=
    list_name_attr=bold
    list_preview_attr=
    list_preview_color_bg=
    list_preview_color_fg=gray
    listborder_attr=
    listborder_color_bg=
    listborder_color_fg=
    status_attr=reverse
    status_color_bg=
    status_color_fg=
    top_attr=reverse
    top_color_bg=
    top_color_fg=

Supported text attributes `_attr` (defaults to `normal` if not specified):

    normal
    underline
    reverse
    bold
    italic

Supported text background `_bg` and foreground `_fg` colors:

    black
    red
    green
    yellow
    blue
    magenta
    cyan
    white
    gray
    bright_black (same as gray)
    bright_red
    bright_green
    bright_yellow
    bright_blue
    bright_magenta
    bright_cyan
    bright_white

Custom colors may be specified using hex RGB code, for example `0xff8937`.

The `history_name_recv_group_color` and `history_text_recv_group_color`
parameters also supports the special value `usercolor`. When set, nchat will
determine which color to use for a user, based on a hash of their user id
used to pick a color from the list in `~/.config/nchat/usercolor.conf`.

Themes
------
Color themes are available in the source package under the `themes` directory
and can be used by copying to `~/.config/nchat/`.

Available themes:

    ayu-dark
    basic-color
    catppuccin-mocha
    default
    dracula
    espresso
    gruvbox-dark
    gruvbox-dark-hard
    purple-light
    solarized-dark-higher-contrast
    tokyo-night
    tomorrow-night
    zenbones-dark
    zenburned

See the [Themes wiki-page](https://github.com/d99kris/nchat/wiki/Themes) for
example screenshots.

### Installing theme from source

With a source code copy, simply copy the theme files to the config directory
(while nchat is not running). Example installing `dracula` theme:

    cp themes/dracula/* ~/.config/nchat/

### Installing theme from web

One can fetch current `master` copy of a theme from the github repository and
download to the config directory (while nchat is not running). Example
installing `dracula` theme:

    THEME="dracula" ; curl -L "https://raw.githubusercontent.com/d99kris/nchat/refs/heads/master/themes/${THEME}/{color.conf,usercolor.conf}" -o ~/.config/nchat/#1

### Generating theme from iTerm2-Color-Schemes

To generate nchat themes based on
[iTerm2 Color Schemes](https://github.com/mbadolato/iTerm2-Color-Schemes)
refer to [Generating nchat themes from iTerm2 Color Schemes](/themes/templates/iterm2-color-schemes/README.md).

General
-------
Deleting a configuration entry line (while nchat is not running) and starting
nchat will populate the configuration file with the default entry.

Configuration files are rewritten when nchat exits. If a file has been
modified externally while nchat is running, it is left untouched, and any
settings changed in the user interface during that session are discarded.


Protocol-Specific Configuration
===============================

The following configuration files (listed with current default values) can be
used to configure nchat.

~/.config/nchat/profiles/Telegram_+nnnnn/telegram.conf
------------------------------------------------------
This configuration file holds protocol-specific settings for Telegram. Default
content:

    local_key=
    markdown_enabled=1
    markdown_version=1
    profile_display_name=

### local_key

For internal use by nchat only.

### markdown_enabled

Specifies whether to enable Markdown <-> text conversion for text messages
(default enabled).

### markdown_version

Specifies which Telegram Markdown version to use (default 1).

### profile_display_name

Specifies an optional short/display name in the status bar when using nchat
with multiple profiles. The default profile name is `Telegram` or
`Telegram_+nnnnn` (when more than one Telegram profile is set up) if this
setting is not specified.

~/.config/nchat/profiles/WhatsAppMd_+nnnnn/whatsappmd.conf
----------------------------------------------------------
This configuration file holds protocol-specific settings for WhatsApp. Default
content:

    profile_display_name=

### profile_display_name

Specifies an optional short/display name in the status bar when using nchat
with multiple profiles. The default profile name is `WhatsAppMd` or
`WhatsAppMd_+nnnnn` (when more than one WhatsAppMd profile is set up) if this
setting is not specified.

~/.config/nchat/profiles/Signal_+nnnnn/signal.conf
----------------------------------------------------------
This configuration file holds protocol-specific settings for Signal. Default
content:

    profile_display_name=

### profile_display_name

Specifies an optional short/display name in the status bar when using nchat
with multiple profiles. The default profile name is `Signal` or
`Signal_+nnnnn` (when more than one Signal profile is set up) if this
setting is not specified.


FAQ
===

### 1. Alt/Opt-keyboard shortcuts are not working?

For Linux please ensure the terminal is configured with
[meta to send escape](https://askubuntu.com/questions/442644/how-to-make-xterm-to-send-the-alt-key-to-emacs-as-meta).
For macOS Terminal ensure that the Terminal profile keyboard setting
"Use Option as Meta key" is enabled.

If issues are still encountered, please use `nchat -k` (keydump mode) to
determine the key codes and modify `~/.config/nchat/key.conf` accordingly.

### 2. Send messages with Enter key?

By default Enter sends (`enter_sends=1` in `~/.config/nchat/ui.conf`) and
Alt/Opt-Enter inserts a linebreak (`newline` in `~/.config/nchat/key.conf`,
default `\33\12`). If Alt/Opt-Enter does not insert a linebreak, use
`nchat -k` to check the codes sent by the terminal. A terminal sending ESC CR
needs `linefeed_on_enter=0` in `ui.conf` and `newline=\33\15` in `key.conf`.
Set `enter_sends=0` to have Enter insert a linebreak and send with Ctrl-x.

### 3. Custom colors are not shown when running nchat in tmux?

Please try to run nchat with a TERM supporting custom colors, e.g:

    TERM=xterm-256color nchat

### 4. Sent messages are not visible?

For terminals with eight colors (or more) the default color theme displays
sent messages in gray (shaded). Some terminals may wrongly report supporting
more colors than two, or the terminal may be set up with gray mapped to black.
In this case sent / own messages may appear invisible. To avoid nchat using
gray one can edit `~/.config/nchat/color.conf` and remove occurances of `gray`.

### 5. How to use Telegram and WhatsApp concurrently or switch between them?

The **recommended** method is to set up nchat with one config directory per
protocol/phone number, and run each instance in separate terminal windows/tabs.
To simplify such usage one can set up aliases, for example:

    alias telegram='nchat -d ~/.config/nchat-telegram'
    alias whatsapp='nchat -d ~/.config/nchat-whatsapp'

Then use regular setup for them (separately), for example:

    telegram -s
    whatsapp -s

The **alternative** method is to set up multiple protocol accounts in a single
nchat config directory. For each protocol/phone nubmer, run setup mode and exit
after initial sync:

    nchat -s

### 6. How to set up Telegram / WhatsApp without scanning a QR code?

By default setting up a Telegram or WhatsApp account will display a QR code
to be scanned using the mobile application on the primary device. As an
alternative one can set an environment flag to use authentication code
(Telegram) or pairing code (WhatsApp) instead:

    USE_PAIRING_CODE=1 nchat -s

### 7. No QR code is shown when setting up Telegram / WhatsApp / Signal?

By default nchat will attempt to detect if the system is capable of viewing
images using a GUI image viewer, and if detected (indicated by "has gui" in
the log) it displays the QR code using default image viewer. If that for some
reason fails, one can use pairing code (see above), or force nchat to display
the QR code in the terminal:

    USE_QR_TERMINAL=1 nchat -s

### 8. Build fails with `c++: fatal error: Killed signal terminated program cc1plus`?

This often means that OOM killer has terminated the compilation due to the
system running out of free RAM.

If the system has **less than 4 GB RAM**, please refer to
[Building on Low Memory Systems](/doc/LOWMEMORY.md).

If the system has **4 GB RAM or more**, the problem can occur if parallelism
is set too high, which is likely to be encountered when installing from the
Arch Linux AUR package. A workaround for the AUR package is to manually
restrict max number of parallel jobs to `available RAM in GB` divided by 4.
For example a system with 8 GB would then need to use max 8 / 4 = 2 jobs:

    CMAKE_BUILD_PARALLEL_LEVEL=2 yay -S nchat

Alternatively one can [Build from Source](#build-from-source) using the
`make.sh` script, which sets parallel job count based on the system
capabilities.

### 9. Terminal transparency is not working?

If on Linux, try removing any custom default background set up, i.e. set
`default_color_bg=` in `~/.config/nchat/color.conf`.


Project Scope
=============
nchat is feature-complete and in maintenance mode. It is not intended to be a
full-featured client on par with official Telegram / WhatsApp / Signal
clients, but rather a light-weight client providing essential functionality
suitable for the terminal. See [Project Scope](/doc/SCOPE.md) for further
details.


Contributions
=============
Please refer to [Project Scope](/doc/SCOPE.md),
[Contributing Guidelines](/doc/CONTRIBUTING.md) and
[Design Notes](/doc/DESIGN.md).


Alternatives
============
Terminal-based Telegram clients:

- [tg](https://github.com/paul-nameless/tg)
- [tgt](https://github.com/FedericoBruzzone/tgt)

Terminal-based WhatsApp clients:

- [whatscli](https://github.com/normen/whatscli)

Terminal-based Signal clients:

- [gurk-rs](https://github.com/boxdot/gurk-rs)


Technical Details
=================

Custom API Id / Hash
--------------------
nchat uses its own Telegram API id and hash by default. To use custom id/hash,
obtained from [https://my.telegram.org/](https://my.telegram.org/) one may set
environment variables `TG_APIID` and `TG_APIHASH` when setting up a new Telegram
account. Example (below values must be changed to valid api id/hash):

    TG_APIID="123456" TG_APIHASH="aaeaeab342aaa23423" nchat -s

Third-party Libraries
---------------------
nchat is primarily implemented in C++ with some parts in Go. Its source tree
includes the source code of the following third-party libraries:

- [apathy](https://github.com/dlecocq/apathy) -
  Copyright 2013 Dan Lecocq -
  [MIT License](/lib/ncutil/ext/apathy/LICENSE)

- [cereal](https://github.com/USCiLab/cereal) -
  Copyright 2013 Randolph Voorhies, Shane Grant -
  [BSD-3 License](/lib/ncutil/ext/cereal/LICENSE)

- [clip](https://github.com/dacap/clip) -
  Copyright 2015 David Capello -
  [MIT License](/lib/ncutil/ext/clip/LICENSE.txt)

- [QR-Code-generator](https://github.com/nayuki/QR-Code-generator) -
  Copyright 2022 Project Nayuki -
  [MIT License](/lib/ncutil/ext/QR-Code-generator/Readme.markdown)

- [mautrix-signal](https://github.com/mautrix/signal) -
  Copyright 2020 Tulir Asokan -
  [AGPL License](/lib/sgchat/go/ext/signal/LICENSE)

- [stb](https://github.com/nothings/stb) -
  Copyright 2017 Sean Barrett -
  [MIT License](/lib/ncutil/ext/stb/LICENSE)

- [sqlite_modern_cpp](https://github.com/SqliteModernCpp/sqlite_modern_cpp) -
  Copyright 2017 aminroosta -
  [MIT License](/lib/ncutil/ext/sqlite_modern_cpp/License.txt)

- [tdlib](https://github.com/tdlib/td) -
  Copyright 2014 Aliaksei Levin, Arseny Smirnov -
  [Boost License](/lib/tgchat/ext/td/LICENSE_1_0.txt)

- [whatsmeow](https://github.com/tulir/whatsmeow) -
  Copyright 2022 Tulir Asokan -
  [MPL License](/lib/wmchat/go/ext/whatsmeow/LICENSE)

Binaries are distributed with a combined THIRD_PARTY_LICENSES file.

The [tdlib](https://github.com/tdlib/td),
[whatsmeow](https://github.com/tulir/whatsmeow) and
[signal](https://github.com/mautrix/signal)
libraries are actively
developed and need to be updated and integrated into nchat on a regular
basis by nchat maintainer(s). To facilitate this there are scripts available
to update to latest (or a specific) version of these libraries. Example usages:

    ./utils/tdlib-update 8517026

    ./utils/whatsmeow-update 7aedaa1

    ./utils/signal-update 93da772


License
=======
Source is distributed under the [MIT license](/LICENSE).

Binaries are distributed under the [GNU AGPL v3 license](/LICENSE.AGPL-3.0),
or the [GNU GPL v3 license](/LICENSE.GPL-3.0) if Signal support is disabled.


Keywords
========
command line, console-based, linux, macos, chat client, ncurses, telegram,
terminal-based, tui.
