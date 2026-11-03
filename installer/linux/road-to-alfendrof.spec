Name:           road-to-alfendrof
Version:        %{game_version}
Release:        1
Summary:        Road to Alfendrof fighting game
# No source license is declared in the repository; do not invent an open-source license.
License:        LicenseRef-Proprietary
URL:            https://github.com/zerw0/road-to-alfendrof
ExclusiveArch:  x86_64

# dlopen-loaded libraries are invisible to RPM's automatic ELF dependency generator.
Requires:       libX11.so.6()(64bit)
Requires:       libXcursor.so.1()(64bit)
Requires:       libXi.so.6()(64bit)
Requires:       libXrandr.so.2()(64bit)
Requires:       libxkbcommon.so.0()(64bit)
Requires:       libwayland-client.so.0()(64bit)
Requires:       libwayland-cursor.so.0()(64bit)
Requires:       libwayland-egl.so.1()(64bit)
Requires:       libvulkan.so.1()(64bit)

# The executable is already a release build; preserve it without debug subpackages or build-id links.
%global debug_package %{nil}
%global __os_install_post %{nil}
%global _build_id_links none

%description
Road to Alfendrof fighting game, including its character, stage, music and artwork assets.
A Vulkan-capable graphics driver is required.

%install
mkdir -p %{buildroot}
cp -a %{staging}/usr %{buildroot}/

%files
/usr/bin/road-to-alfendrof
/usr/lib/road-to-alfendrof/
/usr/share/applications/road-to-alfendrof.desktop
/usr/share/icons/hicolor/256x256/apps/road-to-alfendrof.png
