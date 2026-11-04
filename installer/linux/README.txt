Road to Alfendrof for Linux
==========================

Choose one of the x86_64 Linux packages: portable tar.gz, AppImage, DEB or RPM.

Portable archive
----------------
Extract RoadToAlfendrof-Linux-x86_64.tar.gz into a user-owned directory
(for example ~/Games), then run:
    cd ~/Games/RoadToAlfendrof
    ./t2f
Keep t2f and assets together. Do not run the game as root.
Windowed launches start maximized; saved borderless/fullscreen choices still apply.

AppImage
--------
Download the AppImage, mark it executable and run it:
    chmod +x RoadToAlfendrof-VERSION-x86_64.AppImage
    ./RoadToAlfendrof-VERSION-x86_64.AppImage
Keep it in a user-owned writable folder. The AppImage runtime may require your
distribution's FUSE 2 compatibility library (libfuse2 / libfuse2t64 on Ubuntu).
Without FUSE, use:
    ./RoadToAlfendrof-VERSION-x86_64.AppImage --appimage-extract-and-run

System packages
---------------
Debian/Ubuntu:
    sudo apt install ./road-to-alfendrof_VERSION_amd64.deb
Fedora:
    sudo dnf install ./road-to-alfendrof-VERSION-1.x86_64.rpm
Launch Road to Alfendrof from the desktop menu or run road-to-alfendrof.
DEB/RPM dependencies are installed by the package manager. AppImage bundles its
linked libraries; the host still needs a working graphics/Vulkan driver.

Requirements
------------
The release targets x86_64 Linux with glibc 2.35 or newer (Ubuntu 22.04+,
Debian 12+ and current Fedora). Install your distribution's graphics/Vulkan
driver. Portable installs also need ALSA, udev and X11/Wayland runtime libraries.
curl and tar must be available for automatic updates.
On Ubuntu:
    sudo apt install libasound2 libudev1 libx11-6 libxcursor1 libxi6 libxrandr2 libxkbcommon0 libwayland-client0 libvulkan1 curl
On Ubuntu 24.04+, use libasound2t64 instead of libasound2.

Automatic updates
-----------------
User-owned portable and AppImage release builds check GitHub for a newer release
on startup. Downloads run in the background; installation waits for the title
screen, closes the game, installs the update and restarts.
Portable updates replace the whole RoadToAlfendrof installation directory. Its
parent must be writable. Keep t2f and assets together, and do not keep personal
files inside the installation: an update replaces it.
AppImages replace the original writable AppImage file, never the mounted AppDir.
An unavailable release or failed download leaves the current game usable.
The installer keeps a rollback copy during replacement so a failed installation
can restore the previous version.
DEB/RPM installations are system-managed: update through apt/dnf or install a
newer package manually. Never run the game as root to enable automatic updates.

Controls are in Options > Controls. Settings are stored separately at:
    ~/.config/Road to Alfendrof/settings.ron
For a log, launch the game from a terminal and include the output in a bug report.

Building packages
-----------------
After cargo build --locked --release --bin t2f, run from the repository root:
    LINUXDEPLOY=/path/to/linuxdeploy.AppImage bash installer/linux/package.sh
This produces DEB, RPM, linuxdeploy AppImage (with bundled linked libraries),
and the portable updater archive in dist/. See .github/workflows/linux.yml for
the required packaging tools and the pinned linuxdeploy download.
For only the portable archive:
    sh installer/linux/portable.sh target/release/t2f dist x86_64
