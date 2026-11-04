Road to Alfendrof for Linux
==========================

Extract the archive into a user-owned directory (for example ~/Games), then run:
    cd ~/Games/RoadToAlfendrof
    ./t2f
Keep t2f and assets together. Do not run the game as root.
Windowed launches start maximized; saved borderless/fullscreen choices still apply.

Alternatively, download the AppImage, mark it executable and run it:
    chmod +x RoadToAlfendrof-VERSION-x86_64.AppImage
    ./RoadToAlfendrof-VERSION-x86_64.AppImage
Keep it in a user-owned writable folder. The AppImage runtime may require your
distribution's FUSE 2 compatibility library (libfuse2 / libfuse2t64 on Ubuntu).

The release targets x86_64 Linux with glibc 2.35 or newer (Ubuntu 22.04+).
Install your distribution's graphics/Vulkan driver and ALSA, udev, X11/Wayland
runtime libraries. curl and tar must be available for automatic updates.
On Ubuntu:
    sudo apt install libasound2 libudev1 libx11-6 libxcursor1 libxi6 libxrandr2 libxkbcommon0 libwayland-client0 libvulkan1 curl
On Ubuntu 24.04+, use libasound2t64 instead of libasound2.

Installed release builds check GitHub for a newer release on startup. Downloads
run in the background. At the title screen the game closes, replaces its whole
installation directory, then restarts. The installation's parent directory must
be writable. Do not keep personal files in the installation directory: an update
replaces it. Read-only/system-managed installs must be updated manually.
An unavailable release or download leaves the current game usable.
AppImages update the original user-owned AppImage file in place and restart it;
the mounted AppDir is never modified. Debian/RPM system installations should be
updated through their package manager or by installing a newer package, not by
running the game as root.

Controls are in Options > Controls. Settings are stored separately at:
    ~/.config/Road to Alfendrof/settings.ron
For a log, start ./t2f from a terminal and include the output in a bug report.
